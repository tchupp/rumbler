#!/usr/bin/env bats

bats_load_library 'test_helpers'

setup_file() {
    write_configs
}

setup() {
    reset_dbs
}

teardown() {
    true
}

@test "import: reads from existing rambler migrations table" {
    psql_cmd -d "$RUMBLER_DB" \
        -c "CREATE TABLE ${RAMBLER_TABLE} (migration VARCHAR(255) NOT NULL);" >/dev/null
    psql_cmd -d "$RUMBLER_DB" \
        -c "INSERT INTO ${RAMBLER_TABLE} VALUES ('001_create_users.sql'), ('002_add_email_to_users.sql');" >/dev/null
    psql_cmd -d "$RUMBLER_DB" -c "
        CREATE TABLE users (
            id SERIAL PRIMARY KEY,
            name VARCHAR(255) NOT NULL,
            created_at TIMESTAMP NOT NULL DEFAULT NOW()
        );
        ALTER TABLE users ADD COLUMN email VARCHAR(255);
        CREATE UNIQUE INDEX idx_users_email ON users (email);
    " >/dev/null

    # Rumbler should import from rambler table, then apply migration 003
    run_rumbler apply

    assert_equal "$(get_rumbler_migrations)" "$(cat <<EOF
001_create_users.sql
002_add_email_to_users.sql
003_create_posts.sql
EOF
)"

    run get_tables "$RUMBLER_DB"
    assert_output --partial "posts"

    # Old rambler table should still exist
    local old=$(psql_cmd -d "$RUMBLER_DB" -c \
        "SELECT COUNT(*) FROM information_schema.tables
         WHERE table_schema = 'public' AND table_name = '${RAMBLER_TABLE}';")
    assert_equal 1 "$old"
}

@test "import: does not duplicate entries on second run" {
    psql_cmd -d "$RUMBLER_DB" \
        -c "CREATE TABLE ${RAMBLER_TABLE} (migration VARCHAR(255) NOT NULL);" >/dev/null
    psql_cmd -d "$RUMBLER_DB" \
        -c "INSERT INTO ${RAMBLER_TABLE} VALUES ('001_create_users.sql');" >/dev/null
    psql_cmd -d "$RUMBLER_DB" -c "
        CREATE TABLE users (
            id SERIAL PRIMARY KEY,
            name VARCHAR(255) NOT NULL,
            created_at TIMESTAMP NOT NULL DEFAULT NOW()
        );
    " >/dev/null

    # First run: import + apply 002
    run_rumbler apply

    # Second run: apply 003 (should NOT re-import)
    run_rumbler apply

    assert_equal "$(get_rumbler_migrations)" "$(cat <<EOF
001_create_users.sql
002_add_email_to_users.sql
003_create_posts.sql
EOF
)"
}
