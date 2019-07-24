#!/usr/bin/env sh

set -eux

export PATH="${PATH}:/root/.cargo/bin"

cargo build --release
cp /cache/release/journald-to-cloudwatch /host/
