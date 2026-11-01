# Rust SDK engine integration

This document is the maintainer contract for the private Rust compiler, engine adapter,
runtime image, SDK content, and stock-engine verification. Application lifecycle
and transport ownership are described in [architecture.md](architecture.md).

## Runtime build audit

The Rust runtime follows the observable build-safety contracts shared by established
Dagger SDKs while keeping implementation policy in Rust.

| Contract | Rust decision |
| --- | --- |
| Generation modes are explicit | Missing introspection selects checked committed generation; present introspection selects private legacy generation. Neither mode silently falls through. |
| Dependency resolution is locked | Runtime verification requires `Cargo.lock`, checks with `cargo metadata --locked`, and builds with `cargo build --locked`. |
| Mutable inputs arrive late | Schema, project, and request inputs are mounted after immutable tools and policy. |
| Build caches are not runtime state | Cargo registry, Git, and target caches remain on the builder. A fresh digest-pinned runtime receives only the stripped binary and runtime manifest. |
| Credentials cannot become artifacts | Control data, runtime identity, cache keys, and generated files contain no credentials. Unsafe Cargo output becomes a bounded typed diagnostic; no socket, secret, Cargo home, or cache enters the runtime. |
| Runtime shape is fixed | The image clears inherited arguments, installs one entrypoint, and uses the engine-owned work directory and platform target. |

`crates/dagger-sdk-engine/tests/runtime_properties.rs` exercises these decisions: the
generation-mode state machine, the locked build on the pinned toolchain and target,
secret-free provenance and redacted output, and a runtime that receives only its binary
and provenance. `runtime/main_test.go` checks that a module's identity leaves out only
what generation writes. If an owning build contract changes, update the tests and review
the Rust decision rather than allowing silent drift.

## Verification boundaries

Use three deliberately separate boundaries:

1. While iterating, run the narrowest Cargo package, fixture, or direct Go ABI test.
2. Before a pull request, run the canonical Cargo and Go checks without a Dagger
   engine.
