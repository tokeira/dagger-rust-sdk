# Requirements Document: Rust SDK Module Clients

## Introduction

Feature 7 gives Rust the module clients that the Dagger engine drives for every SDK.
`dagger module client add <module>` records a target module against an SDK scope, and
the SDK writes the code that calls it. In a Rust module, a client becomes a dependency
with the typed bindings module authoring already generates. In any other Cargo package,
a client becomes a typed Rust module that loads its target on first use.

The Dagger engine at `v1.0.0-beta.27` (the Target_Release) is authoritative for the
SDK-module interface, the client records in `dagger.toml`, the pins in `dagger.lock`,
`serveModule`, and the client schema. Dagger's Go SDK defines observable behaviour
where the engine contract does not settle it: the Go SDK module
`github.com/dagger/go-sdk` at `bb23e8d`, its manifest helper
`github.com/dagger/sdk-helpers` at `v1.0.12`, and Go's module transport in
`sdk/go/module_client.go` of dagger/dagger at the Target_Release. Citations name a
file and function in those trees. Go packages, `go.mod` handling, and the process-wide
default session are evidence rather than a Rust design.

Rust owns the public shape. Generated clients are ordinary Rust modules inside the
author's Cargo package, use `tokeira-dagger-sdk`'s Core types, session, and error
model, and follow the calling convention of the dependency bindings that module
authoring generates. The workspace builds with Rust 1.99.1, edition 2024.

This specification replaces the earlier standalone-client contract, which described a
client-initialization command, one generated Cargo project per client, and one module
per client. None of that is reachable on a stock engine at the Target_Release, and
Requirement 13 removes its implementation.

Feature 7 depends on Feature 2's client ownership, Feature 3's transport, Feature 4's
Core bindings, Feature 5's engine operation seam and Go adapter, and Feature 6's
dependency bindings and module generation.

## Glossary

- **Client_Module:** The generated Rust module for one Client_Target, at
  `src/dagger_clients/<name>/` in a Package_Scope.
- **Client_Record:** One entry of a scope's `clients` list in `dagger.toml`, written
  by the engine (`SDKScope` in `core/workspace/config.go`).
- **Client_Root:** The nearest directory at or above the workspace's current location
  that holds a `Cargo.toml`, as `findClientRoot` returns it.
- **Client_Schema:** The introspection schema the engine builds for one Client_Target:
  Core plus that module installed under its own name, without the module's
  dependencies (`clientSchemaIntrospectionJSONFile` in
  `core/schema/modulesource.go`).
- **Client_Target:** A local or Git module that a Client_Record names, passed to
  `generateScope` as a `ModuleSource`.
- **Generated_Marker:** The prefix `// @generated`. A file is generated when its first
  line begins with it, and every generated client file has such a first line.
- **Module_Requirement:** The address and pin of the module a generated client needs
  served before its requests run.
- **Module_Scope:** A scope the engine passes with `isModule` true: a Rust module.
- **Ownership_Record:** A `.dagger-generated.json` file listing generated files and
  their SHA-256 digests.
- **Package_Scope:** A scope the engine passes with `isModule` false: a Cargo package
  that is not a Dagger module.
- **SDK_Dependency:** The exact `tokeira-dagger-sdk` requirement of the runtime that
  generates: `=X.Y.Z` for a registry release, or the Git URL and full revision of a
  commit.
- **Target_Release:** Dagger `v1.0.0-beta.27`, the release named by
  `codegen/target.json`.

## Target State

In a module, `dagger module client add ../greeter` makes the engine record the client
in `dagger.toml`, and generation rewrites the module's `dagger-module.toml` so that
its `[[dependencies]]` are exactly the recorded clients. Module code calls the
dependency as before, through `context.query().greeter()`.

In a Cargo package that is not a module, the same command produces
`src/dagger_clients/greeter/`. The author declares `mod dagger_clients;` once and
writes:

```rust
use dagger_clients::greeter;

let client = dagger_sdk::connect().await?;
let message = greeter::Client::new(&client)
    .greeter_opts(&greeter::QueryGreeterOpts::default().with_greeting("hi"))
    .hello("Ian")
    .await?;
```

