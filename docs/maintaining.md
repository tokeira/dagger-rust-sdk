# Maintaining the Rust SDK for Dagger

This runbook covers generated-client ownership, target refresh, recovery, acceptance,
and releases. It has six ordered steps.

## 1. Confirm ownership and target identity

The core-schema generator owns only:

- `crates/dagger-sdk/src/gen/`;
- `crates/dagger-sdk/tests/core_projection.rs`;
- `crates/dagger-sdk/tests/core_reachability.rs`; and
- `codegen/generated.json`.

Generated Rust files carry a machine-readable source header. The binding manifest binds
the exact path set, byte and semantic digests, checked Dagger revision, and schema
digest. Never edit an owned output or derived manifest by hand. Fix schema validation,
projection, naming, documentation, rendering, or atomic update policy instead.

Before maintenance, confirm `codegen/target.json`, the workspace version, lockfile,
and pinned Rust toolchain agree with the intended target.

## 2. Check and update generated output

Run read-only generation from the repository root:

```console
cargo run -p dagger-bootstrap --bin dagger-rust --locked -- \
  generate --workspace . --check
```

After an intentional generator, schema, or target change, update the complete
owned candidate once:

```console
cargo run -p dagger-bootstrap --bin dagger-rust --locked -- \
  generate --workspace . --update
```

The update is failure-atomic and may replace only manifest-authorized paths. Run check
mode again and inspect `git diff --stat` and `git diff`. The diff must match the
binding manifest; unknown or authored content must remain unchanged.

## 3. Refresh the target deliberately

A target refresh changes an immutable compatibility claim and is separate from ordinary
renderer work:

1. Capture the exact target engine schema as `codegen/schema.json` — do not substitute
   a nearby engine or hand-reserialize it. The document is the client-view schema at
   view `v1.0.0`, produced in-process, with no engine, from an upstream `dagger/dagger`
   checkout at the target tag. From this repository's root, using the Go image that the
   checkout's `go.mod` requires:

   ```console
   UPSTREAM="$(mktemp -d)/dagger"
   git clone --depth 1 --branch v1.0.0-beta.18 https://github.com/dagger/dagger "$UPSTREAM"
   docker run --rm -v "$UPSTREAM:/src:ro" -w /src -e CGO_ENABLED=0 golang:1.26.6 \
     go run ./cmd/introspect introspect --version v1.0.0 > codegen/schema.json
   ```

   `__schemaVersion` inside the captured document must state the view; an empty value
   means the capture ran without `--version` and is not the claimed artifact. For
   `v1.0.0-beta.18` this capture reproduces the committed schema byte for byte.
2. Update `codegen/target.json` with the exact Dagger version, full revision, schema
   digest (the SHA-256 of the captured bytes), Rust SDK version, and Rust toolchain —
   and the matching `APPROVED_*` constants in `crates/dagger-codegen/src/target.rs`,
   which are the reviewed second pin.
3. Hand-carry `crates/dagger-sdk/src/target_generated.rs` to the same identity. It is
   generated *from* the target descriptor but sits outside the bootstrap generator's
   owned artifact set, so the direct update never rewrites it; the pairing test
   `generated_target_matches_checked_repository_metadata` is what refuses a miss.
4. Reset the binding manifest for the new target: artifact provenance is validated
   against the exact target before regeneration, so a refresh replaces
   `codegen/generated.json` with an empty manifest carrying the new
   `target_revision` and `schema_digest`, and removes the owned generated set
   (`crates/dagger-sdk/src/gen/`, `tests/core_projection.rs`,
   `tests/core_reachability.rs`) so generation republishes it whole.
5. Run the direct update, inspect the generated source and compact ownership-manifest
   diff.
