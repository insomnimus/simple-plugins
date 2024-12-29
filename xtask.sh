#!/usr/bin/env sh

set -ue

cd -- "$(dirname -- "$0")"
exec cargo run --bin bundler "$@"