A package may hold any number of clients, one Rust module each. The package's
`Cargo.toml` carries the SDK_Dependency. Each client directory has an
Ownership_Record, so regeneration never overwrites an edited file and removing a
client deletes only its unchanged files. The first request through a client serves
its module in the caller's session; a client older than its target fails with a
message that says to regenerate it.

## Authority and Verification Boundary

| Claim | Authority | Evidence |
|---|---|---|
| Rust answers the engine's client calls | `findClientRoot` and `generateScope` in `core/sdkmodule/provider.go` | Go adapter request-document tests; acceptance |
| A module's dependencies are its clients | `withLegacyRuntimeDependencies` and `prepareOutput` in `main.dang` of `github.com/dagger/sdk-helpers` at `v1.0.12` | Manifest-rule unit tests; acceptance `gate` module |
| A client's API is the bound module only | `Client` in `generate_unified.go` of `github.com/dagger/go-sdk`; Feature 6 dependency bindings | Compile fixture with warnings denied |
| A client loads its module once per session | `ensureModule` in `sdk/go/module_client.go` | Transport tests that mirror `sdk/go/module_client_test.go` |
| A client uses the served names | `clientSchemaIntrospectionJSONFile` in `core/schema/modulesource.go` | Acronym-named schema fixture; adapter source test |
| Edited files survive | `ownedOverlay` in `generator/ownership.go` and `runPruneClients` in `prune.go` of `github.com/dagger/go-sdk` | Ownership unit tests; acceptance |
| A module's edited generated files survive | `ownedOverlay` in `generator/ownership.go` of `github.com/dagger/go-sdk`, with the operation manifest as the record | Ownership unit tests; module generation tests |
| A `dagger.json` module is migrated before it has clients | `docs/current_docs/config/migrate-dagger-json.mdx` in `github.com/dagger/dagger` | Refusal unit test |
| A package client runs against an engine | The Target_Release | Acceptance with one local and one Git target |

## Requirements

### Requirement 1: Engine Control Plane

**User Story:** As a Dagger user, I want the engine's own client commands to work for
Rust, so that I manage Rust clients as I manage any other SDK's.

#### Acceptance Criteria

1. THE Rust runtime SHALL expose client generation only through the SDK-module
   functions `findClientRoot` and `generateScope`.
2. WHEN the engine calls `findClientRoot`, THE runtime SHALL return the Client_Root,
   or null when no `Cargo.toml` is at or above the current location.
3. WHEN the engine calls `generateScope` for a Module_Scope, THE runtime SHALL
   generate the scope as Requirements 2 and 3 state.
4. WHEN the engine calls `generateScope` for a Package_Scope, THE runtime SHALL
   generate the scope as Requirements 4 to 11 state.
5. THE runtime SHALL treat the `clients` argument as the complete client set of the
   scope.
6. THE runtime SHALL leave `dagger.toml` and `dagger.lock` to the engine.
7. THE runtime SHALL return every change as edits to the `Workspace` it received.
8. WHEN `dagger module client add`, `dagger module client rm`,
   `dagger module client update`, `dagger generate`, or `dagger lock update` reaches a
   Rust scope, THE result SHALL be the scope's state for the client set the engine
   passes.
9. THE Go adapter SHALL carry data between the engine and the Rust operation and
   SHALL hold no rendering, naming, ownership, or Cargo policy.

### Requirement 2: Module Scopes

**User Story:** As a Rust module author, I want `dagger module client add` to give my
module a dependency, so that I stop editing `dagger-module.toml` by hand.

#### Acceptance Criteria

1. WHEN a Module_Scope is generated, THE runtime SHALL rewrite the `[[dependencies]]`
   of its `dagger-module.toml` to exactly the scope's Client_Targets.
2. WHEN a Client_Target is local, THE dependency's `source` SHALL be its path relative
   to the module directory.
