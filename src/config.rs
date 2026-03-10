mod lib;
use lib::*;

use crate::error::RumblerError;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Config {
    pub database: String,
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub role: String,
    pub schema: String,
    pub search_path: String,
    pub sslmode: String,
    pub directory: String,
    pub rambler_table: String,
    pub rumbler_table: String,
}

pub fn load(
    config_path: Option<impl Into<String>>,
    environment: Option<impl Into<String>>,
) -> Result<Config, RumblerError> {
    let mut partial_config = try_load_config(config_path)?;
    merge_environment(&mut partial_config, environment)?;

    let rambler_table = partial_config.table.unwrap_or("migrations".into());
    let config = Config {
        database: partial_config.database.unwrap_or_default(),
        host: partial_config.host.unwrap_or("localhost".into()),
        port: partial_config.port.unwrap_or(5432),
        user: partial_config.user.unwrap_or("postgres".into()),
        password: partial_config.password.unwrap_or_default(),
        role: partial_config.role.unwrap_or_default(),
        schema: partial_config.schema.clone().unwrap_or("public".into()),
        search_path: partial_config
            .search_path
            .or(partial_config.schema)
            .unwrap_or("public".into()),
        sslmode: partial_config.sslmode.unwrap_or("disable".into()),
        directory: partial_config.directory.unwrap_or(".".into()),
        rambler_table: rambler_table.clone(),
        rumbler_table: format!("rumbler_{}", rambler_table),
    };

    if config.database.is_empty() {
        return Err(RumblerError::ConfigParse(
            "database name is required".into(),
        ));
    }

    Ok(config)
}

#[derive(PartialEq)]
enum ConfigPath {
    Toml(PathBuf),
    Json(PathBuf),
    None,
}

