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

# ── --no-save ─────────────────────────────────────────────

@test "apply --no-save: schema changes but no tracking" {
    run_rambler apply --no-save
    run_rumbler apply --no-save

    assert_equal "$(get_tables "$RAMBLER_DB")" \
        "$(get_tables "$RUMBLER_DB")"

    assert_equal "" "$(get_rambler_migrations)"
    assert_equal "" "$(get_rumbler_migrations)"
}
