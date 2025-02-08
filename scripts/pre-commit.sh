#!/usr/bin/env bash

set -e

pushd crates
    cargo fmt
    cargo clippy
popd
