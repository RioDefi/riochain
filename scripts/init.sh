#!/usr/bin/env bash

set -e

echo "*** Initializing WASM build environment"

TOOLCHAIN_NIGHTLY_VERSION=nightly-2021-07-20

if [ -z $CI_PROJECT_NAME ] ; then
   rustup install ${TOOLCHAIN_NIGHTLY_VERSION}
   rustup update ${TOOLCHAIN_NIGHTLY_VERSION}
   rustup update stable
fi

rustup target add wasm32-unknown-unknown --toolchain ${TOOLCHAIN_NIGHTLY_VERSION}

# Install wasm-gc. It's useful for stripping slimming down wasm binaries.
command -v wasm-gc || \
	cargo +nightly install --git https://github.com/alexcrichton/wasm-gc --force
