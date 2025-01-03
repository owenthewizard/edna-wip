#!/bin/sh

export RUSTFLAGS="-Zlocation-detail=none"

cargo +nightly flamegraph --profile profiling --bench flamegraph --open
