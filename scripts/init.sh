#!/usr/bin/env bash

set -e

echo "*** Initializing WASM build environment"

if [ -z $CI_PROJECT_NAME ]; then
  rustup default nightly && rustup update
  rustup update nightly-2022-02-19
  rustup update stable
fi

rustup target add wasm32-unknown-unknown --toolchain nightly-2022-02-19
rustup default nightly-2022-02-19
