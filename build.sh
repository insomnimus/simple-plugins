#!/usr/bin/env sh

set -ue

cd -- "$(dirname -- "$0")"

for plugin in simple-{channel,clipper,filter,gain}; do
	cargo run --bin bundler -- bundle "$plugin" "$@"
done
