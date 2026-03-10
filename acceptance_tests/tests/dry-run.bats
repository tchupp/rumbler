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

# ── Dry-run ───────────────────────────────────────────────

@test "dry-run: apply --all makes no DB changes" {
    run_rambler --dry-run apply -a
    run_rumbler --dry-run apply -a

    assert_equal "" "$(get_tables "$RAMBLER_DB")"
    assert_equal "" "$(get_tables "$RUMBLER_DB")"
}

@test "dry-run: apply --all outputs expected SQL" {
    run_rumbler --dry-run apply -a
    assert_success
    assert_output --partial "CREATE TABLE users"
    assert_output --partial "CREATE TABLE posts"
}

@test "dry-run: reverse --all makes no DB changes after apply" {
    expected="$(cat <<EOF
posts
users
EOF
)"

    run_rambler apply -a
    run_rumbler apply -a

    assert_equal "$(get_tables "$RAMBLER_DB")" "$expected"
    assert_equal "$(get_tables "$RUMBLER_DB")" "$expected"

    run_rambler --dry-run reverse -a
    run_rumbler --dry-run reverse -a

    assert_equal "$(get_tables "$RAMBLER_DB")" "$expected"
    assert_equal "$(get_tables "$RUMBLER_DB")" "$expected"

    assert_dbs_match
}

@test "dry-run: reverse --all outputs expected SQL" {
    run_rumbler apply -a

    run_rumbler --dry-run reverse -a
    assert_success
    assert_output --partial "DROP TABLE posts"
    assert_output --partial "DROP TABLE users"
}
