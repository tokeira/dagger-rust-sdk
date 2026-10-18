# Rust SDK engine integration

This document is the maintainer contract for the private Rust compiler, engine adapter,
runtime image, pinned SDK content, and stock-engine verification. Application lifecycle
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

`.dagger/modules/rust-client-dev/internal/enginefree` keeps this source-level audit
executable. If an owning build contract changes, update the test and review the Rust
decision rather than allowing silent drift.

## Verification boundaries

Use three deliberately separate boundaries:

1. While iterating, run the narrowest Cargo package, fixture, or direct Go ABI test.
2. Before a pull request, run the canonical Cargo and Go checks without a Dagger
   engine.
3. For runtime changes and release readiness, run the
   [stock-engine acceptance](#stock-engine-acceptance). It installs the runtime from a
   pushed commit on a stock engine of the target release and drives module and client
   flows against it.

Focused engine cases remain diagnostic tools for engine-only behavior. They are not a
second release gate and do not replace the acceptance run.

## Engine-free development workflow

From the repository root, run the canonical Rust commands in `AGENTS.md`. Useful focused suites
include:

```console
cargo test -p dagger-codegen --test engine_operations --test visible_schema_properties --locked
cargo test -p dagger-sdk-engine --locked
```

The direct Go boundaries are also engine-free:

```console
cd .dagger/modules/rust-client-dev
go test . ./internal/enginefixture ./internal/enginefree -count=1

cd ../../../runtime
go vet ./...
go test ./internal/metadata -count=1
DAGGER_SESSION_PORT=1 DAGGER_SESSION_TOKEN=placeholder go test . -count=1
```

The runtime's generated bindings read the session when the package loads, so its root
package needs placeholder session values; the tests never connect.

Generation is not part of the continuous local loop. Refresh bindings only when their
owning module API or schema changes, inspect that scoped diff once, and return to direct
checks.

## Pinned SDK content

Installed from a Git reference, the runtime builds its SDK content from the commit that
`runtime/assets/sdk-dependency.json` names. It compiles the engine tool and rustfmt from
that commit's crates in a digest-pinned image, adds the runtime sources, `LICENSE`, and
`NOTICE`, and seals the result with `dagger-rust-engine package-content`. Modules it
initializes depend on the same commit:

```toml
dagger-sdk = { package = "tokeira-dagger-sdk", git = "https://github.com/tokeira/dagger-rust-sdk", rev = "<commit>" }
```

A commit cannot record its own hash, so moving the pin takes two commits in one pull
request: the content change, then a commit that points `sdk-dependency.json` at it,
together with any runtime change that depends on the new content, such as the package
name `runtime/internal/metadata/engine.go` accepts in the sealed descriptor. Merge
without squashing or rebasing, so both commits keep their hashes on `main`; a rewritten
content commit would leave the pin naming a commit that `main` does not contain.
Re-pin whenever the content that modules build against should change, including after
every target refresh.

## Stock-engine acceptance

`dagger call ci acceptance --commit <commit>` performs every step below for a commit
pushed to this repository and checks each result, except the README module's `publish`
([ci.md](ci.md)). It runs on the engine that calls the ci module, which
`.dagger/engine.sh` starts as a stock engine of the target release. The `acceptance`
workflow runs it for each push that touches the runtime, the crates, the target, the
README, or CI.

By hand, run the acceptance with the target release's stock CLI, downloaded from
`https://dl.dagger.io/dagger/releases/<version>/` and verified against that release's
`checksums.txt` (`eval "$(.dagger/engine.sh cli)"` does both), and a dedicated engine
container. Name the container so it cannot collide with the `dagger-engine-*` containers
the CLI manages, and keep its state on a volume:

```console
docker run -d --privileged --name rust-sdk-acceptance \
  -v rust-sdk-acceptance-state:/var/lib/dagger registry.dagger.io/engine:v1.0.0-beta.18
export DAGGER_ENGINE=container+docker://rust-sdk-acceptance
dagger version
```

`dagger version` must report the target engine and revision. Then, with the runtime
pushed at `<commit>`, work in a new Git repository, because beta.18's `Workspace.export`
needs a Git workspace:

1. `dagger install github.com/tokeira/dagger-rust-sdk/runtime@<commit>` installs the
   runtime as an SDK module.
2. `dagger module init rust --name gate` scaffolds the module, writes its
   `dagger-module.toml`, and seals the SDK content. The module's `Cargo.toml` depends on
   the pinned commit.
3. Author exports that take and return Core types. Loading the module before
   regeneration fails with a diagnostic that names `dagger generate`.
4. `dagger generate` regenerates. A second run changes nothing, and the module's
   `.gitattributes` stays scoped to the module.
5. `dagger call gate <function>` loads the module and calls each export.
6. A plain client that depends on the pinned commit runs under `dagger run`, makes Void
   calls, and exports an image.
7. Adding a Rust client scope with `dagger module client add` is refused before
   anything is written.

The README's `ci` module is a fuller module check: `test` against PostgreSQL, `build`,
and `publish` to a registry that only the engine can reach
(`docker run -d --network container:rust-sdk-acceptance registry:2`, then
`--address=localhost:5000/…`). The ci module runs its `test` and `build` unchanged, from
the block marked for it; check `publish` by hand. Remove the engine container, its
volume, the registry, and any exported images afterwards.

## Immutable generated dependency

Generated Cargo projects accept either an exact registry version or a credential-free
HTTPS Git repository plus an exact lowercase 40-character revision. A path, branch,
tag-only coordinate, default branch, credential-bearing URL, query, fragment, malformed
revision, or unreachable commit fails before Cargo runs.

For Git-backed development, the selected revision must already be reachable from the
repository and contain byte-identical public SDK build inputs. Confirm the local and
remote revision, then pass the same repository and revision to every focused engine
content invocation. The current workspace identity and the generated project's
dependency identity are separate safety contracts; never substitute a mutable ref to
make resolution pass.

This immutable dependency mechanism protects generated projects. It does not publish a
crate, create a tag, or authorize a hosted release.

## Focused engine regressions

Run a focused case only when its boundary cannot be represented by the direct harness.
The cases are the `engine-unit` and `engine-content` entry points of
`.dagger/modules/rust-client-dev`, which still build the fork engine through an
`engine-dev` module this repository does not contain. They cannot run here until they
move onto a stock engine.

The closed case inventory covers built-in resolution; empty and existing project
initialization; no-generate behavior; library, module, client, and entrypoint
operations; checked and legacy runtime paths; lock/toolchain rejection; path ownership;
and credential-safe diagnostics. Multiple requested cases share one content object.

Before a focused run, describe the initial files, each mutation, schema source,
generation mode, runtime precondition, and expected owned paths. Assert those semantics
in `internal/enginefree` first. A multi-minute engine build is the final observation,
not the fixture-design loop.

The previously observed long-running module-query failure—
`Post "http://dagger/query": unexpected EOF` after roughly 30 seconds—is a separate,
unverified engine regression. Ordinary external-consumer verification does not exercise
that duration boundary, and Rust SDK release readiness does not claim it is fixed.

## Local triage and generated ownership

Start with the owning direct Cargo test and deterministic fixture. Repair generated
ownership through scoped generation; never delete caller-authored Cargo, source, VCS,
or workspace files. Missing or stale lockfiles are repaired by generation, never by an
unlocked runtime build.

Operation manifests, not filenames or Go symbols, own generated files. The runtime
manifest is finalized only after the stripped binary exists, so input validation cannot
fabricate its digest. Inspect the final image for the installed binary and manifest and
confirm that Cargo homes, caches, source, credentials, and builder state are absent.

When the development module's public API changes, preview and apply only that module's
generated Go bindings. Repository-wide generation is not part of an ordinary Rust
engine-integration checkpoint.
