#!/bin/bash

set -eux -o pipefail -o noclobber

# Check unpacked resources
ls offline-vendoring/cargo-vendor/clap-*
ls offline-vendoring/mirrors/advisory-db
ls offline-vendoring/pip/pyyaml*

# Check that rust crates are accessible
(cargo new hello && cd hello && cargo add clap && cargo run)

# Check that rust tools are accessible
cat ~/.cargo/config.toml
(cd hello && cargo nextest --help > /dev/null)

# Check python configuration
pip config debug
pip install pyyaml
