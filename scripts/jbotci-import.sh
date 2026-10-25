#!/bin/bash
# Reproducible local acquisition and replay for one explicit jbotci revision.
set -euo pipefail
export LC_ALL=C

version=
revision=
raw_sha256=
metadata_sha256=
repo=$(cd "$(dirname "$0")/.." && pwd -P)
raw_name=dictionary-en.json
metadata_name=dictionary-en.metadata.toml
marker_name=complete
marker_value=vlajmaji-jbotci-cache-v1
base_url=https://raw.githubusercontent.com/int19h/jbotci

usage() {
    cat <<'USAGE'
Usage:
  scripts/jbotci-import.sh acquire PINS CACHE_ROOT
  scripts/jbotci-import.sh replay PINS CACHE_ROOT FRESH_RELEASE [FRESH_STORE]
  scripts/jbotci-import.sh check PINS CACHE_ROOT [RELEASE]
  scripts/jbotci-import.sh update NEW_PINS CACHE_ROOT FRESH_RELEASE [FRESH_STORE]
  scripts/jbotci-import.sh help

PINS defaults are in scripts/jbotci-pinned.env. Outputs are create-new only.
USAGE
}

fail() {
    printf 'jbotci pipeline: %s\n' "$*" >&2
    exit 2
}

attempt_failed() {
    local status=$1 operation=$2 leaf=$3 release=${4-} store=${5-}
    printf 'jbotci pipeline: %s failed; cache_leaf=%s' "$operation" "$leaf" >&2
    if [[ -n $release ]]; then
        printf ' release=%s report=%s.report.json' "$release" "$release" >&2
    fi
    if [[ -n $store ]]; then printf ' store=%s' "$store" >&2; fi
    printf '\n' >&2
    return "$status"
}

path_absent() {
    [[ ! -e $1 && ! -L $1 ]]
}

load_pins() {
    local file=$1 line key value
    version= revision= raw_sha256= metadata_sha256=
    local have_version=0 have_revision=0 have_raw=0 have_metadata=0
    [[ -f $file && -r $file ]] || fail "pin manifest must be a readable regular file: $file"
    if LC_ALL=C tr -d '\12\40-\176' < "$file" | grep . >/dev/null 2>&1; then
        fail 'pin manifest must contain only printable ASCII and LF'
    fi
    while IFS= read -r line || [[ -n $line ]]; do
        case $line in
            ''|\#*) continue ;;
            ' '*)
                line=${line#"${line%%[! ]*}"}
                case $line in ''|\#*) continue ;; esac
                fail 'pin assignments must not be indented' ;;
        esac
        case $line in
            *=*) key=${line%%=*}; value=${line#*=} ;;
            *) fail "malformed pin assignment: $line" ;;
        esac
        [[ -n $value && $value != *[!A-Za-z0-9_-]* ]] || fail "invalid pin assignment value for $key"
        case $key in
            version)
                [[ $have_version == 0 ]] || fail 'duplicate version key'
                version=$value; have_version=1 ;;
            revision)
                [[ $have_revision == 0 ]] || fail 'duplicate revision key'
                revision=$value; have_revision=1 ;;
            raw_sha256)
                [[ $have_raw == 0 ]] || fail 'duplicate raw_sha256 key'
                raw_sha256=$value; have_raw=1 ;;
            metadata_sha256)
                [[ $have_metadata == 0 ]] || fail 'duplicate metadata_sha256 key'
                metadata_sha256=$value; have_metadata=1 ;;
            *) fail "unknown pin key: $key" ;;
        esac
    done < "$file"
    [[ $have_version == 1 && $have_revision == 1 && $have_raw == 1 && $have_metadata == 1 ]] || fail 'pin manifest is missing a required key'
    [[ $version == 1 ]] || fail 'pin manifest version must be 1'
    [[ $revision =~ ^[0-9a-f]{40}$ ]] || fail 'revision must be 40 lowercase hexadecimal characters'
    [[ $raw_sha256 =~ ^[0-9a-f]{64}$ ]] || fail 'raw_sha256 must be 64 lowercase hexadecimal characters'
    [[ $metadata_sha256 =~ ^[0-9a-f]{64}$ ]] || fail 'metadata_sha256 must be 64 lowercase hexadecimal characters'
}

