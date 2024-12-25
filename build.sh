#!/usr/bin/env sh

set -ue

for plugin in simple-{clipper,gain}; do
	cargo run --bin bundler -- bundle "$plugin" "$@"
done
