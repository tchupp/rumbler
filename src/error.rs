use crate::migration::AppliedMigration;
use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum RumblerError {
    #[error("configuration file not found: {0}")]
    ConfigNotFound(PathBuf),

    #[error("configuration error: {0}")]
    ConfigParse(String),

    #[error("unknown environment: {0}")]
    UnknownEnvironment(String),

    #[error("database error: {0}")]
    Database(#[from] postgres::Error),

    #[error("error executing statement: {statement}\n{source}")]
    StatementError {
        statement: String,
        source: postgres::Error,
    },

    #[error("migration error: {0}")]
    Migration(String),

    #[error("out of order migration: {0}")]
    OutOfOrder(String),

    #[error("missing migration file: {0}")]
    MissingMigration(AppliedMigration),

    #[error("inconsistent migration checksum for {migration}: expected {expected}, found {found}")]
    InconsistentChecksum {
        migration: AppliedMigration,
        expected: String,
        found: String,
    },

    #[error("template error: {0}")]
    Template(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}
