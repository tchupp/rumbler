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

@test "multiple up/down sections in a single migration file" {
    run_rambler apply
    run_rambler apply
    run_rumbler apply
    run_rumbler apply
    assert_dbs_match

    run_rambler reverse
    run_rumbler reverse
    assert_dbs_match
}

@test "rumbler tracks path, checksum, and applied_at" {
    run_rumbler apply -a

    assert_equal "$(get_rumbler_migrations)" "$(cat <<EOF
001_create_users.sql
002_add_email_to_users.sql
003_create_posts.sql
EOF
)"

    local bad=$(psql_cmd -d "$RUMBLER_DB" -c \
        "SELECT COUNT(*) FROM ${RUMBLER_TABLE}
         WHERE length(checksum) != 64 OR checksum !~ '^[0-9a-f]+\$';")
    assert_equal 0 "$bad"
}
