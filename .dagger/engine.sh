#!/usr/bin/env bash
# Provisions the stock Dagger engine and CLI that the ci module runs on.
#
#   eval "$(.dagger/engine.sh)"      start the engine and put the matching CLI on PATH
#   eval "$(.dagger/engine.sh cli)"  put the CLI on PATH without starting an engine
#   .dagger/engine.sh stop           stop the engine and remove its container, keeping its state
#
# Both are pinned to the release codegen/target.json targets, because the SDK talks only to
# an engine of exactly that release. The CLI archive is checked against its SHA-256 before
# it is unpacked, and the engine image is pulled by digest. Only the export lines go to
# standard output, so the result can be evaluated.
#
# The engine keeps its state, build cache included, in the `<container>-state` volume. A CI
# system that keeps a directory between jobs can set DAGGER_RUST_SDK_ENGINE_STATE to that
# directory's absolute path, and the state is kept there instead.
set -euo pipefail

version=1.0.0-beta.23
image=registry.dagger.io/engine:v$version@sha256:a5a79c5b3775519be4be61da6507709699c9b31ae7620be0e43d76e5c557871f
# The CLI removes other releases' `dagger-engine-*` containers, so this name avoids the prefix.
container=dagger-rust-sdk-engine-v$version
state=${DAGGER_RUST_SDK_ENGINE_STATE:-$container-state}
# The engine sizes its cache from the disk it sees, which in a virtual machine or on a CI
# cache volume can be far larger than the space really free. It keeps 10 GB and collects
# above 30 GB: reserved, minimum free, and maximum space in MB.
gc=10000,0,30000
cache=${XDG_CACHE_HOME:-$HOME/.cache}/dagger-rust-sdk/dagger/$version

case $state in
/* | "$container-state") ;;
*)
	echo "engine.sh: DAGGER_RUST_SDK_ENGINE_STATE must be an absolute path" >&2
	exit 2
	;;
esac

case "$(uname -s)/$(uname -m)" in
Darwin/arm64) platform=darwin_arm64 sha256=9a4ec08d9927cba3d0746969dfb9a591914846ae079b8a4a37e3fb7120ec4c09 ;;
Linux/x86_64) platform=linux_amd64 sha256=ba2588c2e0444c723e19fd43812cd6dbd868c58fc6cfa1c346e86708b1643d51 ;;
Linux/aarch64) platform=linux_arm64 sha256=9ecf4217ca607d686f56cf71aeb6a21248fa987b679df11abfaa4b2f1d776cd1 ;;
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
	# The label records how the container was started, so changing a setting replaces it.
	local settings="$image $state $gc"
	if [ "$(docker inspect -f '{{.State.Running}} {{index .Config.Labels "dagger-rust-sdk.engine"}}' "$container" 2>/dev/null)" != "true $settings" ]; then
		stop_engine
		docker run -d --privileged --name "$container" --label "dagger-rust-sdk.engine=$settings" \
			-v "$state:/var/lib/dagger" "$image" --oci-worker-gc-keepstorage "$gc" >/dev/null
	fi
}

stop_engine() {
	# Stopping gracefully lets the engine close its databases before its state is reused.
	docker stop -t 30 "$container" >/dev/null 2>&1 || true
	docker rm -f "$container" >/dev/null 2>&1 || true
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
	stop_engine
	;;
*)
	echo "usage: engine.sh [start|cli|stop]" >&2
	exit 2
	;;
esac
