#!/usr/bin/env bats

bats_load_library 'test_helpers'

export ROLE_NAME="role_test"

setup_file() {
    write_configs

    # Write a config that includes the role
    jq '.role = env.ROLE_NAME' /tests/rambler.json > /tests/rambler_role.json
    jq '.role = env.ROLE_NAME' /tests/rumbler.json > /tests/rumbler_role.json

    # Create a role for testing
    psql_cmd -d postgres -c "DROP ROLE IF EXISTS ${ROLE_NAME};" >/dev/null 2>&1
    psql_cmd -d postgres -c "CREATE ROLE ${ROLE_NAME};" >/dev/null
    psql_cmd -d postgres -c "GRANT ${ROLE_NAME} TO ${PGUSER};" >/dev/null
}

teardown_file() {
    psql_cmd -d postgres -c "DROP DATABASE IF EXISTS ${RAMBLER_DB};" >/dev/null 2>&1
    psql_cmd -d postgres -c "DROP DATABASE IF EXISTS ${RUMBLER_DB};" >/dev/null 2>&1
    psql_cmd -d postgres -c "DROP ROLE IF EXISTS ${ROLE_NAME};" >/dev/null 2>&1
}

setup() {
    reset_dbs

    # Grant the role permissions on the test database
    psql_cmd -d "$RAMBLER_DB" -c "GRANT ALL ON SCHEMA public TO ${ROLE_NAME};" >/dev/null
    psql_cmd -d "$RUMBLER_DB" -c "GRANT ALL ON SCHEMA public TO ${ROLE_NAME};" >/dev/null
}

# ── Role ──────────────────────────────────────────────────

@test "role: tables are owned by the configured role (rambler)" {
    run rambler -c rambler_role.json apply -a 2>/dev/null
    assert_success

    local owner=$(psql_cmd -d "$RAMBLER_DB" -c \
        "SELECT tableowner FROM pg_tables
         WHERE schemaname = 'public' AND tablename = 'users';")
    assert_equal "$owner" "$ROLE_NAME"
}

@test "role: tables are owned by the configured role (rumbler)" {
    run rumbler -c rumbler_role.json apply -a 2>/dev/null
    assert_success

    local owner=$(psql_cmd -d "$RUMBLER_DB" -c \
        "SELECT tableowner FROM pg_tables
         WHERE schemaname = 'public' AND tablename = 'users';")
    assert_equal "$owner" "$ROLE_NAME"
}

@test "role: apply and reverse work with role set (rambler)" {
    run rambler -c rambler_role.json apply -a 2>/dev/null
    assert_success

    local tables=$(get_tables "$RAMBLER_DB")
    assert_equal "$tables" "$(cat << EOF
posts
users
EOF
)"

    run rambler -c rambler_role.json reverse -a 2>/dev/null
    assert_success

    local tables_after=$(get_tables "$RAMBLER_DB")
    assert_equal "" "$tables_after"
}

@test "role: apply and reverse work with role set (rumbler)" {
    run rumbler -c rumbler_role.json apply -a 2>/dev/null
    assert_success

    local tables=$(get_tables "$RUMBLER_DB")
    assert_equal "$tables" "$(cat << EOF
posts
users
EOF
)"

    run rumbler -c rumbler_role.json reverse -a 2>/dev/null
    assert_success

    local tables_after=$(get_tables "$RUMBLER_DB")
    assert_equal "" "$tables_after"
}

@test "role: fails with nonexistent role (rambler)" {
    jq '.role += "nonexistent_role_xyz"' /tests/rambler.json > /tests/rambler_badrole.json

    run rambler -c rambler_badrole.json apply -a
    assert_failure
}

@test "role: fails with nonexistent role (rumbler)" {
    jq '.role += "nonexistent_role_xyz"' /tests/rumbler.json > /tests/rumbler_badrole.json

    run rumbler -c rumbler_badrole.json apply -a
    assert_failure
}
