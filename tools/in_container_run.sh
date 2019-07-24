#!/usr/bin/env sh

set -eux

sudo chown rust:rust \
      /home/rust/.cargo/git \
      /home/rust/.cargo/registry \
      /home/rust/src/target
cargo build --release
cp /home/rust/src/target/x86_64-unknown-linux-musl/release/journald-to-cloudwatch /host/
