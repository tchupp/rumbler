use crate::error::RumblerError;
use itertools::{EitherOrBoth, Itertools};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::{fmt, fs};

#[derive(PartialEq, Debug, Clone)]
pub struct DiscoveredMigration {
    pub name: String,
    pub path: PathBuf,
    pub checksum: String,
    up_statements: Vec<String>,
    down_statements: Vec<String>,
}

impl DiscoveredMigration {
    pub fn up(&self) -> &[String] {
        &self.up_statements
    }

    pub fn down(&self) -> &[String] {
        &self.down_statements
    }
}

#[derive(Debug, Clone)]
pub struct AppliedMigration {
    pub name: String,
    pub path: String,
    pub checksum: String,
}

impl fmt::Display for AppliedMigration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {}, {})", self.name, self.path, self.checksum)
    }
}

pub fn find_migration_files(directory: &str) -> Result<Vec<PathBuf>, RumblerError> {
    let dir = Path::new(directory);
    if !dir.is_dir() {
        return Err(RumblerError::Migration(format!(
            "migration directory not found: {directory}"
        )));
    }

    let mut entries: Vec<PathBuf> = fs::read_dir(dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|ext| ext == "sql"))
        .collect();

    entries.sort();
    Ok(entries)
}

pub fn discover(directory: &str) -> Result<Vec<DiscoveredMigration>, RumblerError> {
    let entries: Vec<PathBuf> = find_migration_files(directory)?;

    let mut migrations = Vec::new();
    for path in entries {
        let migration = parse(&path)?;
        migrations.push(migration);
    }

    Ok(migrations)
}

enum Marker {
    Up,
    Down,
}

fn is_marker(line: &str) -> Option<Marker> {
    match line.trim() {
        "-- rumbler up" | "-- rambler up" => Some(Marker::Up),
        "-- rumbler down" | "-- rambler down" => Some(Marker::Down),
        _ => None,
    }
}

fn parse(path: &Path) -> Result<DiscoveredMigration, RumblerError> {
    let name = path
        .file_name()
        .ok_or_else(|| RumblerError::Migration(format!("invalid migration path: {path:?}")))?
        .to_string_lossy()
        .into_owned();

    let contents = fs::read_to_string(path)?;
    let checksum = format!("{:x}", Sha256::digest(contents.as_bytes()));

    let (up_statements, down_statements) = migration_statements(contents);

    Ok(DiscoveredMigration {
        name,
        path: path.to_path_buf(),
        checksum,
        up_statements,
        down_statements,
    })
}

fn migration_statements(contents: String) -> (Vec<String>, Vec<String>) {
    let mut up_statements = Vec::new();
    let mut down_statements_raw = Vec::new();

    let mut current_section_marker = None;
    let mut current_section_buffer = String::new();
    for line in contents.lines() {
        if let Some(marker) = is_marker(line) {
            // Flush buffer into the current section
            let trimmed = current_section_buffer.trim().to_string();
            if !trimmed.is_empty() {
                match current_section_marker {
                    Some(Marker::Up) => up_statements.push(trimmed),
                    Some(Marker::Down) => down_statements_raw.push(trimmed),
                    _ => {}
                }
            }
            current_section_buffer.clear();
            current_section_marker = Some(marker);
        } else if current_section_marker.is_some() {
            // push the current sql line into the buffer, preserving newlines
            if !current_section_buffer.is_empty() {
                current_section_buffer.push('\n');
            }
            current_section_buffer.push_str(line.trim());
        }
    }

    // Flush final buffer
    let trimmed = current_section_buffer.trim().to_string();
    match current_section_marker {
        Some(Marker::Up) => up_statements.push(trimmed),
        Some(Marker::Down) => down_statements_raw.push(trimmed),
        _ => {}
    }

    // Down statements are returned in reverse order
    down_statements_raw.reverse();
    (up_statements, down_statements_raw)
}