leaf_for() {
    printf '%s/%s-%s-%s' "$2" "$revision" "$raw_sha256" "$metadata_sha256"
}

check_cli() {
    if [[ $# == 2 ]]; then
        run_cli jbotci-check "$1/$raw_name" "$1/$metadata_name" \
            --release "$2" --revision "$revision" --raw-sha256 "$raw_sha256" \
            --metadata-sha256 "$metadata_sha256"
    else
        run_cli jbotci-check "$1/$raw_name" "$1/$metadata_name" \
            --revision "$revision" --raw-sha256 "$raw_sha256" \
            --metadata-sha256 "$metadata_sha256"
    fi
}

run_cli() {
    if [[ -n ${VLAJMAJI_BIN:-} ]]; then
        [[ -n $VLAJMAJI_BIN && -f $VLAJMAJI_BIN && -x $VLAJMAJI_BIN ]] || fail 'VLAJMAJI_BIN must name one executable regular file'
        "$VLAJMAJI_BIN" "$@"
    else
        [[ ${CARGO_TARGET_DIR+x} && -n $CARGO_TARGET_DIR ]] || fail 'set CARGO_TARGET_DIR explicitly when VLAJMAJI_BIN is not supplied'
        local cargo_command=${CARGO:-cargo}
        "$cargo_command" +1.98.0 run --locked --offline --manifest-path "$repo/Cargo.toml" \
            -p vlajmaji-cli --bin vlajmaji -- "$@"
    fi
}

cache_state() {
    local leaf=$1 marker=$1/$marker_name
    if path_absent "$leaf"; then
        printf absent
        return
    fi
    [[ -d $leaf && ! -L $leaf ]] || fail "cache leaf is occupied or not a real directory: $leaf"
    [[ -f $marker && ! -L $marker ]] || fail "cache leaf is incomplete and preserved: $leaf"
    [[ -f $leaf/$raw_name && ! -L $leaf/$raw_name ]] || fail "cached raw input is absent or invalid: $leaf"
    [[ -f $leaf/$metadata_name && ! -L $leaf/$metadata_name ]] || fail "cached metadata is absent or invalid: $leaf"
    local actual actual_size expected_size
    actual=$(cat "$marker")
    actual_size=$(wc -c < "$marker" | tr -d '[:space:]')
    expected_size=$((${#marker_value} + 1))
    [[ $actual == "$marker_value" && $actual_size == "$expected_size" ]] || fail "cache marker does not match format version 1: $leaf"
    printf complete
}

cache_root_preflight() {
    local root=$1
    if ! path_absent "$root"; then
        [[ -d $root ]] || fail "cache root must be a directory: $root"
    else
        [[ $allow_create == 1 ]] || fail "cache root does not exist: $root"
        local parent
        parent=$(dirname "$root")
        [[ -d $parent ]] || fail "cache root parent must exist and be a directory: $parent"
    fi
}

download_inputs() {
    local leaf=$1 operation=$2 release=${3-} store=${4-} curl_command=${CURL:-curl} status
    local url=$base_url/$revision/crates/jbotci-dictionary-data/data
    if "$curl_command" --fail --location --silent --show-error \
        --output "$leaf/$raw_name" "$url/$raw_name"; then
        :
    else
        status=$?
        attempt_failed "$status" "$operation" "$leaf" "$release" "$store"
        return $?
    fi
    if "$curl_command" --fail --location --silent --show-error \
        --output "$leaf/$metadata_name" "$url/$metadata_name"; then
        :
    else
        status=$?
        attempt_failed "$status" "$operation" "$leaf" "$release" "$store"
        return $?
    fi
}

write_marker() {
    local leaf=$1
    (set -C; printf '%s\n' "$marker_value" > "$leaf/$marker_name")
}

fresh_outputs_preflight() {
    local release=$1 store=${2-} report=$1.report.json
    path_absent "$release" || fail "release output already exists: $release"
    path_absent "$report" || fail "report output already exists: $report"
    [[ -d $(dirname "$release") ]] || fail "release parent must exist: $(dirname "$release")"
    [[ -d $(dirname "$report") ]] || fail "report parent must exist: $(dirname "$report")"
    if [[ -n $store ]]; then
        path_absent "$store" || fail "store output already exists: $store"
        [[ -d $(dirname "$store") ]] || fail "store parent must exist: $(dirname "$store")"
    fi
}

replay_leaf() {
    local leaf=$1 release=$2 store=${3-} operation=$4 report=$2.report.json status
    if check_cli "$leaf" >/dev/null; then :; else
        status=$?
        attempt_failed "$status" "$operation cache validation" "$leaf" "$release" "$store"
        return $?
    fi
    if (set -C; run_cli jbotci-import "$leaf/$raw_name" "$leaf/$metadata_name" "$release" \
        --revision "$revision" --raw-sha256 "$raw_sha256" \
        --metadata-sha256 "$metadata_sha256" > "$report"); then :; else
        status=$?
        attempt_failed "$status" "$operation import" "$leaf" "$release" "$store"
        return $?
    fi
    if check_cli "$leaf" "$release" >/dev/null; then :; else
        status=$?
        attempt_failed "$status" "$operation release check" "$leaf" "$release" "$store"
        return $?
    fi
    if [[ -n $store ]]; then
        if run_cli dictionary-build "$release" "$store" >&2; then :; else
            status=$?
            attempt_failed "$status" "$operation store build" "$leaf" "$release" "$store"
            return $?
        fi
    fi
    if cat "$report"; then :; else
        status=$?
        attempt_failed "$status" "$operation report read" "$leaf" "$release" "$store"
        return $?
    fi
}

acquire() {
    local pins=$1 root=$2
    load_pins "$pins"
    local leaf state status
    leaf=$(leaf_for "$revision" "$root")
    allow_create=1
    cache_root_preflight "$root"
    state=$(cache_state "$leaf")
    if [[ $state == complete ]]; then
        if check_cli "$leaf"; then :; else
            status=$?
            attempt_failed "$status" 'acquire cache validation' "$leaf"
            return $?
        fi
        return
    fi
    [[ $state == absent ]] || fail "unexpected cache state: $state"
    if path_absent "$root"; then
        if mkdir -p "$root"; then :; else
            status=$?
            attempt_failed "$status" acquire "$leaf"
            return $?
        fi
    fi
    if mkdir "$leaf"; then :; else
        status=$?
        attempt_failed "$status" acquire "$leaf"
        return $?
    fi
    download_inputs "$leaf" acquire || return $?
    if output=$(check_cli "$leaf"); then :; else
        status=$?
        attempt_failed "$status" 'acquire cache validation' "$leaf"
        return $?
    fi
    if write_marker "$leaf"; then :; else
        status=$?
        attempt_failed "$status" 'acquire marker creation' "$leaf"
        return $?
    fi
    printf '%s\n' "$output"
}

replay() {
    local pins=$1 root=$2 release=$3 store=${4-}
    load_pins "$pins"
    fresh_outputs_preflight "$release" "$store"
    local leaf state
    leaf=$(leaf_for "$revision" "$root")
    allow_create=0
    cache_root_preflight "$root"
    state=$(cache_state "$leaf")
    [[ $state == complete ]] || fail "pinned cache leaf is absent: $leaf"
    replay_leaf "$leaf" "$release" "$store" replay
}

check() {
    local pins=$1 root=$2 release=${3-}
    load_pins "$pins"
    local leaf state
    leaf=$(leaf_for "$revision" "$root")
    allow_create=0
    cache_root_preflight "$root"
    state=$(cache_state "$leaf")
    [[ $state == complete ]] || fail "pinned cache leaf is absent: $leaf"
    local status
    if [[ -n $release ]]; then
        if check_cli "$leaf" "$release"; then :; else
            status=$?
            attempt_failed "$status" check "$leaf" "$release"
            return $?
        fi
    else
        if check_cli "$leaf"; then :; else
            status=$?
            attempt_failed "$status" check "$leaf"
            return $?
        fi
    fi
}

update() {
    local pins=$1 root=$2 release=$3 store=${4-}
    load_pins "$pins"
    fresh_outputs_preflight "$release" "$store"
    local leaf state output status
    leaf=$(leaf_for "$revision" "$root")
    allow_create=1
    cache_root_preflight "$root"
    state=$(cache_state "$leaf")
    if [[ $state == complete ]]; then
        replay_leaf "$leaf" "$release" "$store" update
        return
    fi
    [[ $state == absent ]] || fail "unexpected cache state: $state"
    if path_absent "$root"; then
        if mkdir -p "$root"; then :; else
            status=$?
            attempt_failed "$status" update "$leaf" "$release" "$store"
            return $?
        fi
    fi
    if mkdir "$leaf"; then :; else
        status=$?
        attempt_failed "$status" update "$leaf" "$release" "$store"
        return $?
    fi
    download_inputs "$leaf" update "$release" "$store" || return $?
    if (set -C; run_cli jbotci-import "$leaf/$raw_name" "$leaf/$metadata_name" "$release" \
        --revision "$revision" --raw-sha256 "$raw_sha256" \
        --metadata-sha256 "$metadata_sha256" > "$release.report.json"); then :; else
        status=$?
        attempt_failed "$status" 'update import' "$leaf" "$release" "$store"
        return $?
    fi
    if check_cli "$leaf" "$release" >/dev/null; then :; else
        status=$?
        attempt_failed "$status" 'update release check' "$leaf" "$release" "$store"
        return $?
    fi
    if write_marker "$leaf"; then :; else
        status=$?
        attempt_failed "$status" 'update marker creation' "$leaf" "$release" "$store"
        return $?
    fi
    if [[ -n $store ]]; then
        if run_cli dictionary-build "$release" "$store" >&2; then :; else
            status=$?
            attempt_failed "$status" 'update store build' "$leaf" "$release" "$store"
            return $?
        fi
    fi
    if cat "$release.report.json"; then :; else
        status=$?
        attempt_failed "$status" 'update report read' "$leaf" "$release" "$store"
        return $?
    fi
}

main() {
    local operation=${1-}
    [[ -n $operation ]] || { usage >&2; exit 2; }
    shift
    case $operation in
        help|-h|--help) usage ;;
        acquire)
            [[ $# == 2 ]] || fail 'acquire expects PINS CACHE_ROOT'
            acquire "$1" "$2" ;;
        replay)
            [[ $# == 3 || $# == 4 ]] || fail 'replay expects PINS CACHE_ROOT FRESH_RELEASE [FRESH_STORE]'
            replay "$1" "$2" "$3" "${4-}" ;;
        check)
            [[ $# == 2 || $# == 3 ]] || fail 'check expects PINS CACHE_ROOT [RELEASE]'
            check "$1" "$2" "${3-}" ;;
        update)
            [[ $# == 3 || $# == 4 ]] || fail 'update expects NEW_PINS CACHE_ROOT FRESH_RELEASE [FRESH_STORE]'
            update "$1" "$2" "$3" "${4-}" ;;
        *) fail "unknown operation: $operation" ;;
    esac
}

main "$@"
