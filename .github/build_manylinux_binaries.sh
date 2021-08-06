#!/bin/bash

# run from within the manylinux docker containers

# exit immediately upon failure, print commands while running
set -e -x

# Install rust inside the manylinux container
#
echo ">>> install rust if it does not exist";
if ! [ -x "$(command -v cargo)" ]; then
  curl https://sh.rustup.rs -sSf | sh -s -- -y --profile minimal
	export PATH="${HOME}/.cargo/bin:${PATH}"
fi

echo ">>> build the binaries";
cargo +stable build --verbose --release --manifest-path=io/rust/Cargo.toml --features=untrusted