3. WHEN a Client_Target is a Git module, THE dependency's `source` SHALL be the
   target's `asString`.
4. WHEN a Git Client_Target is the workspace's own module, THE runtime SHALL write it
   as a local target. It is the workspace's own when
   `ws.moduleSource("/" + sourceRootSubpath).asString` equals the target's `asString`;
   an error from that lookup counts as not equal (`inWorkspace` in `main.dang` of
   `github.com/dagger/sdk-helpers`).
5. THE runtime SHALL write no `pin` on any dependency.
6. WHEN no existing dependency has the target's source, THE new dependency's `name`
   SHALL be the target's `moduleOriginalName`.
7. WHEN an existing dependency has the same normalized source as a Client_Target, THE
   runtime SHALL keep that dependency's name and other settings.
8. WHEN such an existing dependency has no `name`, THE runtime SHALL give it the
   target's `moduleOriginalName` and SHALL keep its other settings
   (`adoptedDependency` in `main.dang` of `github.com/dagger/sdk-helpers`).
9. WHEN an existing dependency matches no Client_Target, THE runtime SHALL remove it.
10. THE runtime SHALL order dependencies by name, ascending.
11. THE runtime SHALL keep every other part of the manifest, comments included, as
    Requirement 3 states.
12. WHEN the scope has no Client_Targets, THE runtime SHALL write an empty dependency
    list.
13. AFTER the manifest is written, THE runtime SHALL regenerate the module through the
    engine's module code generation, which is Feature 6's except for the ownership of
    the files it generates (criteria 16 and 17).
14. THE rule SHALL apply both to a scope that already has a manifest and to one the
    runtime has just scaffolded.
15. WHEN a Module_Scope is configured by `dagger.json` and has dependencies or
    Client_Targets, THE runtime SHALL refuse it before writing anything. The refusal
    SHALL advise `dagger module migrate` with the module's path from the workspace
    root, then `dagger generate`, and SHALL say that a coding agent passes `-y` to
    apply each, or `--no-apply` to preview the migration.
16. WHEN a Module_Scope is regenerated, THE runtime SHALL treat its generated files as
    `ownedOverlay` in `generator/ownership.go` of `github.com/dagger/go-sdk` treats
    the files it records, with the previous operation manifest,
    `.dagger/rust/operation-manifest.json`, as the record. A missing file SHALL be
    written. A file with other bytes SHALL be replaced only if the manifest records it
    and it still has its recorded digest, or the manifest does not record it and its
    first line begins with the Generated_Marker; otherwise generation SHALL fail with
    `<path> contains user changes; refusing to overwrite it` before anything is
    written. A recorded file that is no longer generated SHALL be removed only if it
    still has its recorded digest; an edited one SHALL stay, and the new manifest
    SHALL not record it.
17. WHEN regeneration removes a file of a Module_Scope, THE runtime SHALL remove it
    from the workspace.

### Requirement 3: Module Manifest Form

**User Story:** As a Rust module author, I want generation to write my manifest the
way Dagger's own tooling does, so that the two never reformat each other's output.

#### Acceptance Criteria

1. THE runtime SHALL write each `[[dependencies]]` table without indentation, as
   `github.com/dagger/sdk-helpers` at `v1.0.12` writes manifests (`main.dang`):
   `name`, then `source`, then any other settings a kept dependency has.
2. THE runtime SHALL place the dependency tables after `[runtime]`.
3. THE runtime SHALL keep everything outside the dependency tables byte for byte.
4. WHEN generation runs again with the same Client_Targets, THE manifest SHALL not
   change.
5. WHEN the runtime scaffolds a new module, ITS manifest SHALL have the `[runtime]`
   table in the same unindented form.

### Requirement 4: Package Scopes

**User Story:** As a Rust application author, I want each module I add to become a
Rust module in my package, so that I can use several Dagger modules from one program.

#### Acceptance Criteria

1. WHEN a Package_Scope is generated, THE runtime SHALL write one Client_Module per
   Client_Target under `src/dagger_clients/` of the Client_Root.