pub fn check_consistency(
    discovered_migrations: &[DiscoveredMigration],
    applied_migrations: &[AppliedMigration],
) -> Result<Vec<DiscoveredMigration>, RumblerError> {
    let mut pending = Vec::new();

    for item in discovered_migrations.iter().zip_longest(applied_migrations) {
        match item {
            EitherOrBoth::Both(discovered, applied) => {
                if discovered.name != applied.name {
                    return Err(RumblerError::OutOfOrder(discovered.name.clone()));
                }
                if discovered.checksum != applied.checksum {
                    return Err(RumblerError::InconsistentChecksum {
                        migration: applied.clone(),
                        expected: discovered.checksum.clone(),
                        found: applied.checksum.clone(),
                    });
                }
            }
            EitherOrBoth::Left(discovered) => {
                pending.push(discovered.clone());
            }
            EitherOrBoth::Right(applied) => {
                // More applied migrations than discovered ones means some applied migration files are missing
                return Err(RumblerError::MissingMigration(applied.clone()));
            }
        }
    }

    Ok(pending)
}

#[cfg(test)]
mod discover_tests {
    use super::*;
    use sealed_test::prelude::*;
    use std::io::Write;

    fn write_migration(dir: &Path, name: &str, content: &str) {
        let path = dir.join(name);
        let mut f = fs::File::create(path).unwrap();
        f.write_all(content.as_bytes()).unwrap();
    }

    #[sealed_test]
    fn test_discover_not_a_directory() {
        let err = discover("nonexistant/directory").unwrap_err();
        assert_eq!(
            err.to_string(),
            "migration error: migration directory not found: nonexistant/directory"
        );
    }

    #[sealed_test]
    fn test_discover_empty_directory() {
        let dir = tempfile::tempdir().unwrap();

        let migrations = discover(dir.path().to_str().unwrap()).unwrap();
        assert_eq!(migrations.len(), 0);
    }

    #[sealed_test]
    fn test_discover_parse_rumbler_markers() {
        let dir = tempfile::tempdir().unwrap();
        write_migration(
            dir.path(),
            "001_create_users.sql",
            r"
            -- rumbler up
            CREATE TABLE users (id SERIAL PRIMARY KEY);
            -- rumbler down
            DROP TABLE users;
            ",
        );

        let migrations = discover(dir.path().to_str().unwrap()).unwrap();
        assert_eq!(migrations.len(), 1);
        assert_eq!(migrations[0].name, "001_create_users.sql");
        assert_eq!(
            migrations[0].up(),
            &["CREATE TABLE users (id SERIAL PRIMARY KEY);"]
        );
        assert_eq!(migrations[0].down(), &["DROP TABLE users;"]);
    }

    #[sealed_test]
    fn test_discover_parse_rambler_markers() {
        let dir = tempfile::tempdir().unwrap();
        write_migration(
            dir.path(),
            "001_create_users.sql",
            r"
            -- rambler up
            CREATE TABLE users (id SERIAL PRIMARY KEY);
            -- rambler down
            DROP TABLE users;
            ",
        );

        let migrations = discover(dir.path().to_str().unwrap()).unwrap();
        assert_eq!(migrations.len(), 1);
        assert_eq!(
            migrations[0].up(),
            &["CREATE TABLE users (id SERIAL PRIMARY KEY);"]
        );
        assert_eq!(migrations[0].down(), &["DROP TABLE users;"]);
    }

    #[sealed_test]
    fn test_discover_parse_multiline_sql() {
        let dir = tempfile::tempdir().unwrap();
        write_migration(
            dir.path(),
            "001_create_users.sql",
            r"
            -- rambler up
            CREATE TABLE users (
                id SERIAL PRIMARY KEY,
                name TEXT NOT NULL
            );
            -- rambler down
            DROP TABLE users;
            ",
        );

        let migrations = discover(dir.path().to_str().unwrap()).unwrap();
        assert_eq!(migrations.len(), 1);
        assert_eq!(
            migrations[0].up(),
            &[r"CREATE TABLE users (
id SERIAL PRIMARY KEY,
name TEXT NOT NULL
);"]
        );
        assert_eq!(migrations[0].down(), &["DROP TABLE users;"]);
    }

