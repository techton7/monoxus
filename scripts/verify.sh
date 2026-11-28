#!/usr/bin/env bash

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
LOCK_DIR="/tmp/monoxus-cargo.lockdir"

cd "$REPO_ROOT"

acquire_lock() {
    while ! mkdir "$LOCK_DIR" 2>/dev/null; do
        if [[ -r "$LOCK_DIR/pid" ]]; then
            old_pid="$(cat "$LOCK_DIR/pid" 2>/dev/null || true)"
            if [[ "$old_pid" =~ ^[0-9]+$ ]] && kill -0 "$old_pid" 2>/dev/null; then
                sleep 1
                continue
            fi
            if [[ ! "$old_pid" =~ ^[0-9]+$ ]]; then
                sleep 1
                continue
            fi
        else
            sleep 1
            continue
        fi
        rm -f "$LOCK_DIR/pid"
        rmdir "$LOCK_DIR" 2>/dev/null || true
    done
    printf '%s\n' "$$" > "$LOCK_DIR/pid"
}

release_lock() {
    if [[ -r "$LOCK_DIR/pid" ]] &&
        [[ "$(cat "$LOCK_DIR/pid" 2>/dev/null || true)" == "$$" ]]; then
        rm -f "$LOCK_DIR/pid"
        rmdir "$LOCK_DIR" 2>/dev/null || true
    fi
}

acquire_lock
trap release_lock EXIT INT TERM

if command -v sccache >/dev/null 2>&1; then
    export RUSTC_WRAPPER
    RUSTC_WRAPPER="$(command -v sccache)"
    export SCCACHE_CACHE_SIZE="${SCCACHE_CACHE_SIZE:-20G}"
fi

skip_checks=false
run_site=false
while [[ $# -gt 0 ]]; do
    case "$1" in
        --skip-checks|--fast)
            skip_checks=true
            shift
            ;;
        --site)
            run_site=true
            shift
            ;;
        *)
            break
            ;;
    esac
done

if [[ "$skip_checks" == false ]]; then
    cargo check --lib
    cargo check --target wasm32-unknown-unknown
fi

cargo nextest run --no-default-features --features native "$@"

if [[ "$run_site" == true ]]; then
    cargo test --manifest-path "$REPO_ROOT/site/Cargo.toml"
fi
