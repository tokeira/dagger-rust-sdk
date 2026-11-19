# Dagger Rust engine integration

`dagger-sdk-engine` is the private Rust operation tool packaged with the Dagger
engine. It owns canonical operation models, immutable Cargo project adoption,
generated-file ownership, confined execution, and failure-atomic operation
publication. Runtime verification and the thin engine adapter are built in subsequent
integration work.

The packaged `dagger-rust-engine execute` command accepts only a canonical operation
request, exact visible schema, immutable engine descriptor, and explicit project root.
It invokes no caller-selected command or shell. An existing file is replaced only
while a compatible operation manifest records it with its current digest, or, when the
manifest does not record it, while it begins with `// @generated`, and that manifest
becomes visible only after every artifact and bounded post-work action succeeds.

This crate is repository tooling rather than an application dependency. It is not
published, and generated user projects depend only on the public `dagger-sdk` crate.

The crate also owns the typed local-checkpoint planner. Its validated form can contain
only Rust SDK package actions, direct Rust-owned Go ABI tests, generated ownership,
security, package, and clean-output checks. Engine, Dagger, network-graph, unrelated
SDK, unscoped-generation, and distribution proposals are rejected before they can be
rendered as executable work. See [`../../docs/module-authoring.md`](../../docs/module-authoring.md)
for the boundary between this local record and completed-engine verification.

For module clients, this crate also owns two operations. One writes a Cargo package's
clients under `src/dagger_clients/`, with the package's exact SDK dependency and the
ownership records that decide which files generation may replace or remove. The other
rewrites a module's `[[dependencies]]` from the clients recorded for it. Each validates
its whole request before the first write. The consumer and contributor contract is in
[`../../docs/client-generation.md`](../../docs/client-generation.md).

Licensed under Apache-2.0 as part of the Dagger repository.
