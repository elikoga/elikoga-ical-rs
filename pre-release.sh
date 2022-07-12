#!/usr/bin/env bash

set -e

./populate-test-icals.sh

cargo run --release --example generate_random > private-test-icals/generated.ical

cargo test --all --locked