#!/usr/bin/env bash

set -e

./populate-test-icals.sh

cargo run --release --example generate_random > private-test-icals/generated.ical

# also run tests, clippy and fmt

cargo test --all --locked

cargo clippy

cargo fmt --all -- --check