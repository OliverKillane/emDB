#!/usr/bin/env bash

function setup_echo {
    echo "[SETUP] $@"
}

SCRIPT_DIR=$(dirname "$(realpath "$0")")
REPO_DIR=$(realpath "$SCRIPT_DIR/..")

set -ex

pushd $REPO_DIR
    setup_echo " > setup git pre-commit hook"
    ln -sf $SCRIPT_DIR/pre-commit.sh $REPO_DIR/.git/hooks/pre-commit

    setup_echo " > installing additional tooling"
    cargo install mdbook
popd
