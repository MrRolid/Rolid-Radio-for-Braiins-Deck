#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
SDK_REV=b2c945b5f7a8ea3a7d38a8410a1ca64e8b69b9af
if [ ! -d ../bmc-main ]; then git clone https://github.com/BraiinsForge/bmc-main ../bmc-main; git -C ../bmc-main checkout "$SDK_REV"; fi
if [ -d ../bmc-main/.git ]; then actual=$(git -C ../bmc-main rev-parse HEAD); else actual=$(cat ../bmc-main/SDK_REVISION); fi
[ "$actual" = "$SDK_REV" ] || { echo 'SDK revision mismatch'; exit 1; }
rustup target add wasm32-unknown-unknown armv7-unknown-linux-musleabihf
cargo build --locked --release --target wasm32-unknown-unknown
cargo build --locked --manifest-path asset-pack/Cargo.toml
cargo test --locked --manifest-path manifest-check/Cargo.toml
cargo test --locked --manifest-path service/Cargo.toml
CARGO_TARGET_ARMV7_UNKNOWN_LINUX_MUSLEABIHF_LINKER=rust-lld cargo build --locked --release --target armv7-unknown-linux-musleabihf --manifest-path service/Cargo.toml
python3 package.py
python3 tools/release.py