    #[sealed_test]
    fn test_discover_multiple_sections() {
        let dir = tempfile::tempdir().unwrap();
        write_migration(
            dir.path(),
            "001_multi.sql",
            "
-- rumbler up
CREATE TABLE a (id INT);
-- rumbler up
CREATE TABLE b (id INT);
-- rumbler down
DROP TABLE b;
-- rumbler down
DROP TABLE a;
",
        );

        let migrations = discover(dir.path().to_str().unwrap()).unwrap();
        assert_eq!(migrations.len(), 1);
        assert_eq!(migrations[0].up().len(), 2);
        assert_eq!(migrations[0].up()[0], "CREATE TABLE a (id INT);");
        assert_eq!(migrations[0].up()[1], "CREATE TABLE b (id INT);");
        // Down statements are reversed
        assert_eq!(migrations[0].down().len(), 2);
        assert_eq!(migrations[0].down()[0], "DROP TABLE a;");
        assert_eq!(migrations[0].down()[1], "DROP TABLE b;");
    }

    #[sealed_test]
    fn test_discover_ordering() {
        let dir = tempfile::tempdir().unwrap();
        write_migration(dir.path(), "002_second.sql", "-- rumbler up\nSELECT 2;\n");
        write_migration(dir.path(), "001_first.sql", "-- rumbler up\nSELECT 1;\n");
        write_migration(dir.path(), "003_third.sql", "-- rumbler up\nSELECT 3;\n");

        let migrations = discover(dir.path().to_str().unwrap()).unwrap();
        assert_eq!(migrations.len(), 3);
        assert_eq!(migrations[0].name, "001_first.sql");
        assert_eq!(migrations[1].name, "002_second.sql");
        assert_eq!(migrations[2].name, "003_third.sql");
    }
}

#[cfg(test)]
mod consistency_tests {
    use super::*;
    use sealed_test::prelude::*;

    #[sealed_test]
    fn test_consistency_all_pending() {
        let available = vec![
            discovered_migration_stub("001_a.sql"),
            discovered_migration_stub("002_b.sql"),
        ];
        let applied = vec![];
        let pending = check_consistency(&available, &applied).unwrap();
        assert_eq!(pending, available);
    }

    #[sealed_test]
    fn test_consistency_some_applied() {
        let available = vec![
            discovered_migration_stub("001_a.sql"),
            discovered_migration_stub("002_b.sql"),
            discovered_migration_stub("003_c.sql"),
        ];
        let applied = vec![applied_migration_stub("001_a.sql")];
        let pending = check_consistency(&available, &applied).unwrap();
        assert_eq!(
            pending,
            vec![
                discovered_migration_stub("002_b.sql"),
                discovered_migration_stub("003_c.sql"),
            ]
        );
    }

    #[sealed_test]
    fn test_consistency_out_of_order() {
        let available = vec![
            discovered_migration_stub("001_a.sql"),
            discovered_migration_stub("002_new.sql"),
            discovered_migration_stub("003_c.sql"),
        ];
        let applied = vec![
            applied_migration_stub("001_a.sql"),
            applied_migration_stub("003_c.sql"),
        ];
        let result = check_consistency(&available, &applied);
        assert!(matches!(result, Err(RumblerError::OutOfOrder(_))));
    }

    #[sealed_test]
    fn test_consistency_missing_file() {
        let available = vec![
            discovered_migration_stub("001_a.sql"),
            discovered_migration_stub("002_b.sql"),
        ];
        let applied = vec![
            applied_migration_stub("001_a.sql"),
            applied_migration_stub("002_b.sql"),
            applied_migration_stub("003_c.sql"),
        ];
        let result = check_consistency(&available, &applied);
        assert!(matches!(result, Err(RumblerError::MissingMigration(_))));
    }

    fn discovered_migration_stub(name: &str) -> DiscoveredMigration {
        DiscoveredMigration {
            name: name.to_string(),
            path: PathBuf::from(name),
            checksum: String::new(),
            up_statements: vec![],
            down_statements: vec![],
        }
    }

    fn applied_migration_stub(name: &str) -> AppliedMigration {
        AppliedMigration {
            name: name.to_string(),
            path: name.to_string(),
            checksum: String::new(),
        }
    }
}