2. THE Client_Module's name SHALL be the target's `moduleOriginalName` as a Rust
   module identifier, so `engine-dev` becomes `engine_dev`.
3. THE runtime SHALL write `src/dagger_clients/mod.rs`, which declares the
   Client_Modules, one per line and sorted, and holds no other item.
4. THE runtime SHALL never edit a file outside `src/dagger_clients/` other than the
   package's `Cargo.toml`.
5. THE author SHALL declare `mod dagger_clients;`; generation SHALL not add it.
6. A Git target's address SHALL be its `asString` and its pin SHALL be its recorded
   `pin`, unless the target is part of the Git workspace: in the workspace's
   repository, at the workspace's commit and selector. Such a target SHALL be
   addressed as a local target (`workspaceModule` in
   `cmd/dagger-go-sdk-codegen/main.go` of `github.com/dagger/go-sdk`).
7. A local target's address SHALL be `/` followed by its `sourceRootSubpath`, with an
   empty pin.
8. WHEN a Package_Scope has no Client_Targets and no Ownership_Records, THE runtime
    SHALL change nothing.
9. WHEN a Package_Scope has Client_Targets and its Client_Root holds `dagger.json`,
   or a `dagger.json` in a directory above it, as far as the workspace root, names
   the Client_Root as its module's `source`, THE runtime SHALL refuse it before
   writing anything. The refusal SHALL give the advice of Requirement 2.15 with
   `dagger module client add` in place of `dagger generate`, run again from the
   directory that holds `dagger.json`.

### Requirement 5: Target Schemas and Naming

**User Story:** As a Rust application author, I want a client to use the names the
engine serves, so that every generated call resolves.

#### Acceptance Criteria

1. THE runtime SHALL read a Client_Target only through
   `clientSchemaIntrospectionJson`, with `clientVersion` set to the Target_Release.
2. THE runtime SHALL never call `withEngineVersion` on a Client_Target.
3. THE runtime SHALL fetch each target's Client_Schema separately.
4. THE runtime SHALL verify each schema's digest before rendering from it.
5. THE runtime SHALL render each Client_Module from its own schema only.
6. Field selections, inline fragments, and `node(id:)` type names in a Client_Module
   SHALL use exactly the wire names its Client_Schema carries.
7. WHEN a target declares a type whose name has an acronym, such as `HTTPFetcher` in
   a module `naming-app`, THE bindings SHALL select `NamingAppHTTPFetcher` as the
   engine serves it.

### Requirement 6: Generated Client API

**User Story:** As a Rust application author, I want a generated client to read like
the dependency bindings in a module, so that I learn one calling convention.

#### Acceptance Criteria

1. EACH Client_Module SHALL declare `pub struct Client` that is `Clone`.
2. `Client::new(&<sdk>::Client)` SHALL build the client without any request.
3. `Client` SHALL offer the bound module's constructor fields and no Core field.
4. WHEN a constructor has optional arguments, THE Client_Module SHALL offer a
   `<field>_opts` method and a `Query<Field>Opts` struct, as Feature 6 renders them.
5. `Client` SHALL offer `r#ref` and `load` for every type that implements the
   module's `ClientLoadable`.
6. `ClientLoadable` SHALL be implemented for every Core handle and for every handle
   the bound module owns.
7. THE Client_Module SHALL declare one file per type the bound module owns, and SHALL
   make each type reachable as `<name>::<Type>`.
8. THE Client_Module SHALL render no Core type and SHALL refer to Core through the
   package's SDK dependency key.
9. Type, field, argument, enum, and option names SHALL be those Feature 6's
   dependency renderer produces for the same schema.
10. THE first line of every generated file SHALL begin with the Generated_Marker.
11. Generated files SHALL be formatted by the SDK's rustfmt before they are written
    and SHALL compile with warnings denied.

### Requirement 7: The Author's Checks

**User Story:** As a Rust application author, I want generated clients to pass my own
formatting and lint checks, so that adding a client never breaks my build.

#### Acceptance Criteria

