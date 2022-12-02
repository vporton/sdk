#!/usr/bin/env bats

load ../utils/_

setup() {
    standard_setup

    REPO_ROOT="${BATS_TEST_DIRNAME}/../../"
    cp -R "$REPO_ROOT/src/canisters/telemetry" .
    cd telemetry || exit 1
}

teardown() {
    dfx_stop

    standard_teardown
}

timestamp() {
    d="$1"
    h="${2:-0}"
    m="${3:-0}"

    # All timestamps will be relative to the beginning of this day. The specific day doesn't matter.
    DAY0_START="1668556800000000000" # 2022-11-16T00:00Z

    ts=$(( DAY0_START + ( d * 86400 + h * 3600 + m * 60 ) * 1000000000 ))
    echo "$ts"
}

setTime() { # setTime DAY <HOUR <MINUTE>>
    ts=$(timestamp "$@")
    dfx canister call telemetry_backend testSetTime "($ts)"
}

reportCommandResult() {
    dfxVersion="$1"
    network="$2"
    platform="$3"
    command="$4"
    result="$5"

    dfx canister call telemetry_backend reportCommandResult '(record { dfxVersion = "'"$dfxVersion"'"; network=variant {'"$network"'}; platform=variant {'"$platform"'}; command=variant {'"$command"'}; success='"$result"' })'
}

reportCommandSuccess() {
    reportCommandResult "$@" true
}

reportCommandFailure() {
    reportCommandResult "$@" false
}

getCommandResultReportingPeriods() {
    dfx canister call telemetry_backend getCommandResultReportingPeriods
}

getInvocationDetailsForReportingPeriod() {
    reportingPeriod="$(timestamp "$@")"
    dfx canister call telemetry_backend getInvocationDetailsForReportingPeriod "($reportingPeriod)"
}

getCommandSuccessRates() {
    reportingPeriod="$1"
    dfxVersion="$2"
    network="$3"
    platform="$4"

    dfx canister call telemetry_backend getCommandSuccessRates '('"$reportingPeriod"', record { dfxVersion = "'"$dfxVersion"'"; network=variant {'"$network"'}; platform=variant {'"$platform"'};})'
}

reportDailyUse() {
    dfxVersion="$1"
    platform="$2"

    dfx canister call telemetry_backend reportDailyUse '(record { dfxVersion = "'"$dfxVersion"'"; platform=variant {'"$platform"'};})'
}

getDailyActiveUsersReportingPeriods() {
    dfx canister call telemetry_backend getDailyActiveUsersReportingPeriods
}

getDailyUsers() {
    reportingPeriod="$(timestamp "$@")"
    dfx canister call telemetry_backend getDailyUsers "($reportingPeriod)"
}

reportMonthlyUse() {
    dfxVersion="$1"
    platform="$2"

    dfx canister call telemetry_backend reportMonthlyUse '(record { dfxVersion = "'"$dfxVersion"'"; platform=variant {'"$platform"'};})'
}

getMonthlyActiveUsersReportingPeriods() {
    dfx canister call telemetry_backend getMonthlyActiveUsersReportingPeriods
}

getMonthlyUsers() {
    reportingPeriod="$(timestamp "$@")"
    dfx canister call telemetry_backend getMonthlyUsers "($reportingPeriod)"
}

@test "stores success rates by command" {
    dfx_start
    dfx deploy

    setTime 0 15
    assert_command getCommandResultReportingPeriods
    assert_eq '(vec {})'

    reportCommandSuccess 0.12.1 localProject linux 'dfxStart'

    assert_command getCommandResultReportingPeriods
    assert_eq '(vec { 1_668_556_800_000_000_000 : int })'

    assert_command getInvocationDetailsForReportingPeriod 0
    assert_contains 'dfxVersion = "0.12.1";'
    assert_contains 'network = variant { localProject };'
    assert_contains 'platform = variant { linux };'

    assert_command getCommandSuccessRates "$(timestamp 0)" 0.12.1 localProject linux
    assert_contains 'successRate = 100 : nat;'

    reportCommandFailure 0.12.1 localProject linux 'dfxStart'
    assert_command getCommandSuccessRates "$(timestamp 0)" 0.12.1 localProject linux
    assert_contains 'successRate = 50 : nat;'

    reportCommandSuccess 0.12.1 localProject linux 'dfxStart'
    reportCommandSuccess 0.12.1 localProject linux 'dfxStart'

    # events within 30 days go into the same reporting period
    setTime 20

    reportCommandSuccess 0.12.1 localShared darwin 'dfxCanisterCall'
    reportCommandFailure 0.12.1 localShared darwin 'dfxCanisterCall'

    assert_command getCommandSuccessRates "$(timestamp 0)" 0.12.1 localProject linux
    assert_contains 'successRate = 75 : nat;'

    assert_command getCommandSuccessRates "$(timestamp 0)" 0.12.1 localShared darwin
    assert_contains 'successRate = 50 : nat;'

    # on the start of the 30th day, events go into a new reporting period
    setTime 30
    reportCommandSuccess 0.12.1 localShared darwin 'dfxCanisterCall'
    assert_command getCommandSuccessRates "$(timestamp 0)" 0.12.1 localShared darwin
    assert_contains 'successRate = 50 : nat;'
    assert_command getCommandSuccessRates "$(timestamp 30)" 0.12.1 localShared darwin
    assert_contains 'successRate = 100 : nat;'
}


