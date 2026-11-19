# Implementation Plan

Every task is open. Tasks 1 to 7 are the delivery steps, in order. Each is one commit,
except that sub-tasks 4.3 and 4.4 are commits of their own after task 5; every commit
builds and passes the repository's gates on its own.

- [ ] 1. Add module loading to `tokeira-dagger-sdk`
  - [ ] 1.1 Carry a module requirement on generated builders and handles
    - Add the doc-hidden `Client::generated_module_query_builder(address, pin)`,
      which performs no I/O.
    - Keep the requirement on the session handle that builders and handles clone,
      so `select`, the generated query root, Core handles, and re-entry builders
      inherit it; leave `Client::query()` without one.
    - _Requirements: 8.1, 8.10_
  - [ ] 1.2 Serve the module once per session and requirement
    - At the single point where generated requests reach the session, await the
      load: start `serveModule(address, refPin)` when absent, wait when present.
    - Share one load among concurrent requests, clear a failed load, keep a load
      alive when a waiter is dropped, restart it when its driver is dropped, and
      fail a closed session without sending anything.
    - Skip requests that send nothing.
    - _Requirements: 8.2-8.9_
  - [ ] 1.3 Add the three errors with Go's wording
    - Classify a missing field as `missingFieldError` does in
      `sdk/go/module_client.go`, including in an HTTP error response.
    - Add the unsupported-engine, load-failure, and out-of-date cases to
      `QueryError`, each with its cause as `source()`.
    - _Requirements: 8.11-8.15, 12.5_
  - [ ] 1.4 Test the transport without an engine
    - Mirror each test of `sdk/go/module_client_test.go` over the SDK's fake
      connection.
    - Property 5 and Property 6.
    - _Requirements: 14.1_

- [ ] 2. Add the client surface to the dependency renderer
  - [ ] 2.1 Check in client schemas from the target release
    - Fetch the greeter module's client schema with
      `clientSchemaIntrospectionJson(clientVersion:)` and confirm its types carry
      the source-map directive the dependency renderer selects by.
    - Fetch the schema of a Dang module with an acronym in a type name.
    - _Requirements: 5.1, 14.2, 14.6_
  - [ ] 2.2 Render a client module
    - Add `BindingSurface::Client`: `mod.rs` with `Client`, `new`, `r#ref`, `load`,
      and `ClientLoadable`; one file per owned type; constructors as
      `impl super::Client` with `_opts` and `Query<Field>Opts`.
    - Begin every file with `// @generated`; name Core through the SDK alias; keep
      wire names unchanged in selections, inline fragments, and `node(id:)`.
    - Allow, in `mod.rs`, what generated code cannot avoid, and `unused_imports` on
      each re-export.
    - _Requirements: 5.5-5.7, 6.1-6.10, 7.5_
  - [ ] 2.3 Refuse reserved names
    - Refuse a target type named `Client` or `ClientLoadable` and a constructor
      named `new`, `ref`, or `load`, with a diagnostic at the schema coordinate.
    - Keep every refusal of the dependency renderer.
    - _Requirements: 11.2, 11.3, 11.5, 11.7, 11.8_
  - [ ] 2.4 Add the compile fixture
    - Render the greeter client into a test crate with a caller that uses the
      constructor with and without options, `r#ref`, and `load`, and never names a
      type of the greeter's that no function returns; compile with warnings denied.
    - Assert the acronym fixture's bindings use the served wire names.
    - Property 3 and Property 4.
    - _Requirements: 6.11, 7.4, 7.5, 14.3, 14.6_

- [ ] 3. Generate package clients
  - [ ] 3.1 Add the `generate-package-clients` request
    - Add the request and its result, with `touched_paths` and `removed_paths`;
      let the operation tool locate one schema per client and verify each digest.
    - Derive each client's Rust module name; refuse two clients with one name, and
      a client whose module would be `mod`.
    - _Requirements: 4.1, 4.2, 5.3, 5.4, 11.1, 11.6, 11.7, 11.9_
  - [ ] 3.2 Render the aggregator
    - Write `src/dagger_clients/mod.rs`: the marker, its documentation line, and a
      sorted `#[rustfmt::skip] pub mod <name>;` per client, with no allowance of its
      own.
    - _Requirements: 4.3, 7.6, 7.7_
  - [ ] 3.3 Implement ownership and pruning
    - Write and read `.dagger-generated.json` per client directory and for the
      aggregator, in Go's format.
    - Apply Go's overwrite and prune rules, prune directories that are no longer
      clients, validate the whole scope first, and refuse symbolic links and
      malformed records.
    - Hash the formatted bytes.
    - Property 1 and Property 7.
    - _Requirements: 4.4, 4.8, 9.1-9.14, 12.6_
  - [ ] 3.4 Edit the Cargo manifest
    - Set the exact SDK dependency under the existing key or `dagger-sdk`, keep
      other fields, accept a matching inherited entry, and refuse a differing one
      and a manifest without `[package]`.
    - Leave `Cargo.lock` and, with no clients, `Cargo.toml` alone.
    - Property 8.
    - _Requirements: 10.1-10.10_
  - [ ] 3.5 Wire the Go adapter for package scopes
    - Stop refusing a package scope; build the request from each client's name,
      address, pin, and client schema read with `ClientVersion` set to the target
      release; refuse a target that is neither local nor Git; bind a Git target in
      the Git workspace's repository, commit and selector by its path, as Go does.
    - Mount the scope's manifests, `src/dagger_clients/**`, and the schemas; apply
      touched and removed paths to the workspace.
    - Test the request document and the address rule, and add the source test that
      no client path calls `WithEngineVersion`.
    - _Requirements: 1.1, 1.2, 1.4-1.9, 4.6, 4.7, 5.1, 5.2, 11.4, 14.5_
  - [ ] 3.6 Test the author's checks
    - Build a binary crate with `mod dagger_clients;` that uses part of one client;
      run `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings` from
      its root.
    - Property 9.
    - _Requirements: 7.1-7.4, 14.4, 14.7_

