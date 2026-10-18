# CI

This repository's CI is a Dagger module written in Rust with this SDK. The same function
runs on a laptop and in CI. GitHub Actions only starts the pinned engine and calls the
module, so moving to another CI system, such as Buildkite on Namespace agents, changes
only the trigger.

## Layout

| Path | Contents |
| --- | --- |
| `dagger.toml` | Dagger workspace: installs the Rust SDK runtime from this repository at a pinned commit and registers the `ci` module |
| `.dagger/modules/ci/` | The module, a Cargo project whose generated files are committed |
| `.dagger/engine.sh` | Downloads and verifies the pinned CLI, starts the pinned engine, and prints the environment |
| `.github/workflows/gates.yml` | Runs `dagger call ci gates` for every pushed commit; `main` requires it |

The module never sees the CI system. Its constructor takes the repository, which defaults
to the workspace root without `.git` and build output; pass
`dagger call ci --source=<dir> …` to check another tree.

## Functions

| Call | What it does |
| --- | --- |
| `gates` | Runs the engine-free acceptance of [maintaining.md](maintaining.md), section 5: `cargo fmt`, `check`, `clippy -D warnings`, `doc -D warnings`, the test suite with all and with no default features, `cargo deny check`, and the generated-output check, alongside the runtime's and `rust-client-dev`'s Go tests |

The [stock-engine acceptance](engine-integration.md#stock-engine-acceptance) is run by
hand.

## Local use

Start the engine and put the matching CLI on `PATH`, then call the module:

```console
eval "$(.dagger/engine.sh)"
dagger call ci gates
```

The first run downloads the CLI into `~/.cache/dagger-rust-sdk/dagger/`, checks it against
its pinned SHA-256, and starts the `dagger-rust-sdk-engine-v1.0.0-beta.18` container from
the image pinned by digest. Later runs reuse the container. `.dagger/engine.sh stop`
removes the container and keeps its cache volume.

If your global Git configuration rewrites `https://github.com/` to SSH, run `dagger` with
`GIT_CONFIG_GLOBAL=/dev/null`: the engine fetches the runtime over HTTPS and cannot use
your SSH keys.

### Changing the module

Edit `.dagger/modules/ci/src/lib.rs` or its `Cargo.toml`, run `dagger generate`, and commit
the regenerated files with the change. Until you do, loading the module fails with a
diagnostic that names `dagger generate`. `Cargo.lock` is committed, because the engine
builds the module with `--locked`. The root workspace excludes `.dagger`, so Cargo
resolves the module on its own.

## The pinned SDK

The module runs on this repository's own SDK at a pinned commit, not the commit under
test, so a change cannot break the tool that checks it. `dagger.toml` and the module's
`dagger-module.toml` name the runtime at that commit, and the module's `Cargo.toml`
depends on the content commit that runtime pins. To move it, install the runtime at the
new commit with `dagger install`, run `dagger generate`, and commit the result.

## Changing the Dagger release

The SDK talks only to an engine of exactly its target release, so a target refresh
([maintaining.md](maintaining.md), section 3) also moves:

- `version`, the CLI checksums, and the image digest in `.dagger/engine.sh`;
- the module's SDK pin, to a commit of the new target, and `engineVersion` in
  `.dagger/modules/ci/dagger-module.toml` with it.

## Moving to Buildkite on Namespace

A pipeline step runs the same commands as the gates workflow:
`eval "$(.dagger/engine.sh)"`, for example in a pre-command hook, then
`dagger call ci gates`. Agents need a Docker daemon that can run privileged containers.
