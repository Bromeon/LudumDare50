#!/bin/bash
GODOT_BIN="../Godot_v3.4.4-stable_x11.64"
cargo build --manifest-path Rust/Cargo.toml && \
    $GODOT_BIN --path Godot

