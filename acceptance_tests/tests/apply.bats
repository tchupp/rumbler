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

@test "apply: single migration" {
    run_rambler apply
    run_rumbler apply
    assert_dbs_match
}

@test "apply --all" {
    run_rambler apply -a
    run_rumbler apply -a
    assert_dbs_match
}

@test "apply: step-by-step (3x apply without --all)" {
    run_rambler apply
    run_rambler apply
    run_rambler apply

    run_rumbler apply
    run_rumbler apply
    run_rumbler apply

    assert_dbs_match
}

@test "apply: no-op when all migrations already applied" {
    run_rambler apply -a
    run_rumbler apply -a

    run_rambler apply
    assert_success
    run_rumbler apply
    assert_success
    assert_dbs_match
}