@test "stores entries by reporting period" {
    dfx_start
    dfx deploy

    # first reporting period
    setTime 0 23 # reporting period starts at the beginning of the day of the first event received
    reportCommandSuccess 0.12.1 localShared linux 'dfxStart'
    reportCommandSuccess 0.12.1 localShared linux 'dfxDeploy'
    reportCommandFailure 0.12.1 localShared linux 'dfxStart'
    reportCommandSuccess 0.12.1 localShared linux 'dfxStop'

    setTime 7 2
    reportCommandFailure 0.12.1 localShared linux 'dfxCanisterCall'
    reportCommandSuccess 0.12.1 localShared linux 'dfxStart'
    reportCommandSuccess 0.12.1 localShared linux 'dfxCanisterCall'

    # second reporting period
    setTime 30
    reportCommandSuccess 0.12.1 localShared darwin 'dfxDeploy'
    setTime 52 4
    reportCommandSuccess 0.12.1 localShared darwin 'dfxCanisterCall'

    # third reporting period
    setTime 61
    reportCommandSuccess 0.12.1 localProject linux 'dfxStart'
    setTime 68
    reportCommandSuccess 0.12.1 localProject linux 'dfxDeploy'
    reportCommandFailure 0.12.1 localProject linux 'dfxDeploy'

    assert_command getCommandResultReportingPeriods
    assert_contains '1_668_556_800_000_000_000'
    assert_contains '1_671_148_800_000_000_000'
    assert_contains '1_673_740_800_000_000_000'

    assert_command getInvocationDetailsForReportingPeriod 0
    assert_eq '(
  vec {
    record {
      dfxVersion = "0.12.1";
      network = variant { localShared };
      platform = variant { linux };
    };
  },
)'

    assert_command getInvocationDetailsForReportingPeriod 30
    assert_eq '(
  vec {
    record {
      dfxVersion = "0.12.1";
      network = variant { localShared };
      platform = variant { darwin };
    };
  },
)'

    assert_command getInvocationDetailsForReportingPeriod 60
    assert_eq '(
  vec {
    record {
      dfxVersion = "0.12.1";
      network = variant { localProject };
      platform = variant { linux };
    };
  },
)'
}

@test 'stores daily users' {
    dfx_start
    dfx deploy

    setTime 0 2
    reportDailyUse 0.12.1 linux
    setTime 0 7
    reportDailyUse 0.12.1 linux
    assert_command getDailyActiveUsersReportingPeriods
    assert_eq "(vec { 1_668_556_800_000_000_000 : int })"

    setTime 2 1
    reportDailyUse 0.12.1 darwin
    setTime 2 8
    reportDailyUse 0.12.1 darwin
    setTime 2 17
    reportDailyUse 0.12.1 darwin
    assert_command getDailyActiveUsersReportingPeriods
    assert_contains '1_668_556_800_000_000_000'
    assert_contains '1_668_729_600_000_000_000'

    setTime 3 0 5
    reportDailyUse 0.12.1 darwin
    setTime 3 2
    reportDailyUse 0.12.1 linux
    setTime 3 7
    reportDailyUse 0.12.1 linux
    assert_command getDailyActiveUsersReportingPeriods
    assert_contains '1_668_556_800_000_000_000'
    assert_contains '1_668_729_600_000_000_000'
    assert_contains '1_668_816_000_000_000_000'

    assert_command getDailyUsers 0
    assert_contains "platform = variant { linux }"
    assert_contains "users = 2 : nat"
    assert_contains 'record { platform = variant { linux }; users = 2 : nat }'


    assert_command getDailyUsers 2
    assert_contains "platform = variant { darwin }"
    assert_contains "users = 3 : nat"
    assert_contains 'record { platform = variant { darwin }; users = 3 : nat }'

    assert_command getDailyUsers 3
    assert_contains 'record { platform = variant { darwin }; users = 1 : nat };'
    assert_contains 'record { platform = variant { linux }; users = 2 : nat };'
}

@test 'stores monthly users' {
    dfx_start
    dfx deploy

    setTime 0 2
    reportMonthlyUse 0.12.1 linux
    setTime 0 7
    reportMonthlyUse 0.12.1 linux
    assert_command getMonthlyActiveUsersReportingPeriods
    assert_eq "(vec { 1_668_556_800_000_000_000 : int })"

    setTime 35 1
    reportMonthlyUse 0.12.1 darwin
    setTime 37 8
    reportMonthlyUse 0.12.1 darwin
    setTime 42  17
    reportMonthlyUse 0.12.1 darwin
    assert_command getMonthlyActiveUsersReportingPeriods
    assert_contains '1_668_556_800_000_000_000'
    assert_contains '1_671_148_800_000_000_000'

    setTime 61 0 5
    reportMonthlyUse 0.12.1 darwin
    setTime 72 2
    reportMonthlyUse 0.12.1 linux
    setTime 83 7
    reportMonthlyUse 0.12.1 linux
    assert_command getMonthlyActiveUsersReportingPeriods
    assert_contains '1_668_556_800_000_000_000'
    assert_contains '1_671_148_800_000_000_000'
    assert_contains '1_673_740_800_000_000_000'

    assert_command getMonthlyUsers 0
    assert_contains "platform = variant { linux }"
    assert_contains "users = 2 : nat"
    assert_contains 'record { platform = variant { linux }; users = 2 : nat }'


    assert_command getMonthlyUsers 30
    assert_contains "platform = variant { darwin }"
    assert_contains "users = 3 : nat"
    assert_contains 'record { platform = variant { darwin }; users = 3 : nat }'

    assert_command getMonthlyUsers 60
    assert_contains 'record { platform = variant { darwin }; users = 1 : nat };'
    assert_contains 'record { platform = variant { linux }; users = 2 : nat };'
}
