# Design Document: Rust SDK Module Clients

## Overview

Feature 7 answers the two SDK-module calls through which the engine at
`v1.0.0-beta.27` manages clients: `findClientRoot` and `generateScope`
(`core/sdkmodule/provider.go`). The engine owns the records in `dagger.toml` and the
pins in `dagger.lock`, and its commands drive both calls
(`withSDKModuleClient`, `withoutSDKModuleClient`, and `withUpdatedSDKModuleClients` in
`core/schema/workspace_sdk_module.go`). The Rust runtime turns the client set the
engine passes into one of two results.

In a module scope the clients become the module's dependencies. The runtime rewrites
`[[dependencies]]` in `dagger-module.toml` and the engine's module code generation
does the rest, as Feature 6 delivers it, with Go's ownership rule for the files it
generates (Module ownership, below).

In a package scope each client becomes a Rust module under `src/dagger_clients/`:

```
app/Cargo.toml                                    the exact SDK dependency
app/src/dagger_clients/mod.rs                     the client modules, sorted
app/src/dagger_clients/.dagger-generated.json     ownership record for mod.rs
app/src/dagger_clients/greeter/.dagger-generated.json
app/src/dagger_clients/greeter/mod.rs             Client, r#ref, load, ClientLoadable
app/src/dagger_clients/greeter/<type>.rs          one file per type of the target
```

```rust
mod dagger_clients; // written once by the author

use dagger_clients::greeter;

let client = dagger_sdk::connect().await?;
let message = greeter::Client::new(&client)
    .greeter_opts(&greeter::QueryGreeterOpts::default().with_greeting("hi"))
    .hello("Ian")
    .await?;
```

The design adds three things and removes one. `tokeira-dagger-sdk` gains module
loading. The dependency renderer gains a client surface. The engine operation tool
gains two requests, one per kind of scope. The standalone-client generator, which no
command reaches at this release, is removed.

## Dependencies and Non-Goals

### Owning relationships

- Feature 2 owns the session and its close semantics; module loading lives inside
  the session and ends with it.
- Feature 3 owns request execution and the error model that the three new messages
  extend.
- Feature 4 owns the Core bindings every client reuses through the SDK dependency.
- Feature 5 owns the operation tool, its request protocol, the content sealing, and
  the Go adapter that both new requests pass through. Feature 7 gives the tool's
  ownership check Go's rule and carries a module's removals through the adapter.
- Feature 6 owns the dependency renderer, its naming and refusals, and module
  generation. Feature 7 adds a surface to that renderer and changes none of its
  output for modules.

### Construction rules

