#!/bin/bash

echo "*** building x86_64 ***"
cargo --locked build --target x86_64-unknown-linux-gnu

echo "*** building aarch64 ***"
cargo --locked build --target aarch64-unknown-linux-gnu

echo "*** building riscv64 ***"
cargo --locked build --target riscv64gc-unknown-linux-gnu
