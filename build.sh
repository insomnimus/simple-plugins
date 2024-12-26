#!/usr/bin/env sh

set -ue

for plugin in simple-{clipper,filter,gain}; do
	cargo run --bin bundler -- bundle "$plugin" "$@"
done
