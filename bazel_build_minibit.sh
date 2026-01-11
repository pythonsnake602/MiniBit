#!/bin/bash

out="$1"

cargo build --bin minibit
cp target/debug/minibit "$out"