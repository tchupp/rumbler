use crate::config::Config;
use crate::error::RumblerError;
use crate::migration::{AppliedMigration, DiscoveredMigration};
use postgres::{Client, NoTls};

pub struct Db {
    client: Client,
    catalog: String,
    schema: String,
    rambler_table: String,
    rumbler_table: String,
}

impl Db {
    pub fn connect(config: &Config) -> Result<Self, RumblerError> {
        let conn_string = format!(
            "host={} port={} user={} password={} dbname={} options=--search_path={} sslmode={}",
            config.host,
            config.port,
            config.user,
            config.password,
            config.database,
            config.search_path,
            config.sslmode,
        );

        let mut client = Client::connect(&conn_string, NoTls)?;

        if !config.role.is_empty() {
            log::debug!("setting role: {}", config.role);
            client.batch_execute(&format!("SET ROLE {}", config.role))?;
        }

        Ok(Db {
            client,
            catalog: config.database.clone(),
            schema: config.schema.clone(),
            rambler_table: config.rambler_table.clone(),
            rumbler_table: config.rumbler_table.clone(),
        })
    }

    pub fn initialize(&mut self, migrations: &[DiscoveredMigration]) -> Result<(), RumblerError> {
        if !self.table_exists(&self.rumbler_table.clone())? {
            log::info!("creating migration table: {}", self.rumbler_table);
            let query = format!(
                "CREATE TABLE {} (\
                    migration VARCHAR(255) NOT NULL, \
                    path TEXT NOT NULL DEFAULT '', \
                    checksum VARCHAR(64) NOT NULL DEFAULT '', \
                    applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW()\
                )",
                self.rumbler_table,
            );
            self.client.batch_execute(&query)?;

            self.import_from_rambler(migrations)?;
        }
        Ok(())
    }

    fn import_from_rambler(
        &mut self,
        migrations: &[DiscoveredMigration],
    ) -> Result<(), RumblerError> {
        if !self.table_exists(&self.rambler_table.clone())? {
            log::debug!(
                "skipping import from rambler table '{}' because it does not exist",
                self.rambler_table
            );
            return Ok(());
        }
        if !self.get_applied()?.is_empty() {
            log::debug!(
                "skipping import from rambler table '{}' because rumbler already contains applied migrations",
                self.rambler_table
            );
            return Ok(());
        }

        let query = format!(
            "SELECT migration FROM {} ORDER BY migration ASC",
            self.rambler_table
        );
        let rambler_applied_migrations: Vec<String> = self
            .client
            .query(&query, &[])?
            .into_iter()
            .map(|row| row.get(0))
            .collect();
        if rambler_applied_migrations.is_empty() {
            return Ok(());
        }

        for rambler_applied_migration in &rambler_applied_migrations {
            let matching_applied_migration = migrations
                .iter()
                .find(|m| &m.name == rambler_applied_migration);
            match matching_applied_migration {
                None => {
                    log::warn!(
                        "migration '{}' found in rambler table '{}' not in available rumbler migrations, skipping import",
                        rambler_applied_migration,
                        self.rambler_table,
                    );
                    continue;
                }
                Some(matching_applied_migration) => {
                    log::debug!(
                        "importing migration '{}' from rambler table '{}'",
                        matching_applied_migration.name,
                        self.rambler_table,
                    );
                    self.add_applied(
                        &matching_applied_migration.name,
                        &matching_applied_migration.path.to_string_lossy(),
                        &matching_applied_migration.checksum,
                    )?;
                }
            }
        }

        let count = rambler_applied_migrations.len();
        log::info!(
            "imported {count} migration(s) from rambler table '{}'",
            self.rambler_table
        );

        Ok(())
    }

    fn table_exists(&mut self, table_name: &str) -> Result<bool, RumblerError> {
        let row = self.client.query_one(
            "SELECT COUNT(*) FROM information_schema.tables WHERE table_catalog = $1 AND table_schema = $2 AND table_name = $3",
            &[&self.catalog, &self.schema, &table_name],
        )?;
        let count: i64 = row.get(0);
        Ok(count > 0)
    }

    pub fn get_applied(&mut self) -> Result<Vec<AppliedMigration>, RumblerError> {
        let query = format!(
            "SELECT migration, path, checksum FROM {} ORDER BY migration ASC",
            self.rumbler_table
        );
        let rows = self.client.query(&query, &[])?;
        let names = rows
            .iter()
            .map(|r| AppliedMigration {
                name: r.get(0),
                path: r.get(1),
                checksum: r.get(2),
            })
            .collect();

        Ok(names)
    }

    pub fn add_applied(
        &mut self,
        name: &str,
        path: &str,
        checksum: &str,
    ) -> Result<(), RumblerError> {
        let query = format!(
            "INSERT INTO {} (migration, path, checksum) VALUES ($1, $2, $3)",
            self.rumbler_table,
        );
        self.client.execute(&query, &[&name, &path, &checksum])?;
        Ok(())
    }

    pub fn remove_applied(&mut self, name: &str) -> Result<(), RumblerError> {
        let query = format!("DELETE FROM {} WHERE migration = $1", self.rumbler_table);
        self.client.execute(&query, &[&name])?;
        Ok(())
    }

    pub fn execute(&mut self, statement: &str) -> Result<(), RumblerError> {
        self.client
            .batch_execute(statement)
            .map_err(|e| RumblerError::StatementError {
                statement: statement.to_string(),
                source: e,
            })?;
        Ok(())
    }
}
