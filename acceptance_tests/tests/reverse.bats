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

# ── Reverse ───────────────────────────────────────────────

@test "reverse: single migration after apply --all" {
    run_rambler apply -a
    run_rumbler apply -a

    run_rambler reverse
    run_rumbler reverse

    assert_dbs_match
}

@test "reverse --all after apply --all" {
    run_rambler apply -a
    run_rumbler apply -a

    run_rambler reverse -a
    run_rumbler reverse -a

    assert_dbs_match
}

@test "reverse: step-by-step (apply all, then 3x reverse)" {
    run_rambler apply -a
    run_rumbler apply -a

    run_rambler reverse
    run_rambler reverse
    run_rambler reverse

    run_rumbler reverse
    run_rumbler reverse
    run_rumbler reverse

    assert_dbs_match
}

@test "reverse: no-op when nothing is applied" {
    run_rambler apply -a
    run_rumbler apply -a
    run_rambler reverse -a
    run_rumbler reverse -a

    run_rambler reverse
    assert_success
    run_rumbler reverse
    assert_success
    assert_dbs_match
}

@test "reverse: succeeds on fresh database" {
    run_rambler reverse
    assert_success
    cat << EOF | assert_output -
[INFO ] creating migration table: rumbler_migrations
[INFO ] no applied migrations to reverse
EOF
}

@test "reverse: imports from rambler table on first run" {
    # Set up rambler table and actual schema as if rambler had applied migrations
    psql_cmd -d "$RUMBLER_DB" \
        -c "CREATE TABLE ${RAMBLER_TABLE} (migration VARCHAR(255) NOT NULL);" >/dev/null
    psql_cmd -d "$RUMBLER_DB" \
        -c "INSERT INTO ${RAMBLER_TABLE} VALUES ('001_create_users.sql'), ('002_add_email_to_users.sql');" >/dev/null
    psql_cmd -d "$RUMBLER_DB" -c "
        CREATE TABLE users (
            id SERIAL PRIMARY KEY,
            name VARCHAR(255) NOT NULL,
            created_at TIMESTAMP NOT NULL DEFAULT NOW(),
            email VARCHAR(255)
        );
        CREATE UNIQUE INDEX idx_users_email ON users (email);
    " >/dev/null

    # Reverse should import from rambler, then reverse the last migration
    run_rumbler reverse

    assert_equal "$(get_rambler_migrations "$RUMBLER_DB")" "$(cat << EOF
001_create_users.sql
002_add_email_to_users.sql
EOF
)"
    assert_equal "$(get_rumbler_migrations)" "$(cat << EOF
001_create_users.sql
EOF
)"
}
