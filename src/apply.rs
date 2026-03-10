use crate::db::Db;
use crate::error::RumblerError;
use crate::migration::DiscoveredMigration;
use crate::{lint, migration};

pub struct ApplyOptions {
    pub all: bool,
    pub no_save: bool,
    pub dry_run: bool,
    pub migration_filter: Option<String>,
}

pub fn run(db: &mut Db, directory: &str, options: &ApplyOptions) -> Result<(), RumblerError> {
    let available = migration::discover(directory)?;
    db.initialize(&available)?;

    let applied = db.get_applied()?;
    let pending = migration::check_consistency(&available, &applied)?;

    if pending.is_empty() {
        log::info!("no pending migrations");
        return Ok(());
    }

    let to_apply: Vec<DiscoveredMigration> = find_migrations_to_apply(options, pending)?;

    for migration in to_apply {
        lint::lint_file(&migration.path)?;

        log::info!("applying: {}", migration.name);
        for statement in migration.up() {
            if options.dry_run {
                println!("{statement}");
            } else {
                log::debug!("executing: {statement}");
                db.execute(statement)?;
            }
        }

        if !options.dry_run && !options.no_save {
            db.add_applied(
                &migration.name,
                &migration.path.to_string_lossy(),
                &migration.checksum,
            )?;
        }

        log::info!("applied: {}", migration.name);
    }

    Ok(())
}

fn find_migrations_to_apply(
    options: &ApplyOptions,
    pending: Vec<DiscoveredMigration>,
) -> Result<Vec<DiscoveredMigration>, RumblerError> {
    if options.all {
        Ok(pending)
    } else if let Some(ref filter) = options.migration_filter {
        let found = pending
            .into_iter()
            .find(|m| m.name == *filter)
            .ok_or_else(|| {
                RumblerError::Migration(format!("pending migration not found: {filter}"))
            })?;
        Ok(vec![found])
    } else {
        Ok(pending.into_iter().take(1).collect())
    }
}