6. Sweep the remaining pin surface. The exact target is deliberately asserted in many
   independent places; a refresh visits every one:
   - The release identity everywhere it is stated (see *Release identity*). Sweep
     with a **filterless** `grep -rl` for the outgoing version and revision:
     extension-filtered searches miss lockfiles, and
     `testdata/external-consumer/Cargo.lock` plus `examples/*/Cargo.lock` pin the
     exact crate versions that `--locked` runs refuse to drift past.
   - The reviewed `EXACT_INVENTORY` counts in `schema/validate.rs`, and their
     independent duplicates in `tests/exact_target.rs`.
   - The projection expectations in `tests/projection.rs` (field-strategy counts,
     named-type kinds, catalog binding totals and per-kind counts, directive record
     and application counts) and the coverage totals in `tests/render.rs` and
     `tests/render_properties.rs`. Take the actual values from the projection of the
     captured schema — never invent them — and review each movement in the diff.
   - The module-introspection exclusions in `crates/dagger-codegen/src/engine/visible.rs`
     and their duplicate in `module_visible_schema` of
     `crates/dagger-codegen/tests/support/mod.rs`. They mirror
     `TypesToIgnoreForModuleIntrospection` and `FieldsToIgnoreForModuleIntrospection`
     in `core/moddeps.go` and `TypesHiddenFromModuleSDKs` in `core/env.go` at the target
     revision; a coordinate the engine hides that they omit fails module generation
     with `SCHEMA_CORE_COORDINATE_MISSING`.
   - The checked generated-client fixtures: regenerate through the sanctioned path
     (`DAGGER_UPDATE_GENERATED_CLIENT_FIXTURE=1 cargo test -p dagger-codegen
     --test client_renderer`), then run the workspace formatter.
   - Target-derived engine fixtures and compile-contract goldens outside the generated
     set: update the core-schema digest in
     `crates/dagger-sdk-engine/tests/client_usability_properties.rs` and
     `crates/dagger-sdk-engine/tests/module_generation_properties.rs`, which a version
     sweep does not find, and refresh any generated public-type lists in
     `crates/dagger-sdk/tests/generated-ui/` from the wrapper-free compiler output.
   - The runtime module: `engineVersion` in `runtime/dagger-module.toml`, and its Go
     bindings (`dagger.gen.go`, `internal/dagger/`, `go.mod`, and `go.sum`), which the
     target release's own Go code generator writes. Export them from a stock engine of
     that release through `moduleSource(refString: "runtime")` and its
     `generatedContextDirectory`, then run the runtime's direct Go checks in
     [engine-integration.md](engine-integration.md).
   - The Dagger release pins in `.dagger/engine.sh` and the `ci` module's SDK pin and
     `engineVersion`, as [ci.md](ci.md#changing-the-dagger-release) lists.

Changed or removed schema coordinates fail closed until their generated and
compatibility policies are explicit — a new directive needs a `DirectivePolicy`
registration; inventory growth is admitted deliberately through the counts above.
Never refresh a digest merely to make a check pass.

Finish by running the complete section-5 acceptance gate with **strict exit
propagation** — observe each command's own exit status directly, never through a
pipe filter, which reports the filter's status instead of the command's — and with
an unwrapped `rustc` in a fresh target directory. The compile-contract suites compare
compiler diagnostics against checked goldens; merely clearing `RUSTC_WRAPPER` does
not invalidate artifacts previously produced through a wrapper, so a warm target can
continue to report remapped source paths after repeated reruns. Any residual failure
from the fresh target names a pin this list is missing: fix it *and add it to this
list* in the same change. Then run the
[stock-engine acceptance](engine-integration.md#stock-engine-acceptance) against the new
target.

## 4. Recover or roll back as one unit

A validation, formatting, or cancellation failure leaves the previous generated set
intact. If review rejects a completed update, restore the whole owned set from the
reviewed pre-update commit or repeat in a clean worktree. Restoring selected generated
files can combine different source and manifest identities.

After recovery, direct `--check` must pass. If it does not, compare target, schema,
toolchain, and generator revision in that order. Compiler fix-ups and hand edits are
not recovery tools.

## 5. Run engine-free acceptance

From the repository root:

```console
export RUSTC_WRAPPER=
export CARGO_TARGET_DIR="$(mktemp -d "${TMPDIR:-/tmp}/dagger-rust-acceptance.XXXXXX")"
cargo fmt --all --check
cargo check --workspace --all-features --locked
cargo test --workspace --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps --locked
cargo test -p tokeira-dagger-sdk --no-default-features --locked
cargo deny check
cargo run -p dagger-bootstrap --bin dagger-rust --locked -- \
  generate --workspace . --check
```

Also run the direct Go ABI and Dagger-module Go tests listed in
[engine-integration.md](engine-integration.md). `dagger call ci gates` runs all of these
in containers ([ci.md](ci.md)); CI calls it for every pushed commit, and `main` requires
it to pass. Review dependency activation with
`cargo tree -p tokeira-dagger-sdk -e features` and inspect every Cargo Deny advisory, license,
ban, and source result. These checks do not construct an engine.

Acceptance confirms `unsafe_code = "deny"`, both public package contents, default and
no-default feature paths, warning-denied documentation, generated serde semantics,
owned-path confinement, checksum verification, and credential-safe diagnostics.

## 6. Release

A release publishes both public crates to crates.io at one version and tags the commit.
The maintainer publishes by hand, with their own crates.io token from `cargo login`; no CI
system holds one. A crate's first release needs a token that may publish new crates. The
ci module's `package` prepares the archives, and `published` proves the result
([ci.md](ci.md)):

1. Prepare one release commit that moves the release identity (below) to the new
   version. Its changelog section comes from
   `CHANGIE_ENGINE_VERSION=$(jq -r .engine_version codegen/target.json) changie batch <version>`,
   which names the targeted engine in the section's header, and `changie merge` adds it
   to `CHANGELOG.md`.
2. Run the section-5 gate and the
   [stock-engine acceptance](engine-integration.md#stock-engine-acceptance) on that
   commit.
3. Package the release commit, then review each crate's file list:
   `dagger call ci package --commit <commit> export --path release`.
4. Tag the commit `vX.Y.Z`, annotated and under the noreply identity, and push the tag.
5. From a clean checkout of the tag, publish the macros crate, wait until
   `https://index.crates.io/to/ke/tokeira-dagger-sdk-macros` lists the new version, then
   publish the SDK: `cargo publish --locked -p tokeira-dagger-sdk-macros`, then
   `cargo publish --locked -p tokeira-dagger-sdk`. Packaging is reproducible, so each
   upload is the archive `package` returned, byte for byte.
6. Prove the release: `dagger call ci published --sums release/SHA256SUMS`.
7. Create the GitHub release from the tag, with the changelog section as its notes.

Tags and releases are immutable: never delete or recreate one. Nor does crates.io let a
version be replaced: a fix is a new version, and a broken one is yanked.

## Release identity

Every release carries one identity, the workspace version, starting at `0.1.0`. Both
crate versions, the Git tag `vX.Y.Z`, the changelog section, and the checked target's
`rust_sdk_version` with its approved pin all state it, and each release moves them
together.

The engine and CLI are stock Dagger releases, versioned independently of the SDK.
`codegen/target.json` names the exact Dagger version and revision an SDK release
targets, and the connector refuses an engine with a different version or revision. A
target refresh (section 3) moves them deliberately.
