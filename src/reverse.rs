use crate::db::Db;
use crate::error::RumblerError;
use crate::migration::DiscoveredMigration;
use crate::{lint, migration};

pub struct ReverseOptions {
    pub all: bool,
    pub no_save: bool,
    pub dry_run: bool,
    pub migration_filter: Option<String>,
}

pub fn run(db: &mut Db, directory: &str, options: &ReverseOptions) -> Result<(), RumblerError> {
    let available = migration::discover(directory)?;
    db.initialize(&available)?;

    let applied = db.get_applied()?;
    let _ = migration::check_consistency(&available, &applied)?;

    if applied.is_empty() {
        log::info!("no applied migrations to reverse");
        return Ok(());
    }

    let applied_names = applied.iter().map(|m| &m.name).collect::<Vec<_>>();

    let mut only_applied = available
        .into_iter()
        .rev()
        .filter(|m| applied_names.contains(&&m.name));

    let to_reverse = find_migrations_to_reverse(options, &mut only_applied)?;

    for migration in to_reverse {
        lint::lint_file(&migration.path)?;

        log::info!("reversing: {}", migration.name);
        for statement in migration.down() {
            if options.dry_run {
                println!("{statement}");
            } else {
                log::debug!("executing: {statement}");
                db.execute(statement)?;
            }
        }

        if !options.dry_run && !options.no_save {
            db.remove_applied(&migration.name)?;
        }

        log::info!("reversed: {}", migration.name);
    }

    Ok(())
}

fn find_migrations_to_reverse(
    options: &ReverseOptions,
    only_applied: &mut impl Iterator<Item = DiscoveredMigration>,
) -> Result<Vec<DiscoveredMigration>, RumblerError> {
    if options.all {
        Ok(only_applied.collect())
    } else if let Some(ref filter) = options.migration_filter {
        let found = only_applied.find(|m| m.name == *filter).ok_or_else(|| {
            RumblerError::Migration(format!("applied migration not found: {filter}"))
        })?;

        Ok(vec![found])
    } else {
        Ok(only_applied.take(1).collect())
    }
}