3. For runtime changes and release readiness, run the
   [stock-engine acceptance](#stock-engine-acceptance). It installs the runtime from a
   pushed commit on a stock engine of the target release and drives module and client
   flows against it.

## Engine-free development workflow

From the repository root, run the canonical Rust commands in `AGENTS.md`. Useful focused suites
include:

```console
cargo test -p dagger-codegen --test engine_operations --test visible_schema_properties --locked
cargo test -p dagger-sdk-engine --locked
```

The runtime's Go tests are also engine-free:

```console
cd runtime
go vet ./...
go test ./internal/metadata -count=1
DAGGER_SESSION_PORT=1 DAGGER_SESSION_TOKEN=placeholder go test . -count=1
```

The runtime's generated bindings read the session when the package loads, so its root
package needs placeholder session values; the tests never connect.

Generation is not part of the continuous local loop. Refresh bindings only when their
owning module API or schema changes, inspect that scoped diff once, and return to direct
checks.

## SDK content

Installed from a Git reference, the runtime builds its SDK content from its own commit.
`runtime/dagger-module.toml` includes the repository files the content needs, so the
engine loads them with the runtime. The runtime compiles the engine tool and rustfmt from
those crates in a digest-pinned image, adds the runtime sources, `LICENSE`, and `NOTICE`,
and seals the result with `dagger-rust-engine package-content`.

The reference the runtime is installed from decides what the modules it initializes
depend on. This repository's tag for the release the content carries, `vX.Y.Z`, selects
that release from crates.io, and an unversioned reference resolves to the latest release
tag:

```toml
dagger-sdk = { package = "tokeira-dagger-sdk", version = "=X.Y.Z" }
```

Any other reference, such as a commit, a branch, or a fork's tag, selects the commit it
resolves to, in the repository it names:

```toml
dagger-sdk = { package = "tokeira-dagger-sdk", git = "https://github.com/tokeira/dagger-rust-sdk", rev = "<commit>" }
```

A module's `Cargo.toml` keeps the dependency it was initialized with, while the engine
resolves the module's runtime reference each time it loads the module. A branch or an
unversioned reference therefore moves the runtime under the module whenever it resolves
to a new commit, for example after `dagger lock update`, and the module must then be
regenerated, with its `Cargo.toml` dependency changed by hand to match. Install the
runtime from a tag or a commit. A runtime whose commit has
`runtime/assets/sdk-dependency.json`, such as 0.1.0's, instead builds its content from the
commit that file names, and its modules depend on that commit.

## Stock-engine acceptance

`dagger call ci acceptance --commit <commit>` performs every step below for a commit
pushed to this repository and checks each result, except the README module's `publish`
([ci.md](ci.md)). It runs on the engine that calls the ci module, which
`.dagger/engine.sh` starts as a stock engine of the target release. CI runs it for each
push that touches the runtime, the crates, the target, the README, or CI, as
[ci.md](ci.md) describes.

By hand, run the acceptance with the target release's stock CLI, downloaded from
`https://dl.dagger.io/dagger/releases/<version>/` and verified against that release's
`checksums.txt` (`eval "$(.dagger/engine.sh cli)"` does both), and a dedicated engine
container. Name the container so it cannot collide with the `dagger-engine-*` containers
the CLI manages, and keep its state on a volume:

```console
docker run -d --privileged --name rust-sdk-acceptance \
  -v rust-sdk-acceptance-state:/var/lib/dagger registry.dagger.io/engine:v1.0.0-beta.23
export DAGGER_ENGINE=container+docker://rust-sdk-acceptance
dagger version
```

`dagger version` must report the target engine and revision. Then, with the runtime
pushed at `<commit>`, work in a new Git repository, because beta.23's `Workspace.export`
needs a Git workspace:

1. `dagger install github.com/tokeira/dagger-rust-sdk/runtime@<commit>` installs the
   runtime as an SDK module.
2. `dagger module init rust --name gate` scaffolds the module, writes its
   `dagger-module.toml`, and seals the SDK content. The module's `Cargo.toml` depends on
   `<commit>`.
3. Author exports that take and return Core types: objects, enums, and scalars, with
   defaults, and an agent function. Loading the module before regeneration fails with a
   diagnostic that names `dagger generate`.
4. `dagger generate` regenerates. A second run changes nothing, and the module's
   `.gitattributes` stays scoped to the module.
5. `dagger call gate <function>` loads the module and calls each export, with and
   without the arguments that have defaults. The engine describes the module with its
   crate documentation. `dagger agent --list` lists the agent function, and composing it
   gives a conversation the module's functions as tools, with no model involved.
6. Plain clients built against the commit's crates run under `dagger run`. One makes
   Void calls and exports an image; the other makes each kind of generated Core call and
   meets each kind of failure the client reports.
7. Adding a Rust client scope with `dagger module client add` is refused before
   anything is written.
8. `dagger module init rust --name existing` adopts an existing crate at
   `.dagger/modules/existing` whose source declares a root object and
   `pub mod dagger_generated;`. Its source is unchanged, its own dependencies stay, its
   `Cargo.toml` gains the SDK dependency, and the module answers calls. A crate without
   a root object is refused, and nothing is written.
9. A module whose generated files are missing fails to load with `GENERATED_MISSING`,
   and one changed since its last generation, such as by a new dependency, with
   `GENERATED_STALE`. One whose `rust-toolchain.toml` names a release older than the
   SDK's minimum fails to load with `TOOLCHAIN_UNSUPPORTED`, even after
   `dagger generate`, and one whose agent function does not return the conversation it
   extends with the engine's reason. One whose `dagger-module.toml` declares an earlier
   release as its `engineVersion` fails `dagger generate` with `TARGET_IDENTITY_INVALID`,
   and the message names the `engineVersion` to declare.
10. `dagger module init` writes nothing outside the workspace. A `--path` that climbs out
    of it is refused, and so is a module path that is a symbolic link to a directory
    outside, with `OUTPUT_SYMLINK_ESCAPE`, which leaves that directory empty. A crate that
    already holds a file where the SDK generates one, such as `src/bin/dagger-module.rs`,
    is refused with `OWNERSHIP_CONFLICT`, and one whose SDK dependency URL carries a
    credential with `SDK_DEPENDENCY_CONFLICT`, without printing the credential. None of
    these refusals writes anything.
11. A module configured by a legacy `dagger.json`, without `dagger-module.toml` or
    generated files, loads and answers `dagger -m <path> call`. The runtime generates
    its code privately, and the workspace is unchanged.
12. A plain client's query outlasts a lost reply to a TCP keepalive probe. The client
    runs under `dagger run` with insecure root capabilities and waits 45 s on an exec.
    Meanwhile it drops the session's acknowledgements to it with iptables for 20 s,
    spanning its first probe. It also fails if no probe went unanswered, because then the
    step proved nothing.

The README's `ci` module is a fuller module check: `test` against PostgreSQL, `build`,
and `publish` to a registry that only the engine can reach
(`docker run -d --network container:rust-sdk-acceptance registry:2`, then
`--address=localhost:5000/…`). The ci module runs its `test` and `build` unchanged, from
the block marked for it, lists and composes its `reviewer` as `dagger agent` does, and runs
`review` with a recorded conversation in which the reviewer approves, so no model is
involved; check `publish` by hand. Remove the engine container, its
volume, the registry, and any exported images afterwards.

## Immutable generated dependency

Generated Cargo projects accept either an exact registry version or a credential-free
HTTPS Git repository plus an exact lowercase 40-character revision. A path, branch,
tag-only coordinate, default branch, credential-bearing URL, query, fragment, malformed
revision, or unreachable commit fails before Cargo runs.

For Git-backed development, the selected revision must already be reachable from the
repository and contain byte-identical public SDK build inputs, so push a commit before
installing the runtime from it. The current workspace identity and the generated
project's dependency identity are separate safety contracts; never substitute a mutable
ref to make resolution pass.

This immutable dependency mechanism protects generated projects. It does not publish a
crate, create a tag, or authorize a hosted release.

## Local triage and generated ownership

Start with the owning direct Cargo test and deterministic fixture. Repair generated
ownership through scoped generation; never delete caller-authored Cargo, source, VCS,
or workspace files. Missing or stale lockfiles are repaired by generation, never by an
unlocked runtime build.

Operation manifests, not filenames or Go symbols, own generated files. The runtime
manifest is finalized only after the stripped binary exists, so input validation cannot
fabricate its digest. Inspect the final image for the installed binary and manifest and
confirm that Cargo homes, caches, source, credentials, and builder state are absent.
