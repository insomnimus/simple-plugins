#!/usr/bin/env sh

set -ue

cd -- "$(dirname -- "$0")"

for plugin in depth mono simple-{channel,clipper,filter,gain} sundara-monitors; do
	cargo run --bin bundler -- bundle "$plugin" "$@"
done