- [ ] 4. Generate a module scope's clients as its dependencies
  - [ ] 4.1 Add the `write-module-dependencies` request
    - Rewrite `[[dependencies]]` by the helper's rules: relative path or
      `asString`, no pin, kept name and settings, an unnamed matching entry given
      the target's name, unmatched entries removed, ordered by name ascending.
    - Write a Git target that is the workspace's own module as a local, relative
      path, as the helper's `inWorkspace` decides it.
    - Write the tables unindented after `[runtime]`, keep every other byte, and
      change nothing on a second run.
    - Refuse a module configured by `dagger.json` that has dependencies or clients,
      with advice to run `dagger module migrate` on its path from the workspace
      root, then `dagger generate`.
    - Refuse a package's clients the same way when the package is a module on
      `dagger.json`, with advice that ends in `dagger module client add` again from
      the directory that holds `dagger.json`.
    - Property 2.
    - _Requirements: 2.1-2.12, 2.15, 3.1-3.4, 4.9, 14.4_
  - [ ] 4.2 Wire the Go adapter for module scopes
    - Stop refusing clients in a module scope; run the request for an existing and
      for a newly scaffolded manifest, then regenerate through the engine.
    - Make the helper's `inWorkspace` lookup for each Git client and pass the
      resulting kind in the request; treat an error as not the workspace's own.
    - Write a new module's `[runtime]` table unindented.
    - _Requirements: 1.3, 2.4, 2.13, 2.14, 3.5_
  - [ ] 4.3 Apply Go's ownership rule to a module's generated files
    - Make `verify_ownership` apply `ownedOverlay` with the previous operation
      manifest as the record: write a missing file; replace a recorded file only
      while it has its recorded digest, and an unrecorded one only when it begins
      with the marker; refuse any other before anything is written.
    - Remove a recorded file that is no longer generated only while it has its
      recorded digest; keep an edited one and leave it out of the new manifest.
    - Test each case through the generation stack, including a deleted binding
      written again and an edited binding of a removed dependency kept.
    - _Requirements: 2.16_
  - [ ] 4.4 Carry a module's removals into the workspace
    - Name the removed files in the generation result, and accept them in the Go
      decoder.
    - In `Codegen`, list them from the module's source root in
      `.dagger/rust/removed-paths.json` in the returned code when there are any; in
      `GenerateScope`, take the list out, write the code, and remove each listed
      file with `WithoutFile`.
    - Refuse a list that is not one of normalized paths inside the module, or that
      names no file or itself.
    - _Requirements: 1.9, 2.17_

- [ ] 5. Remove the legacy path
  - [ ] 5.1 Remove the Go entry points and metadata
    - Remove `GenerateClient`, `RequiredClientGenerationFiles`, the
      client-generation metadata and its sealing, and the client-set plan, with
      their tests.
    - _Requirements: 13.1, 13.4_
  - [ ] 5.2 Remove the Rust generator
    - Remove the standalone-client compiler and renderer, the project reconciler,
      the initializer, the planner, their request kinds, the runner's client
      paths, `client-generation.json`, and their tests and fixtures.
    - Move any type the remaining code uses before deleting its module.
    - _Requirements: 13.2-13.5_
  - [ ] 5.3 Re-check the runtime's content inputs
    - Confirm the `include` list and the sealing filter still cover every input of
      the remaining operations.
    - _Requirements: 13.6_

- [ ] 6. Extend the acceptance
  - [ ] 6.1 Make the greeter fixture a module directory
    - Keep it a Dang module so it can be a Git target at the pushed commit.
    - _Requirements: 14.11_
  - [ ] 6.2 Replace the client refusal with a package that runs
    - Generate a Cargo package with a local and a Git client and check both
      answers against the engine.
    - Cover `client update`, `dagger generate` restoring a deleted file,
      `client rm` pruning, and an edited file refused with nothing written.
    - _Requirements: 12.1-12.4, 14.8, 14.9_
  - [ ] 6.3 Record every fixture's dependencies as clients
    - Give `gate` its `greeter` dependency through `dagger module client add`, and
      record the clients of every other fixture with hand-written dependencies.
    - _Requirements: 1.8, 14.10_

- [ ] 7. Document the feature and add its changelog entries
  - [ ] 7.1 Rewrite the client guide
    - Rewrite `docs/client-generation.md` for `dagger module client add`, with
      `mod dagger_clients;` as its first step.
    - _Requirements: 4.5, 12.1-12.6_
  - [ ] 7.2 Update the module guide and references
    - Use `dagger module client add` in the Dependencies section of
      `docs/module-authoring.md`, with the step for existing hand-written
      dependencies.
    - Update `README.md`, `docs/architecture.md`, `docs/engine-integration.md`,
      and `docs/maintaining.md`.
    - _Requirements: 2.1, 2.9_
  - [ ] 7.3 Add the changelog entries
    - Added: module clients in a Cargo package. Added: a module's dependencies
      come from its clients. Changed: hand-written dependencies are replaced by
      the recorded clients. Removed: the legacy client generator.
    - _Requirements: 2.9, 13.1_

## Notes

- The Go adapter stays data only. Naming, rendering, ownership, and manifest policy
  are tested in Rust.
- Every engine-free task is proven without an engine. Only Task 6 needs one, and it
  needs the commit pushed, because its Git client names that commit.
- Property numbers refer to the design document.
