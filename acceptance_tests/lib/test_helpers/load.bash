#!/usr/bin/env bash

bats_load_library 'bats-support'
bats_load_library 'bats-assert'
bats_load_library 'bats-file'

export PGHOST="${PGHOST:-postgres}"
export PGUSER="${PGUSER:-postgres}"
export PGPASSWORD="${PGPASSWORD:-postgres}"

RAMBLER_DB="rambler_test"
RUMBLER_DB="rumbler_test"
RAMBLER_TABLE="migrations"
RUMBLER_TABLE="rumbler_${RAMBLER_TABLE}"

psql_cmd() {
    psql -h "$PGHOST" -U "$PGUSER" -t -A "$@"
}

reset_dbs() {
    psql_cmd -d postgres -c "DROP DATABASE IF EXISTS ${RAMBLER_DB};" >/dev/null 2>&1
    psql_cmd -d postgres -c "DROP DATABASE IF EXISTS ${RUMBLER_DB};" >/dev/null 2>&1
    psql_cmd -d postgres -c "CREATE DATABASE ${RAMBLER_DB};" >/dev/null 2>&1
    psql_cmd -d postgres -c "CREATE DATABASE ${RUMBLER_DB};" >/dev/null 2>&1
}

write_configs() {
    cat > /tests/rumbler.json <<EOF
{
    "host": "${PGHOST}",
    "port": 5432,
    "user": "${PGUSER}",
    "password": "${PGPASSWORD}",
    "database": "${RUMBLER_DB}",
    "directory": "migrations",
    "table": "${RAMBLER_TABLE}"
}
EOF
    RAMBLER_DB="$RAMBLER_DB" jq '.driver += "postgresql" | .protocol += "tcp" | .database = env.RAMBLER_DB' /tests/rumbler.json > /tests/rambler.json
}

get_tables() {
    local db="$1"
    psql_cmd -d "$db" -c \
        "SELECT table_name FROM information_schema.tables
         WHERE table_schema = 'public'
           AND table_name NOT IN ('${RAMBLER_TABLE}', '${RUMBLER_TABLE}')
         ORDER BY table_name;"
}

get_columns() {
    local db="$1"
    psql_cmd -d "$db" -c \
        "SELECT table_name, column_name, data_type, is_nullable
         FROM information_schema.columns
         WHERE table_schema = 'public'
           AND table_name NOT IN ('${RAMBLER_TABLE}', '${RUMBLER_TABLE}')
         ORDER BY table_name, ordinal_position;"
}

get_indexes() {
    local db="$1"
    psql_cmd -d "$db" -c \
        "SELECT tablename, indexname FROM pg_indexes
         WHERE schemaname = 'public'
           AND tablename NOT IN ('${RAMBLER_TABLE}', '${RUMBLER_TABLE}')
         ORDER BY tablename, indexname;"
}

get_rambler_migrations() {
    local db="${1:-$RAMBLER_DB}"
    psql_cmd -d "$db" -c \
        "SELECT migration FROM ${RAMBLER_TABLE} ORDER BY migration;" 2>/dev/null || echo ""
}

get_rumbler_migrations() {
    local db="${1:-$RUMBLER_DB}"
    psql_cmd -d "$db" -c \
        "SELECT migration FROM ${RUMBLER_TABLE} ORDER BY migration;" 2>/dev/null || echo ""
}

assert_dbs_match() {
    assert_equal "$(get_tables "$RAMBLER_DB")" \
        "$(get_tables "$RUMBLER_DB")"
    assert_equal "$(get_columns "$RAMBLER_DB")" \
        "$(get_columns "$RUMBLER_DB")"
    assert_equal "$(get_indexes "$RAMBLER_DB")" \
        "$(get_indexes "$RUMBLER_DB")"
    # shellcheck disable=SC2119
    assert_equal "$(get_rambler_migrations)" \
        "$(get_rumbler_migrations)"
}

run_rambler() {
    run rambler -c rambler.json --debug "$@" 2>/dev/null
    assert_success
}

run_rumbler() {
    run rumbler -c rumbler.json --debug "$@" 2>/dev/null
    assert_success
}