- Go is the reference. Where Rust differs, the difference is listed under
  [Deviations from Go](#deviations-from-go). Citations name files in the trees the
  requirements name: dagger/dagger at `v1.0.0-beta.27` for `core/` and `sdk/go/`
  paths, `github.com/dagger/go-sdk` at `bb23e8d`, and `github.com/dagger/sdk-helpers`
  at `v1.0.12`.
- The Go adapter is data only: it reads engine objects, writes one request document,
  mounts inputs, and applies the result. Naming, rendering, ownership, and Cargo
  policy are Rust's.
- A target is read only through `clientSchemaIntrospectionJson`. No code path calls
  `withEngineVersion` on a target.
- A scope is validated as a whole before anything is written.
- No new third-party dependency. Manifests are edited with `toml_edit`, which the
  operation tool already uses.

### Non-goals

- Go's Dang-entrypoint module layout, package clients beside a module, and scope
  settings.
- Resolving `Cargo.lock`, or running any Cargo command in the author's package during
  generation.
- A process-wide default session.

## Architecture

### Control plane

```
dagger module client add | rm | update, dagger generate, dagger lock update
        │  engine: dagger.toml records, dagger.lock pins
        ▼
findClientRoot(ws)                      nearest Cargo.toml, or null
generateScope(ws, isModule, name, clients)
        │
        ├─ isModule ──► write-module-dependencies ──► dagger-module.toml
        │               then the engine's module code generation (Feature 6)
        │
        └─ otherwise ─► generate-package-clients ───► src/dagger_clients/**, Cargo.toml
```

`FindClientRoot` in `runtime/main.go` is unchanged. `GenerateScope` stops refusing
clients. In both branches the Go adapter builds one request and the Rust operation
tool returns the paths it wrote and removed, which the adapter applies to the
`Workspace`.

### Run-time data plane

```
greeter::Client::new(&client)          no I/O
        │  QueryBuilder carrying (address, pin)
        ▼
first request ──► serveModule(address, refPin)   once per session and (address, pin)
        ▼
the request itself ──► missing field ⇒ "out of date; regenerate it"
```

## Components and Interfaces

### Module loading (`crates/dagger-sdk`)

This follows `ModuleGraphQLClient` and `ensureModule` in `sdk/go/module_client.go`.

```rust
impl Client {
    #[doc(hidden)]
    pub fn generated_module_query_builder(&self, address: &str, pin: &str) -> QueryBuilder;
}
```

The builder carries a module requirement `(address, pin)`. The requirement lives on
the session handle that every builder and handle clones (`SessionHandle` in
`lifecycle.rs`), so `select`, `generated_query_root`, `generated_core_handle`, the
re-entry builders, and every Core handle built from them inherit it.
`Client::query()` carries none.

The session keeps a map from requirement to load state. The single point through
which generated requests reach `session.execute` first awaits the load:

- An absent entry starts `query{serveModule(address:"…",refPin:"…")}`, with the
  strings encoded by the SDK's argument encoding. `refPin` is always sent; it is `""`
  for a local target, as Go sends it.
- A present entry is awaited. Dropping a waiter does not cancel the load. If the
  future driving the load is dropped, the entry returns to absent and the next
  request starts it.
- A failed load is delivered to every waiter and the entry is cleared.
- A closed session fails before the map is consulted.
- A request that sends nothing, such as the loaded-ID short circuit in
  `Selection::execute`, does not reach this point.

Errors are a new `QueryError` case (the enum is `#[non_exhaustive]`),
`ModuleClient`, which holds a `ModuleClientError` of one of three kinds. Its
`Display` is Go's wording without the cause, which is its `source()`:

| Condition | Message |
|---|---|
| `serveModule` is not in the schema | `the engine does not support generated module clients (serveModule is required)` |
| Any other load failure | `load module "<address>"` |
| A later request names a field the schema lacks | `generated client for "<address>" is out of date; regenerate it` |

A field is missing when a GraphQL error has extension `code` equal to
`GRAPHQL_VALIDATION_FAILED`, no `path`, and a message starting `Cannot query field `
(`missingFieldError` in `sdk/go/module_client.go`). The classification also applies
when the errors arrive in an HTTP error response.

### Client surface (`crates/dagger-codegen`)

`BindingSurface` (`engine/dependency.rs`) gains `Client` beside `Library` and
`Module`. It renders one target into a directory: `mod.rs` and one file per type the
target owns.

The engine installs the bound module in a client schema "like any dependency"
(`clientSchemaIntrospectionJSONFile` in `core/schema/modulesource.go`), so the
target's types are those `VisibleSchemaPlan::is_dependency_type` already selects.
Core types are never rendered; bindings name them through the SDK alias, which is
the package's dependency key with `-` replaced by `_`.

A client's `mod.rs` declares:

```rust
#[derive(Clone)]
pub struct Client { /* QueryBuilder from generated_module_query_builder(ADDRESS, PIN) */ }

impl Client {
    pub fn new(client: &dagger_sdk::Client) -> Self;
    pub fn r#ref<T: ClientLoadable>(&self, id: impl Into<dagger_sdk::IdInput<T>>) -> T;
    pub async fn load<T: ClientLoadable>(
        &self,
        id: impl Into<dagger_sdk::IdInput<T>>,
    ) -> Result<T, dagger_sdk::QueryError>;
}

pub trait ClientLoadable: Sized { /* doc-hidden methods */ }
```

- `ClientLoadable` has the shape of `ModuleLoadable` (`module/render.rs`): a blanket
  implementation over the SDK's `Loadable` for Core handles, and one implementation
  per target handle in that handle's file.
- Constructors are the `Query` fields that return the target's object. They are
  rendered in the object's file as `impl super::Client`, as the module surface
  renders `impl super::ModuleQuery`, each with its `_opts` method and
  `Query<Field>Opts` struct when it has optional arguments.
- `mod.rs` declares and re-exports each type file. It does not re-export the SDK:
  the client's API contains only the bound module, as Go's `Client` does
  (`generate_unified.go`).
- `mod.rs` allows, for the whole client, what generated code cannot avoid:
  `dead_code`, `deprecated`, the Clippy lints the dependency bindings allow, and
  `clippy::module_inception`, since the type named after the module is bound in a
  file named as the directory is. Each re-export allows `unused_imports`: an author
  may name none of a file's types, and nothing else names a type that none of the
  module's functions returns.
- The address and pin are string literals in `Client::new`, and nowhere else.
- The first line of every file begins `// @generated`. The existing
  `generated_header` puts a `//!` line first, so the client surface writes the marker
  line before it.
- Selections, inline fragments, and `node(id:)` type names use the schema's wire
  names unchanged. Only Rust identifiers pass through `naming.rs`.

The surface refuses a target type named `Client` or `ClientLoadable`, a constructor
that would be named `new`, `ref`, or `load`, and everything the dependency renderer
refuses.

### Package clients (`crates/dagger-sdk-engine`)

`EngineExecutionRequest` (`model.rs`) gains `generate-package-clients`:

| Field | Content |
|---|---|
| `format_version` | The request protocol's version |
| `target` | `TargetIdentity` |
| `sdk_dependency` | `PublishedSdkDependency` |
| `client_root` | The Cargo package directory beneath the operation root |
| `clients[]` | `name`, `address`, optional `pin`, and `schema` (path and digest) |

The runner refuses a package that is a module on `dagger.json`, as Module
dependencies describes, verifies every schema digest, then plans the whole scope:

1. Derive each client's Rust module name from `name` with the repository's module
   naming. Directory names never carry `r#`; the `mod` line uses it where Rust
   requires it. Refuse two clients with one name.
2. Render each client through the client surface and format it with the SDK's
   rustfmt (`PostWorkPlan::FormatRust`).
3. Render the aggregator `src/dagger_clients/mod.rs`:

   ```rust
   // @generated …
   //! Typed clients for Dagger modules.

   #[rustfmt::skip]
   pub mod greeter;
   ```

   `#[rustfmt::skip]` on an out-of-line module makes the author's rustfmt leave the
   client's files alone, whatever its configuration. The aggregator itself is short,
   sorted, and stable under rustfmt's defaults. It allows no lint: each client's
   `mod.rs` allows what its code cannot avoid (see Client surface), where
   `#[rustfmt::skip]` keeps the author's rustfmt away from the list.
4. Plan the `Cargo.toml` edit.
5. Validate ownership for every client directory, every directory to prune, and the
   aggregator.
6. Write, prune, and write the records.

Any failure before step 6 writes nothing. The result lists `touched_paths` and
`removed_paths`, relative to the operation root.

#### Ownership

Ownership follows `ownedOverlay` in `generator/ownership.go` and `runPruneClients` in
`prune.go` of `github.com/dagger/go-sdk`. Where Go tests for its
`// Code generated by dagger.` prefix, Rust tests that the first line begins
`// @generated`; "the marker" below means that test.

- The record is `{"version": 1, "files": {path: sha256-hex}}`, keys sorted,
  two-space indentation, trailing newline. Paths are relative to the record's
  directory and never include the record.
- An existing file with different bytes may be overwritten only if it begins with the
  marker and either is not in the old record or still has its recorded digest.
  Otherwise: `<path> contains user changes; refusing to overwrite it`.
- A recorded file that is no longer generated is removed only if it begins with the
  marker and still has its recorded digest.
- A directory under `src/dagger_clients/` with a record that is not in the client set
  is pruned by the same rule, and its record is rewritten with no files, as Go keeps
  the emptied record.
- The aggregator has its own record in `src/dagger_clients/`, treated the same way
  when the client set becomes empty.
- A generated path or a record's directory that crosses a symbolic link is refused. A
  malformed record or an unknown version is refused.

#### Cargo manifest

The edit reuses the manifest code that writes the SDK dependency for a new module
(`project/manifest.rs`, `initialization.rs`).

| State of `[dependencies]` | Result |
|---|---|
| An entry for `tokeira-dagger-sdk`, by key or by `package` | Its source and requirement fields are replaced; its key and other fields are kept |
| No entry | `dagger-sdk = { package = "tokeira-dagger-sdk", … }` is added |
| `{ workspace = true }`, and the workspace entry states the requirement | Unchanged |
| `{ workspace = true }`, and the workspace entry differs | Refused, with the exact entry to write |
| No `[package]` table | Refused |
| No client generated | `Cargo.toml` untouched |

The requirement is `version = "=X.Y.Z"` for a registry release, or `git` and `rev`
for a commit. `Cargo.lock` is not an input and is never written.

### Module dependencies (`crates/dagger-sdk-engine`)

`EngineExecutionRequest` gains `write-module-dependencies`, with the module root and,
for each client, `name`, kind, and `as_string` (Git) or `source_root_subpath`
(local). It rewrites `dagger-module.toml` with `toml_edit` by the rules of
`withLegacyRuntimeDependencies` and `prepareOutput` in `main.dang` of
`github.com/dagger/sdk-helpers` at `v1.0.12`:

| Case | Written dependency |
|---|---|
| Local target | `source` = its path relative to the module directory |
| Git target | `source` = `asString` |
| Git target that is the workspace's own module | Written as a local target |
| No existing entry with that source | `name` = `moduleOriginalName` |
| Existing entry with the same normalized source | Its name and settings kept; its `pin` dropped |
| Such an entry without a `name` | `name` = `moduleOriginalName`; its settings kept |
| Existing entry matching no target | Removed |

A Git target is the workspace's own when the workspace is itself a remote checkout
and `ws.moduleSource("/" + sourceRootSubpath).asString` equals the target's
`asString`; an error counts as not equal (`inWorkspace`). The adapter makes that
lookup and passes the request the resulting kind, so the Rust operation sees such a
target as local.

Tables are ordered by name, ascending. Each is written in the helper's form, without
indentation, `name` then `source` then kept settings, after `[runtime]`. Everything
outside the dependency tables is kept byte for byte, and a second run with the same
clients changes nothing. `moduleManifest` in `runtime/main.go` writes a new module's
`[runtime]` table in the same form.

A module configured by `dagger.json` is left alone while it has no dependencies and
no clients. Otherwise it is refused before anything is written, with advice to run
`dagger module migrate` on the module's path from the workspace root, then
`dagger generate`, as Dagger's migration guide describes; a coding agent passes `-y`
to apply each, or `--no-apply` to preview the migration. The path begins with `/`,
which `dagger module migrate` reads from the workspace root wherever it runs.

`dagger module client add` in a module on `dagger.json` reaches generation as a
package scope instead: the engine takes a scope that `dagger.toml` does not record yet
for a package's (`withSDKModuleClient` in `core/schema/workspace_sdk_module.go`), and
Go writes a client package there. A package's clients are refused the same way when
its directory holds `dagger.json`, or when a `dagger.json` above it, as far as the
workspace root, names it as the module's `source`. The advice then ends with
`dagger module client add` again, run from the directory that holds `dagger.json`:
the migration records that directory as the module's scope, and from a package below
it the command would still ask for package clients.

#### Module ownership

Module generation keeps Feature 5's operation manifest,
`.dagger/rust/operation-manifest.json`, as its record, and `verify_ownership` applies
`ownedOverlay` with it:

- A file that does not exist is written, so a deleted binding comes back.
- A file with other bytes is replaced when the manifest records it and it still has
  its recorded digest, or when the manifest does not record it and it begins with
  the marker. Otherwise: `<path> contains user changes; refusing to overwrite it`,
  and nothing is written.
- A recorded file that generation no longer writes is removed while it still has its
  recorded digest. An edited one stays as its author left it, and the new manifest
  no longer records it, so it is the author's from then on.

Go keeps such a record for its package clients and its entrypoint modules. Its
modules of the runtime layout, the layout Rust's modules have, keep none, and there
the engine's changeset is the only review.

### Go adapter (`runtime/main.go`)

For a package scope the adapter:

1. Resolves the SDK as scaffolding does, for the descriptor and `sdk_dependency`.
2. For each client reads `ModuleOriginalName`, the address and pin, and
   `ClientSchemaIntrospectionJSON` with `ClientVersion` set to `"v"` plus the
   descriptor's engine version. The address rule is Go's (`generate_unified.go`): a
   Git source gives `AsString` and `Pin`; a local source gives
   `"/" + SourceRootSubpath` and no pin; any other kind is refused. A Git source in
   the Git workspace's repository, at the workspace's commit and selector, is bound
   as a local source, by its path, as `workspaceModule` in
   `cmd/dagger-go-sdk-codegen/main.go` binds it: its address would name a commit
   that every commit changes. The adapter reads what Go reads for that: the
   workspace's `address`, the commit of its `git.head`, where an error counts as no
   commit, and the source's `cloneRef`.
3. Mounts the `Cargo.toml` and `dagger.json` of the scope and of each directory above
   it up to the workspace root, `src/dagger_clients/**`, and the schemas.
4. Applies each touched path with `WithFile` and each removed path with
   `WithoutFile`.

For a module scope it runs `write-module-dependencies`, writes the manifest, and
regenerates through the engine. The engine returns a module's generated code as the
files that differ from the module, which cannot say that a file has gone, so the
generation result names the files Rust removed and `Codegen` lists them, from the
module's source root, in `.dagger/rust/removed-paths.json` in the code it returns.
The adapter takes the list out of the code, writes the code, and removes each listed
file with `WithoutFile`. It compares no bytes: Rust has kept each edited file and left
it out of the list. It never writes `dagger.toml`.

### Removal

The following go, with their tests and fixtures:

- Go: `GenerateClient`, `RequiredClientGenerationFiles`, the client-generation
  metadata and its sealing, and the client-set plan in `internal/metadata`.
- Rust: `OperationKind::GenerateClient`, `engine/client.rs`, `engine/metadata.rs`,
  `assets/client-generation.json`, `dagger-codegen/src/client/`,
  `dagger-sdk-engine/src/client/`, the `InitializeClient` and `PlanClientSet`
  requests, and the runner's client paths.

A type the remaining code uses moves out of the removed modules first. The runtime's
`include` list and the sealing filter in `runtime/content.go` are re-checked against
the remaining operations.

## Deviations from Go

| Go | Rust | Reason |
|---|---|---|
| Reads `clientSchemaIntrospectionJSON` without `clientVersion` (`generateStandaloneClient` in `go-sdk.dang`) | Passes the target release | Core in the schema must match the SDK's Core types |
| `Connect(connections...)`, with a default session | `Client::new(&client)` | The SDK has no process-wide session |
| Clients at `internal/dagger/clients/<module>/` | `src/dagger_clients/<module>/` | Cargo's source layout |
| `go mod tidy` after setting the requirement | `Cargo.lock` untouched | Generation runs no Cargo command in the author's package |
| Marker `// Code generated by dagger.` | `// @generated` | The marker the SDK's generated Rust already uses, here on the first line |
| No formatter or lint attributes | Allowances in each client's `mod.rs`, and `#[rustfmt::skip]` in the aggregator | The author's `cargo fmt` and `cargo clippy` must pass |
| Appends the cause to each module-loading message with `%w` | The message alone; the cause is `source()` | As elsewhere in `tokeira-dagger-sdk`, so a printer of the whole chain shows the cause once |
| Replaces or removes a recorded file only if it begins with the marker, except its compilation helper's files, which it removes on their digest alone (`ownedOverlay`) | In a module, a file the operation manifest records is replaced or removed on its digest alone; an unrecorded file needs the marker | A module's dependency bindings begin with their documentation, and its JSON control files cannot carry a comment |

## Correctness Properties

### Property 1: A scope's output is a function of its client set

*For any* client set, generating a scope twice SHALL produce the same bytes, and
generating the set after any earlier set SHALL leave the same generated files as
generating it first, apart from emptied records.

**Validates: Requirements 1.5, 4.1–4.3, 9.9, 9.14, 12.2–12.4**

### Property 2: A module's dependencies are exactly its clients

*For any* existing manifest and client set, the written dependency list SHALL hold
one entry per client and no other, with kept names and settings, no pins, and every
byte outside the dependency tables unchanged.

**Validates: Requirements 2.1–2.12, 3.1–3.4**

### Property 3: Bindings use the served names

*For any* client schema, every selection, inline fragment, and `node(id:)` type name
SHALL be a wire name of that schema, and no client path SHALL read a target through
a forced engine version.

**Validates: Requirements 5.1–5.7**

### Property 4: A client's API is the bound module

*For any* client schema, the generated module SHALL expose the target's
constructors, its owned types, `r#ref`, and `load`, with the names the dependency
renderer gives the same schema, and no Core field.

**Validates: Requirements 6.1–6.10, 11.2, 11.3, 11.5**

### Property 5: One load per session and requirement

*For any* interleaving of requests, drops, and failures, at most one `serveModule`
SHALL be in flight per session and requirement, every waiter SHALL observe its
result, a success SHALL never be repeated, and a failure SHALL be retried by the next
request.

**Validates: Requirements 8.1–8.10**

### Property 6: Only a missing field is classified

*For any* GraphQL error, the unsupported-engine and out-of-date messages SHALL be
produced exactly when the error is a missing-field validation error.

**Validates: Requirements 8.11–8.15, 12.5**

### Property 7: Edited files are never lost

*For any* tree of generated, edited, and authored files, generation SHALL overwrite
or remove only files that begin with the marker and still have their recorded digest,
and a refused scope SHALL be left byte-identical.

**Validates: Requirements 9.1–9.13, 11.6**

### Property 8: The Cargo edit is minimal and exact

*For any* valid package manifest, the result SHALL state exactly the SDK dependency
under one key, keep every other byte, and never be produced for a manifest generation
cannot edit safely.

**Validates: Requirements 10.1–10.10**

### Property 9: Generated code passes the author's checks

*For any* generated package, `cargo fmt --check` and
`cargo clippy --all-targets -- -D warnings` SHALL pass from the package root with no
author configuration.

**Validates: Requirements 7.1–7.7**

## Error Handling

| Condition | Behaviour |
|---|---|
| Two clients with one Rust module name | Diagnostic naming both; nothing written |
| A client whose Rust module would be `mod` | Diagnostic; nothing written |
| Target type named `Client` or `ClientLoadable` | Diagnostic at the schema coordinate |
| Constructor named `new`, `ref`, or `load` | Diagnostic at the schema coordinate |
| Target that is neither local nor Git | Refused by the adapter before any request |
| Schema digest mismatch | Diagnostic; nothing written |
| Edited generated file in the way | `<path> contains user changes; refusing to overwrite it` |
| Symbolic link on a generated path or record directory | Diagnostic; nothing written |
| Malformed or unsupported ownership record | Diagnostic; nothing written |
| `Cargo.toml` without `[package]` | Diagnostic; nothing written |
| Inherited SDK dependency that differs | Diagnostic with the exact workspace entry to write |
| Engine without `serveModule` | Unsupported-engine error on the first request |
| Module fails to load | `load module "<address>"`, with the cause as its source; retried by the next request |
| Target changed since generation | Out-of-date error on the request |

Refusals use the existing `DiagnosticSet` and `DiagnosticCode` machinery. Diagnostics
name schema coordinates and operation-relative paths, never host paths or credentials.

## Testing Strategy

- **Transport.** Tests over the SDK's fake connection mirror
  `sdk/go/module_client_test.go`: loads shared by session and pin, a failed load
  retried, a cancelled wait that does not cancel the load, lazy construction that
  follows the session's lifecycle, and missing-field classification, including in an
  HTTP error response.
- **Renderer.** Two checked client schemas fetched from an engine at
  `v1.0.0-beta.27`: the acceptance's greeter module, and a module with an acronym in
  a type name. A compile fixture builds the greeter client with warnings denied, with
  a caller that uses the constructor with and without options, `r#ref`, and `load`,
  and never names a type of the greeter's that no function returns.
  Unit tests cover each refusal.
- **Package clients.** Unit tests cover the record format, each overwrite and prune
  rule, validation before writing, symbolic links, each row of the Cargo table, the
  empty set, name collisions, and a package that is a module on `dagger.json`.
- **Author's checks.** A test builds a binary crate with `mod dagger_clients;` that
  uses part of one client, then runs `cargo fmt --check` and
  `cargo clippy --all-targets -- -D warnings` from its root.
- **Module dependencies.** Unit tests cover each row of the dependency table, a
  hand-written dependency removed, an unnamed entry named, comments kept, the unindented form, and a second
  run that changes nothing, and the refusal of a `dagger.json` module with its advice.
- **Module generation.** Tests through the generation stack cover each case of the
  module ownership rule: a deleted binding written again; the unchanged bindings of a
  removed dependency removed and named in the result; an edited one kept, left out of
  the record and refused when the dependency comes back; and an edited file that
  generation would rewrite refused with nothing written. Unit tests of
  `verify_ownership` cover the same cases and an unrecorded destination.
- **Go adapter.** Tests cover both request documents, the address rule, and the
  removal list: paths from the module's source root, a path outside the module
  refused, no list removing nothing, and a damaged list refused. Source tests assert
  that no client path calls `WithEngineVersion`, that `Codegen` and `GenerateScope`
  pass removals through the list, and, beside the existing test that the adapter
  holds no Rust behaviour, that the list's code computes no digest.
- **Acceptance.** The ci module generates a Cargo package with a local and a Git
  client of the greeter module and runs it against the engine. It then runs
  `dagger module client update`, restores a deleted file with `dagger generate`,
  prunes with `dagger module client rm`, and checks that an edited file is refused
  with nothing written. The `gate` module receives `greeter` through
  `dagger module client add`. Targets are Dang modules, which need no Cargo build.