1. WHEN the author runs `cargo fmt --check` from the package root, THE generated
   clients SHALL pass, whatever the author's rustfmt configuration or edition.
2. WHEN the author runs `cargo clippy --all-targets -- -D warnings` from the package
   root, THE generated clients SHALL pass.
3. BOTH checks SHALL pass with no configuration, attribute, or exclusion added by
   the author.
4. BOTH checks SHALL pass in a binary crate that uses only part of one client.
5. EACH Client_Module's `mod.rs` SHALL allow, for the whole client, dead code, the
   lints that Feature 6's dependency bindings allow, and `clippy::module_inception`,
   and SHALL allow `unused_imports` on each re-export of a type file.
6. `src/dagger_clients/mod.rs` SHALL declare each Client_Module as
   `#[rustfmt::skip] pub mod <name>;`.
7. `src/dagger_clients/mod.rs` SHALL itself be stable under rustfmt's defaults.

### Requirement 8: Run-Time Module Loading

**User Story:** As a Rust application author, I want a client to load its module by
itself, so that I do not write loading code.

#### Acceptance Criteria

1. EVERY request built from a `Client`, or from any handle reached from it, Core
   handles included, SHALL carry the client's Module_Requirement.
2. BEFORE such a request is sent, THE session SHALL serve the module with
   `serveModule(address, refPin)`, once per live session and Module_Requirement.
3. THE session SHALL send `refPin` always, as an empty string for a local target.
4. WHEN several requests need the same load, THEY SHALL wait for the one load.
5. WHEN a load fails, THE session SHALL report the failure to the waiting requests and
   SHALL try again on the next request.
6. WHEN a waiting request is dropped, THE load SHALL continue for the others.
7. WHEN the request driving a load is dropped, A later request SHALL start the load
   again.
8. WHEN the session is closed, THE request SHALL fail as any request after close,
   without sending `serveModule`.
9. A request that sends nothing SHALL not load the module.
10. Requests that carry no Module_Requirement SHALL behave as before.
11. IF the engine's schema lacks `serveModule`, THEN THE error SHALL read
    `the engine does not support generated module clients (serveModule is required)`.
12. IF a load fails for any other reason, THEN THE error SHALL read
    `load module "<address>"`.
13. IF a request fails after the load because the schema lacks a field, THEN THE error
    SHALL read `generated client for "<address>" is out of date; regenerate it`.
14. THE schema lacks a field only when a GraphQL error has the extension code
    `GRAPHQL_VALIDATION_FAILED`, no path, and a message that starts
    `Cannot query field `.
15. EACH of these errors SHALL keep its cause as `source()`, and SHALL not repeat the
    cause in its message.

### Requirement 9: Ownership and Regeneration

**User Story:** As a Rust application author, I want generation to respect my edits,
so that regenerating never destroys work.

#### Acceptance Criteria

1. EACH Client_Module directory SHALL hold an Ownership_Record for its files.
2. `src/dagger_clients/` SHALL hold an Ownership_Record for `mod.rs`.
3. AN Ownership_Record SHALL be
   `{"version": 1, "files": {"<relative path>": "<sha256 hex>"}}` with sorted keys,
   two-space indentation, and a trailing newline, and SHALL not list itself.
4. THE digests SHALL be those of the final, formatted bytes.
5. WHEN an existing file's bytes equal the new bytes, THE runtime SHALL accept it.
6. WHEN an existing file differs, THE runtime SHALL overwrite it only if its first
   line begins with the Generated_Marker and either is absent from the old record or still has
   its recorded digest.
7. IF an existing file may not be overwritten, THEN generation SHALL fail with
   `<path> contains user changes; refusing to overwrite it`.
8. WHEN a recorded file is no longer generated, THE runtime SHALL remove it only if
   its first line begins with the Generated_Marker and it still has its recorded
   digest.
9. WHEN a directory under `src/dagger_clients/` has an Ownership_Record and is no
   longer a Client_Module, THE runtime SHALL prune it by the same rule and SHALL
   rewrite its record with no files.
