#!/usr/bin/env bash

set -e

pushd crates
    cargo fmt -- --check || exit 1
    cargo clippy -- -D warnings || exit 1
popd
