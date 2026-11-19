# dagger-codegen

`dagger-codegen` is the schema compiler used to build the generated client surface of
the [Rust SDK for Dagger](https://github.com/tokeira/dagger-rust-sdk). It is
development tooling for the SDK rather than an application-facing client library.
Application authors should depend on `tokeira-dagger-sdk`, as the
[SDK README](../dagger-sdk/README.md) describes.

The crate owns the pure part of generation:

- validating the reviewed Dagger, engine, schema, Rust SDK, and toolchain identities;
- checking that bounded introspection bytes match the target's SHA-256 digest;
- compiling raw introspection into an ordered canonical schema with exact GraphQL
  wire names, recursive nullability, defaults, directives, documentation, and source
  coordinates;
- projecting that schema into collision-checked Rust 2024 names, wrapper-correct
  types, required/omittable argument plans, typed directive and enum-alias policies,
  and one total field execution strategy per coordinate;
- cataloging 1,661 exact-target semantic bindings, including explicit no-symbol
  containment for target-private metadata, with domain-separated implementation
  fingerprints;
- producing deterministic in-memory candidate artifacts and structured diagnostics.

The module-client renderer is a third surface of the dependency renderer, beside the
library and the module. From one module's client schema it renders that module's typed
client for a Cargo package: a `Client` with the module's constructor, `r#ref`, and
`load`, and a file for each type the module owns. It renders nothing for Core, whose
types the bindings name through the package's SDK dependency.

It deliberately has no filesystem, process, network, engine-session, or publication
authority. Repository input discovery, formatting, comparison,
and transactional publication belong to `dagger-bootstrap`; runtime query execution
belongs to `dagger-sdk`.

## Correctness contract

Generation is fail-closed. A target identity, schema digest, reference, wrapper,
default, directive, or exact coordinate-inventory mismatch produces sorted typed
diagnostics and no projection plan. Rendering accepts only an immutable validated
plan through the exact-target `render_core` boundary, so raw transport data cannot be
reinterpreted by that pipeline's templates.

Canonical maps and sets are ordered by their exact GraphQL wire identities. Equivalent
introspection input order therefore produces byte-identical candidate output. The
crate performs no repository writes; callers must complete all validation, formatting,
and ownership checks before publishing generated files.

## Development status

`dagger-codegen` is workspace-private while its API and the generated Rust client
surface are being stabilized. Dagger's repository tooling consumes it through an exact
workspace path rather than as an independently supported package. If a future external
generator requires this crate, its public contract and publication policy must be
reviewed before enabling publication. It is not a supported replacement for
`dagger-sdk`.

Contributors should read the Rust SDK
[`CONTRIBUTING.md`](https://github.com/tokeira/dagger-rust-sdk/blob/main/CONTRIBUTING.md)
and
[`docs/architecture.md`](https://github.com/tokeira/dagger-rust-sdk/blob/main/docs/architecture.md)
before changing schema, projection, rendering, or publication boundaries.
The end-to-end module-client ownership and checkpoint contract is documented in
[`docs/client-generation.md`](https://github.com/tokeira/dagger-rust-sdk/blob/main/docs/client-generation.md).
