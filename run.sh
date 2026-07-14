#!/bin/bash
set -e
cd "$(dirname "$0")"

echo "Compiling GSettings schema..."
glib-compile-schemas data/

echo "Building..."
cargo build

echo "Running with GSETTINGS_SCHEMA_DIR=data..."
GSETTINGS_SCHEMA_DIR="$PWD/data" exec cargo run