10. WHEN the client set becomes empty, THE runtime SHALL treat
    `src/dagger_clients/mod.rs` by the same rule.
11. THE runtime SHALL validate every write and removal of the scope before making
    any.
12. IF a generated path or a record's directory crosses a symbolic link, THEN
    generation SHALL fail.
13. IF an Ownership_Record is malformed or has an unsupported version, THEN
    generation SHALL fail.
14. Repeated generation with the same inputs SHALL produce the same bytes.

### Requirement 10: Cargo Manifest

**User Story:** As a Rust application author, I want generation to set the SDK
dependency my clients need, so that the package builds without guesswork.

#### Acceptance Criteria

1. WHEN at least one Client_Module is generated, THE package's `Cargo.toml` SHALL
   depend on `tokeira-dagger-sdk` at exactly the SDK_Dependency.
2. WHEN `[dependencies]` already has an entry for `tokeira-dagger-sdk`, by key or by
   `package`, THE runtime SHALL use that key.
3. WHEN it has none, THE runtime SHALL add
   `dagger-sdk = { package = "tokeira-dagger-sdk", … }`.
4. THE runtime SHALL replace only the entry's source and requirement fields and SHALL
   keep its other fields, such as `features` and `default-features`.
5. THE runtime SHALL keep every other part of the manifest, comments included.
6. WHEN the entry is `{ workspace = true }` and the Cargo workspace's
   `[workspace.dependencies]` entry already states the SDK_Dependency, THE runtime
   SHALL accept it unchanged.
7. IF the inherited entry differs, THEN generation SHALL fail and SHALL state the
   exact entry to write in the workspace manifest.
8. IF the Client_Root's `Cargo.toml` has no `[package]`, THEN generation SHALL fail.
9. THE runtime SHALL never read, write, or resolve `Cargo.lock`.
10. WHEN no Client_Module is generated, THE runtime SHALL leave `Cargo.toml` alone.

### Requirement 11: Refusals

**User Story:** As a Rust application author, I want generation to refuse what it
cannot do correctly, so that I never get half a client.

#### Acceptance Criteria

1. IF two Client_Targets of one scope have the same Client_Module name, THEN
   generation SHALL fail and SHALL name both.
2. IF a bound module owns a type named `Client` or `ClientLoadable`, THEN generation
   SHALL fail and SHALL name the schema coordinate.
3. IF a constructor would be named `new`, `ref`, or `load`, THEN generation SHALL fail
   and SHALL name the schema coordinate.
4. IF a Client_Target is neither a local nor a Git module, THEN generation SHALL fail.
5. EVERY refusal of Feature 6's dependency renderer SHALL apply to a Client_Module.
6. WHEN generation fails for any client of a scope, THE runtime SHALL write nothing
   for that scope.
7. EVERY refusal SHALL be a typed diagnostic with a stable code.
8. Diagnostics SHALL carry no credential, session token, or host path.
9. IF a Client_Module would be named `mod`, THEN generation SHALL fail, since Rust
   would find its file both at `src/dagger_clients/mod.rs` and at
   `src/dagger_clients/mod/mod.rs`.

### Requirement 12: Staleness and Refresh

**User Story:** As a Rust application author, I want a clear way to bring a client up
to date, so that a changed module does not leave me with silent breakage.

#### Acceptance Criteria

1. WHEN `dagger module client update` moves a Git target to a new commit, THE
   regenerated Client_Module SHALL embed the new pin.
2. WHEN `dagger generate` or `dagger lock update` runs, THE runtime SHALL regenerate
   the scope from the current Client_Schemas.
3. WHEN an unedited generated file has been deleted, regeneration SHALL restore it.
4. WHEN `dagger module client rm` removes a target, regeneration SHALL prune its
   Client_Module as Requirement 9 states.
5. WHEN a client calls a field its target no longer has, THE request SHALL fail with
   the out-of-date error of Requirement 8.
6. THE runtime SHALL keep no staleness state of its own beyond the Ownership_Records.

### Requirement 13: Removal of the Legacy Path

