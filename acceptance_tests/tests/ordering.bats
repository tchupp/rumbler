#!/usr/bin/env bats

bats_load_library 'test_helpers'

setup_file() {
    write_configs
}

setup() {
    reset_dbs
}

teardown() {
    # Clean up temp files from consistency tests
    rm -f /tests/migrations/001a_inserted.sql
    if [[ -f /tmp/002_add_email_to_users.sql.bak ]]; then
        mv /tmp/002_add_email_to_users.sql.bak /tests/migrations/002_add_email_to_users.sql
    fi
}

@test "consistency: rejects out-of-order migration" {
    cp /tests/migrations/001_create_users.sql /tests/migrations/001a_inserted.sql

    psql_cmd -d "$RAMBLER_DB" \
        -c "CREATE TABLE ${RAMBLER_TABLE} (migration VARCHAR(255) NOT NULL);" >/dev/null
    psql_cmd -d "$RAMBLER_DB" \
        -c "INSERT INTO ${RAMBLER_TABLE} VALUES ('001_create_users.sql'), ('003_create_posts.sql');" >/dev/null

    psql_cmd -d "$RUMBLER_DB" \
        -c "CREATE TABLE ${RUMBLER_TABLE} (migration VARCHAR(255) NOT NULL, path TEXT NOT NULL DEFAULT '', checksum VARCHAR(64) NOT NULL DEFAULT '', applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW());" >/dev/null
    psql_cmd -d "$RUMBLER_DB" \
        -c "INSERT INTO ${RUMBLER_TABLE} (migration) VALUES ('001_create_users.sql'), ('003_create_posts.sql');" >/dev/null

    run_rambler apply
    assert_failure

    run_rumbler apply
    assert_failure
}

@test "consistency: rejects missing migration file" {
    run_rambler apply -a
    run_rumbler apply -a

    mv /tests/migrations/002_add_email_to_users.sql /tmp/002_add_email_to_users.sql.bak

    run_rambler reverse
    assert_failure

    run_rumbler reverse
    assert_failure
}
