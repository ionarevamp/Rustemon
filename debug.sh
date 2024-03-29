#!/bin/bash

RUSTFLAGS="-C target-feature=+crt-static" \
cargo build && strip --strip-unneeded target/debug/pokemon_structs && upx --best --lzma target/debug/pokemon_structs && target/debug/pokemon_structs
