#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
[ "$(cat ../bmc-main/SDK_REVISION)" = b2c945b5f7a8ea3a7d38a8410a1ca64e8b69b9af ]
[ -f .cargo/config.toml ]
cargo build --offline --locked --release --target wasm32-unknown-unknown
cargo build --offline --locked --manifest-path asset-pack/Cargo.toml
cargo test --offline --locked --manifest-path manifest-check/Cargo.toml
cargo test --offline --locked --manifest-path service/Cargo.toml
cargo build --offline --locked --manifest-path service/Cargo.toml
CARGO_TARGET_ARMV7_UNKNOWN_LINUX_MUSLEABIHF_LINKER=rust-lld cargo build --offline --locked --release --target armv7-unknown-linux-musleabihf --manifest-path service/Cargo.toml
python3 package.py
printf '\nBuild complete. Existing release files preserved. Run tests, then tools/release.py when ready.\n'
