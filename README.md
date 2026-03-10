# rumbler

A simple SQL schema migration tool for PostgreSQL, written in Rust. Drop-in replacement for [rambler](https://github.com/elwinar/rambler) (PostgreSQL only).

## Installation

```bash
cargo install --path .
```

## Usage

```
rumbler [OPTIONS] <COMMAND>

Commands:
  apply     Apply pending migrations
  reverse   Reverse applied migrations

Options:
  -c, --configuration <FILE>   Path to config file [default: rumbler.toml]
  -e, --environment <NAME>     Environment to use
      --debug                  Enable debug logging
      --dry-run                Print SQL without executing
      --no-save                Execute without
```

### Apply migrations

```bash
rumbler apply           # apply the next pending migration
rumbler apply --all     # apply all pending migrations
```

### Reverse migrations

```bash
rumbler reverse         # reverse the last applied migration
rumbler reverse --all   # reverse all applied migrations
```

### Other flags

```bash
rumbler --dry-run apply --all       # print SQL without executing
rumbler apply --no-save             # execute SQL but don't record in tracking table
rumbler apply --migration 003.sql   # apply a specific migration
```

## Configuration

Rumbler reads from `rumbler.toml` (default) or `rambler.json` (fallback).

Rumbler supports all the same options as rambler, but ignores

### TOML

```toml
database = "myapp"
host = "localhost"
port = 5432
user = "postgres"
password = ""
schema = "public"
sslmode = "disable"
directory = "migrations"
table = "rumbler_migrations"

[environments.production]
host = "db.prod.example.com"
password = "secret"
sslmode = "require"
```

### JSON

```json
{
  "database": "myapp",
  "host": "localhost",
  "port": 5432,
  "user": "postgres",
  "password": "",
  "directory": "migrations",
  "table": "rumbler_migrations"
}
```

Select an environment with `-e`:

```bash
rumbler -e production apply --all
```

### Environment variables

All options can be overridden via environment variables. `RUMBLER_` is checked first, with `RAMBLER_` as fallback:

`RUMBLER_DATABASE`, `RUMBLER_HOST`, `RUMBLER_PORT`, `RUMBLER_USER`, `RUMBLER_PASSWORD`, `RUMBLER_SCHEMA`, `RUMBLER_SSLMODE`, `RUMBLER_DIRECTORY`, `RUMBLER_TABLE`

## Migration files

Migrations are `.sql` files in the configured directory, executed in alphabetical order. Each file uses comment markers to separate up and down sections:

```sql
-- rumbler up
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL
);

-- rumbler down
DROP TABLE users;
```

Both `-- rumbler up/down` and `-- rambler up/down` markers are supported. Multiple sections per file are allowed:

```sql
-- rumbler up
ALTER TABLE users ADD COLUMN email VARCHAR(255);

-- rumbler up
CREATE UNIQUE INDEX idx_users_email ON users (email);

-- rumbler down
ALTER TABLE users DROP COLUMN email;

-- rumbler down
DROP INDEX idx_users_email;
```

Down sections are executed in reverse order, so write each down section to match its corresponding up section.

## Migration tracking

Applied migrations are recorded in a table (default: `rumbler_migrations`) with the following schema:

| Column | Type | Description |
|--------|------|-------------|
| `migration` | `VARCHAR(255)` | Migration filename |
| `path` | `TEXT` | File path at time of application |
| `checksum` | `VARCHAR(64)` | SHA-256 hash of the migration file |
| `applied_at` | `TIMESTAMPTZ` | When the migration was applied |

Rumbler creates this table automatically on first run.

### Migrating from rambler

If rumbler detects an existing rambler `migrations` table (the single-column format), it automatically imports those entries into its own tracking table on first run.  
The original `migrations` table is left untouched.  
This allows a seamless transition from rambler to rumbler without re-running migrations.

### Consistency checks

Rumbler enforces consistency — it will error if:
- A new migration file appears between already-applied migrations
- A previously-applied migration file is missing from the filesystem
- A previously-applied migration file has changed (checksum mismatch)

## Acceptance tests

The acceptance tests run both rambler and rumbler against the same migrations and compare database state:

```bash
./dev/acceptance_tests.sh
```

## License

See [LICENSE](LICENSE).
