# CI

This repository's CI is a Dagger module written in Rust with this SDK. The same functions
run on a laptop and in CI. A CI system only starts the pinned engine and calls the module:
Buildkite does so on Namespace's agents, and GitHub Actions runs the same calls until
Buildkite replaces it.

## Layout

| Path | Contents |
| --- | --- |
| `dagger.toml` | Dagger workspace: installs the Rust SDK runtime from this repository at a pinned commit and registers the `ci` module |
| `.dagger/modules/ci/` | The module, a Cargo project whose generated files are committed |
| `.dagger/modules/ci/acceptance/` | The acceptance's fixtures: the module it authors, its plain client, and the sample service that the README's module builds |
| `.dagger/modules/ci/consumer/` | The application that `package` and `published` build against a release's crates, outside this repository's workspace |
| `.dagger/engine.sh` | Downloads and verifies the pinned CLI, starts the pinned engine, and prints the environment |
| `.buildkite/pipeline.yml` | Runs `gates` for every pushed commit and `acceptance` for commits that touch the runtime, the crates, the target, the README, or CI, on Namespace's agents |
| `.github/workflows/gates.yml` | Runs `dagger call ci gates` for every pushed commit; `main` requires it |
| `.github/workflows/acceptance.yml` | Runs `dagger call ci acceptance` for pushes that touch the runtime, the crates, the target, the README, or CI |

The module never sees the CI system. Its constructor takes the repository, which defaults
to the workspace root without `.git` and build output; pass
`dagger call ci --source=<dir> …` to check another tree.

## Functions

| Call | What it does |
| --- | --- |
| `gates` | Runs the engine-free acceptance of [maintaining.md](maintaining.md), section 5: `cargo fmt`, `check`, `clippy -D warnings`, `doc -D warnings`, the test suite with all and with no default features, `cargo deny check`, and the generated-output check, alongside the runtime's and `rust-client-dev`'s Go tests |
| `acceptance --commit=<commit>` | Runs the [stock-engine acceptance](engine-integration.md#stock-engine-acceptance) for a commit pushed to this repository: installs the runtime from it, drives module and client flows, and runs the README module's `test` and `build` |
| `package --commit=<commit>` | Packages the crates a release publishes from a pushed commit, verifies each by building it from its archive, then builds and runs an application against the archives; returns the archives, their `SHA256SUMS`, and each crate's file list |
| `published --sums=<file>` | Proves a release on crates.io: each archive crates.io serves matches the `SHA256SUMS` that `package` returned, and an application that depends on the version alone resolves both crates from crates.io, builds, and runs |

`acceptance` checks the commit as pushed. The runtime, the crates, the target, and the
README come from GitHub at that commit; only the module and its fixtures come from the
checkout. It runs on the engine that calls it, which must be of the release the commit
targets, and starts each `dagger` command in a container as a new session of that engine,
so it needs no engine of its own. The README module's `publish` needs a registry, so it is
checked by hand.

`package` and `published` bracket a release, and no CI system runs them: a maintainer
calls them by hand around publishing, as [maintaining.md](maintaining.md), section 6,
describes. `package` refuses a version crates.io already has, a warning from
`cargo package` other than its notices about tests the archives leave out, an archive over
crates.io's 10 MiB limit, and an archive that does not record the clean commit. Packaging
is reproducible, so `cargo publish` from that commit uploads the same bytes, which
`published` then checks against crates.io.

## Local use

Start the engine and put the matching CLI on `PATH`, then call the module:

```console
eval "$(.dagger/engine.sh)"
dagger call ci gates
```

The first run downloads the CLI into `~/.cache/dagger-rust-sdk/dagger/`, checks it against
its pinned SHA-256, and starts the `dagger-rust-sdk-engine-v1.0.0-beta.18` container from
the image pinned by digest. Later runs reuse the container. The engine keeps its state in
the `dagger-rust-sdk-engine-v1.0.0-beta.18-state` volume and collects its cache above
30 GB, because the disk it sees, such as a container VM's on a Mac, can be far larger than
the space really free.

`.dagger/engine.sh stop` stops the engine and removes its container, keeping the volume.
Stop it this way rather than with `docker rm -f`: an engine that is killed rather than
stopped discards its whole cache when it next starts.

The acceptance installs the runtime from GitHub, so push the commit first:

```console
dagger call ci acceptance --commit "$(git rev-parse HEAD)"
```

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
  `.dagger/modules/ci/dagger-module.toml` with it;
- `DAGGER_VERSION` and the CLI checksums that the acceptance uses, in
  `.dagger/modules/ci/src/lib.rs`.

## Buildkite on Namespace

The [Buildkite pipeline](https://buildkite.com/tokeira/dagger-rust-sdk) has one configured
step, which uploads `.buildkite/pipeline.yml` from the commit being built. That file's steps
run the workflows' commands on Namespace's agents, with 8 vCPUs and 32 GB of memory each.

Each step keeps its engine's state on a Namespace cache volume of its own by setting
`DAGGER_RUST_SDK_ENGINE_STATE`, so the engine's build cache, Cargo's included, outlives the
job. Namespace saves a volume only when its job passes, and the step stops the engine before
the job ends, so the saved state comes from a clean shutdown. A volume must stay larger
than the engine's 30 GB cache cap: a cache that outgrows its volume starts empty on the next
job.

Each step posts its own commit status, `buildkite/gates` or `buildkite/acceptance`, and the
pipeline's own GitHub status is off. `acceptance` runs when the commit changes one of its
paths since its merge base with `main`, so a build of `main` itself skips it: the commit
already passed on its branch before `main` was fast-forwarded to it.

The agents reach the host's Docker daemon, which starts the engine's privileged container,
so the agents need no privileges of their own.