**User Story:** As an SDK maintainer, I want one client-generation path, so that no
unreachable generator has to be kept correct.

#### Acceptance Criteria

1. THE runtime SHALL no longer expose `generateClient` or
   `requiredClientGenerationFiles`.
2. THE repository SHALL no longer contain the standalone-client compiler and
   renderer, the client project reconciler, the client initializer, the client-set
   planner, or their request kinds.
3. THE packaged SDK content SHALL no longer contain `client-generation.json`.
4. THE tests and fixtures of the removed code SHALL be removed with it.
5. A type that remaining code still uses SHALL move to a module that outlives the
   removal.
6. THE runtime's content inputs SHALL still list every input the remaining operations
   need.

### Requirement 14: Verification

**User Story:** As an SDK maintainer, I want each part proven where it is cheapest, so
that the engine-backed acceptance stays small.

#### Acceptance Criteria

1. THE transport SHALL have engine-free tests for each case
   `sdk/go/module_client_test.go` covers: shared loads, retry after failure, a
   cancelled wait, lazy construction, and missing-field classification.
2. THE renderer SHALL have a checked Client_Schema fixture fetched from an engine at
   the Target_Release.
3. A compile fixture SHALL build a generated client, with a caller that uses the
   constructor with and without options, `r#ref`, and `load`, and a type the target
   owns that no function returns, with warnings denied and without an engine.
4. Ownership, the Cargo manifest rules, the module manifest rules, and each refusal
   SHALL have engine-free unit tests.
5. THE Go adapter SHALL have tests for its request documents and its address rule,
   and a source test that no client path calls `WithEngineVersion`.
6. A checked Client_Schema of a target with an acronym in a type name SHALL prove
   that the bindings use the served wire names.
7. A test SHALL build a binary package that uses part of a generated client and
   SHALL run the two commands of Requirement 7 from its root.
8. THE acceptance SHALL generate a Cargo package with one local and one Git client
   and SHALL run it against an engine at the Target_Release.
9. THE acceptance SHALL cover `client update`, `dagger generate` restoring a deleted
   file, `client rm` pruning, and an edited file refused with nothing written.
10. THE acceptance SHALL give a module its dependency through
    `dagger module client add`.
11. Client targets in the acceptance SHALL be Dang modules.

## Out of Scope

- Go's module layout with a Dang entrypoint and shared Core types in modules.
- Package clients generated beside a module in a Module_Scope.
- SDK settings for a scope, including Go's `lock` setting that writes dependency pins.
- A process-wide default session; a client always receives a `Client`.
- Retargeting to a later engine release.
- Resolving or updating `Cargo.lock`.
- Changing Feature 6's dependency bindings, or its module generation beyond the
  ownership of the files it generates.

## Iteration and Feedback Notes

- The earlier contract bound one module per generated Cargo project and exposed it
  through an extension of the Core client. The engine at the Target_Release has no
  command that reaches that path, so it is replaced rather than extended.
- Rust passes `clientVersion` when it asks for a Client_Schema and Go does not
  (`generateStandaloneClient` in `go-sdk.dang`). Rust's clients use the SDK's Core
  types, which are generated for the Target_Release, so the schema's view of Core must
  match them.
- Go runs `go mod tidy` after setting its library requirement. Rust leaves
  `Cargo.lock` to the author's next Cargo command.
- Go's entry point is `Connect`, which may use a default session. Rust's is `new`
  because it does no I/O and always receives the session.
- Forcing a target's engine version renames its declarations by that version's casing
  rules, so a client would say `NamingAppHttpFetcher` where the engine serves
  `NamingAppHTTPFetcher`. `clientVersion` describes Core at the client's view and
  leaves the module at the version it declares (`clientSchemaIntrospectionJSONFile`
  in `core/schema/modulesource.go`).
- In a binary crate `mod dagger_clients;` is private, so every generated item the
  author does not call is dead code, and a type file's re-export is an unused import
  when the author names none of its types. A module may own a type that none of its
  functions returns, which nothing else names. The allowances of Requirement 7 exist
  for these cases.
