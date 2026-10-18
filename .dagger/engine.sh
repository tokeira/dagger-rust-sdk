#!/usr/bin/env bash
# Provisions the stock Dagger engine and CLI that the ci module runs on.
#
#   eval "$(.dagger/engine.sh)"      start the engine and put the matching CLI on PATH
#   eval "$(.dagger/engine.sh cli)"  put the CLI on PATH without starting an engine
#   .dagger/engine.sh stop           remove the engine container and keep its cache volume
#
# Both are pinned to the release codegen/target.json targets, because the SDK talks only to
# an engine of exactly that release. The CLI archive is checked against its SHA-256 before
# it is unpacked, and the engine image is pulled by digest. Only the export lines go to
# standard output, so the result can be evaluated.
set -euo pipefail

version=1.0.0-beta.18
image=registry.dagger.io/engine:v$version@sha256:bdba5a56b1286e38244ad060c21fb4a5db6d30abc0077d79677f7450c737ae48
# The CLI removes other releases' `dagger-engine-*` containers, so this name avoids the prefix.
container=dagger-rust-sdk-engine-v$version
cache=${XDG_CACHE_HOME:-$HOME/.cache}/dagger-rust-sdk/dagger/$version

case "$(uname -s)/$(uname -m)" in
Darwin/arm64) platform=darwin_arm64 sha256=b132521ba660b63da0d3e2b6b0e160313e716684a6bde86cf766d1efd434b7dd ;;
Linux/x86_64) platform=linux_amd64 sha256=5326f48825b64008705290cb2a8109c28e4a88d341ef765be8234860acf32fdb ;;
Linux/aarch64) platform=linux_arm64 sha256=abcc2e70154498e7933758e7d81b3be79f7a5699d30b0d04ea33ed70d6edf827 ;;
*)
	echo "engine.sh: no pinned Dagger CLI for $(uname -s)/$(uname -m)" >&2
	exit 1
	;;
esac

install_cli() {
	[ -x "$cache/dagger" ] && return
	local archive=$cache/dagger_v${version}_$platform.tar.gz
	mkdir -p "$cache"
	curl -fsSLo "$archive" "https://dl.dagger.io/dagger/releases/$version/dagger_v${version}_$platform.tar.gz"
	echo "$sha256  $archive" | shasum -a 256 -c - >&2
	tar -xzf "$archive" -C "$cache" dagger
}

start_engine() {
	if [ "$(docker inspect -f '{{.State.Running}} {{.Config.Image}}' "$container" 2>/dev/null)" != "true $image" ]; then
		docker rm -f "$container" >/dev/null 2>&1 || true
		docker run -d --privileged --name "$container" -v "$container-state:/var/lib/dagger" "$image" >/dev/null
	fi
}

case ${1:-start} in
start)
	install_cli
	start_engine
	# shellcheck disable=SC2016 # $PATH expands where the output is evaluated
	printf 'export PATH=%q:"$PATH"\n' "$cache"
	printf 'export DAGGER_ENGINE=%q DAGGER_NO_NAG=1\n' "container+docker://$container"
	;;
cli)
	install_cli
	# shellcheck disable=SC2016 # $PATH expands where the output is evaluated
	printf 'export PATH=%q:"$PATH"\n' "$cache"
	;;
stop)
	docker rm -f "$container" >/dev/null
	;;
*)
	echo "usage: engine.sh [start|cli|stop]" >&2
	exit 2
	;;
esac
