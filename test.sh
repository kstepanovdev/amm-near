#!/bin/bash
set -e

sh build.sh
cargo test -- --nocapture