fn merge_environment(
    config: &mut PartialConfig,
    environment: Option<impl Into<String>>,
) -> Result<(), RumblerError> {
    if let Some(environment) = environment {
        let environment = environment.into();
        let env = config
            .environments
            .get(environment.as_str())
            .ok_or_else(|| RumblerError::UnknownEnvironment(environment))?;

        if let Some(_) = env.database {
            config.database = env.database.clone();
        }
        if let Some(_) = env.host {
            config.host = env.host.clone();
        }
        if let Some(_) = env.port {
            config.port = env.port;
        }
        if let Some(_) = env.user {
            config.user = env.user.clone();
        }
        if let Some(_) = env.password {
            config.password = env.password.clone();
        }
        if let Some(_) = env.role {
            config.role = env.role.clone();
        }
        if let Some(_) = env.schema {
            config.schema = env.schema.clone();
        }
        if let Some(_) = env.sslmode {
            config.sslmode = env.sslmode.clone();
        }
        if let Some(_) = env.directory {
            config.directory = env.directory.clone();
        }
        if let Some(_) = env.table {
            config.table = env.table.clone();
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sealed_test::prelude::*;
    use std::io::Write;

    #[sealed_test]
    fn test_defaults() {
        let mut f = std::fs::File::create("rumbler.toml").unwrap();
        writeln!(f, r#"database = "testdb""#).unwrap();

        let config = load(None::<String>, None::<String>).unwrap();
        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, 5432);
        assert_eq!(config.user, "postgres");
        assert_eq!(config.password, "");
        assert_eq!(config.schema, "public");
        assert_eq!(config.search_path, "public");
        assert_eq!(config.sslmode, "disable");
        assert_eq!(config.directory, ".");
        assert_eq!(config.rambler_table, "migrations");
        assert_eq!(config.rumbler_table, "rumbler_migrations");
    }

    #[sealed_test(env = [
            ("RAMBLER_DATABASE", "foo-database"),
            ("RAMBLER_DIRECTORY", "foo-directory"),
            ("RAMBLER_HOST", "foo-host"),
            ("RAMBLER_PASSWORD", "foo-password"),
            ("RAMBLER_PORT", "1234"),
            ("RAMBLER_ROLE", "foo-role"),
            ("RAMBLER_SCHEMA", "foo-schema"),
            ("RAMBLER_SSLMODE", "foo-sslmode"),
            ("RAMBLER_TABLE", "foo-table"),
            ("RAMBLER_USER", "foo-user"),
        ]
    )]
    fn test_env_rambler_compat() {
        let config = load(None::<String>, None::<String>).unwrap();
        assert_eq!(config.database, "foo-database");
        assert_eq!(config.host, "foo-host");
        assert_eq!(config.port, 1234);
        assert_eq!(config.user, "foo-user");
        assert_eq!(config.password, "foo-password");
        assert_eq!(config.schema, "foo-schema");
        assert_eq!(config.search_path, "foo-schema");
        assert_eq!(config.sslmode, "foo-sslmode");
        assert_eq!(config.directory, "foo-directory");
        assert_eq!(config.rambler_table, "foo-table");
        assert_eq!(config.rumbler_table, "rumbler_foo-table");
    }

    #[sealed_test(env = [
            ("RUMBLER_DATABASE", "foo-database"),
            ("RUMBLER_DIRECTORY", "foo-directory"),
            ("RUMBLER_HOST", "foo-host"),
            ("RUMBLER_PASSWORD", "foo-password"),
            ("RUMBLER_PORT", "1234"),
            ("RUMBLER_ROLE", "foo-role"),
            ("RUMBLER_SCHEMA", "foo-schema"),
            ("RUMBLER_SEARCH_PATH", "foo-search-path"),
            ("RUMBLER_SSLMODE", "foo-sslmode"),
            ("RUMBLER_TABLE", "foo-table"),
            ("RUMBLER_USER", "foo-user"),
        ]
    )]
    fn test_env_rumbler_specific() {
        let config = load(None::<String>, None::<String>).unwrap();
        assert_eq!(config.database, "foo-database");
        assert_eq!(config.host, "foo-host");
        assert_eq!(config.port, 1234);
        assert_eq!(config.user, "foo-user");
        assert_eq!(config.password, "foo-password");
        assert_eq!(config.schema, "foo-schema");
        assert_eq!(config.search_path, "foo-search-path");
        assert_eq!(config.sslmode, "foo-sslmode");
        assert_eq!(config.directory, "foo-directory");
        assert_eq!(config.rambler_table, "foo-table");
        assert_eq!(config.rumbler_table, "rumbler_foo-table");
    }

    #[sealed_test]
    fn test_parse_toml_config() {
        let mut f = std::fs::File::create("rumbler.toml").unwrap();
        writeln!(
            f,
            r#"
database = "testdb"
host = "db.example.com"
port = 5433
user = "admin"
password = "secret"
schema = "myschema"
search_path = "myschema,public"
sslmode = "require"
directory = "migrations"
table = "schema_migrations"

[environments.staging]
database = "staging_db"
host = "staging.example.com"
"#
        )
        .unwrap();

        let config = load(None::<String>, None::<String>).unwrap();
        assert_eq!(config.database, "testdb");
        assert_eq!(config.host, "db.example.com");
        assert_eq!(config.port, 5433);
        assert_eq!(config.user, "admin");
        assert_eq!(config.password, "secret");
        assert_eq!(config.schema, "myschema");
        assert_eq!(config.search_path, "myschema,public");
        assert_eq!(config.sslmode, "require");
        assert_eq!(config.directory, "migrations");
        assert_eq!(config.rambler_table, "schema_migrations");
        assert_eq!(config.rumbler_table, "rumbler_schema_migrations");

        let config = load(None::<String>, Some("staging")).unwrap();
        assert_eq!(config.database, "staging_db");
        assert_eq!(config.host, "staging.example.com");
        assert_eq!(config.port, 5433); // inherited
    }

    #[sealed_test]
    fn test_parse_json_config() {
        let mut f = std::fs::File::create("rambler.json").unwrap();
        writeln!(
            f,
            r#"{{
  "database": "jsondb",
  "host": "localhost",
  "port": 5432
}}"#
        )
        .unwrap();

        let config = load(None::<String>, None::<String>).unwrap();
        assert_eq!(config.database, "jsondb");
    }

    #[sealed_test]
    fn test_parse_absolute_path() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("config.toml");
        let mut f = std::fs::File::create(&config_path).unwrap();
        writeln!(f, r#"database = "testdb""#).unwrap();

        let config = load(config_path.to_str(), None::<String>).unwrap();
        assert_eq!(config.database, "testdb");
    }

    #[sealed_test]
    fn test_prioritizes_rumbler_config() {
        let mut f = std::fs::File::create("rumbler.toml").unwrap();
        writeln!(f, r#"database = "rumbler-db-wins""#).unwrap();
        let mut f = std::fs::File::create("rambler.json").unwrap();
        writeln!(f, r#"database = "rambler-db-loses""#).unwrap();

        let config = load(None::<String>, None::<String>).unwrap();
        assert_eq!(config.database, "rumbler-db-wins");
    }

    #[sealed_test(env = [("RAMBLER_DATABASE", "env-wins")])]
    fn test_prioritizes_env_vars() {
        let mut f = std::fs::File::create("rumbler.toml").unwrap();
        writeln!(f, r#"database = "rumbler-db-backup""#).unwrap();
        let mut f = std::fs::File::create("rambler.json").unwrap();
        writeln!(f, r#"database = "rambler-db-loses""#).unwrap();

        let config = load(None::<String>, None::<String>).unwrap();
        assert_eq!(config.database, "env-wins");
    }

    #[sealed_test]
    fn test_unknown_environment() {
        let mut f = std::fs::File::create("rumbler.toml").unwrap();
        writeln!(f, r#"database = "testdb""#).unwrap();

        let result = load(None::<String>, Some("nonexistent"));
        assert!(result.is_err());
    }

    #[sealed_test]
    fn test_missing_database() {
        let mut f = std::fs::File::create("rumbler.toml").unwrap();
        writeln!(f, r#"host = "localhost""#).unwrap();

        let result = load(None::<String>, None::<String>);
        assert!(result.is_err());
    }
}
