# Rust SDK for Dagger

Rust SDK for [Dagger](https://dagger.io), maintained by Tokeira. Not affiliated with
Dagger, Inc.

The SDK provides an owned, asynchronous client for the Dagger GraphQL API. Its
default connector can reuse a session created by `dagger run` or provision and launch
the exact Dagger CLI release compiled into the SDK.

The client shares one connection and one close result across generated handles, raw
queries, compositional queries, and clones. Implicit HTTP is authenticated and
loopback-only; downloaded CLI bytes are checksum-verified before use.

## Install

Add the SDK and an async runtime to `Cargo.toml`:

```toml
[dependencies]
tokeira-dagger-sdk = "0.3.0"
tokio = { version = "1.35.1", features = ["macros", "rt-multi-thread"] }
```

Code names the crate `dagger_sdk` whichever package name `Cargo.toml` uses. The SDK pins
its procedural-macro companion, `tokeira-dagger-sdk-macros`, to the exact matching
version, so applications never depend on it directly.

Commit an application lockfile under the application's own reproducibility policy.

## Use

A CI pipeline is an ordinary Rust program. This one gates a change on formatting, lints,
and tests, running the checks concurrently with Cargo's caches kept between runs:

```rust
use std::process::ExitCode;

use dagger_sdk::{Container, HostDirectoryOpts, Query, QueryError};

/// Every check runs on one toolchain, pinned like the project's `rust-toolchain.toml`.
const RUST_IMAGE: &str = "rust:1.99.1-bookworm";

// The gate a change must pass. Each check is one Cargo subcommand with a target
// directory of its own, so the checks run concurrently without waiting on Cargo's
// build lock.
const FMT: &[&str] = &["fmt", "--all", "--check"];
const CLIPPY: &[&str] = &["clippy", "--all-targets", "--locked", "--", "-Dwarnings"];
const TEST: &[&str] = &["test", "--locked"];

#[tokio::main]
async fn main() -> Result<ExitCode, Box<dyn std::error::Error>> {
    let project = std::env::args().nth(1).unwrap_or_else(|| ".".to_owned());
    let client = dagger_sdk::connect().await?;
    let workspace = rust_workspace(&client.query(), &project);

    // The first failing check ends the run.
    let checks = tokio::try_join!(
        cargo(&workspace, FMT),
        cargo(&workspace, CLIPPY),
        cargo(&workspace, TEST),
    );
    client.close().await?;

    match checks {
        Ok(_) => Ok(ExitCode::SUCCESS),
        Err(QueryError::Exec { error, .. }) => {
            // A failed check fails the run with the tool's own output, as a CI log should.
            eprintln!("{} failed:", error.command().unwrap_or_default().join(" "));
            eprintln!("{}", error.stdout().unwrap_or_default());
            eprintln!("{}", error.stderr().unwrap_or_default());
            Ok(ExitCode::FAILURE)
        }
        Err(error) => Err(error.into()),
    }
}

/// The project on the pinned toolchain, with Cargo's downloads and builds cached
/// between runs.
fn rust_workspace(client: &Query, project: &str) -> Container {
    let source = client.host().directory_opts(
        project,
        &HostDirectoryOpts::default().with_exclude(vec!["target/"]),
    );
    let registry = client.cache_volume("cargo-registry");
    let targets = client.cache_volume("cargo-target");
    client
        .container()
        .from(RUST_IMAGE)
        .with_exec(vec!["rustup", "component", "add", "clippy", "rustfmt"])
        .with_mounted_cache(registry, "/usr/local/cargo/registry")
        .with_mounted_cache(targets, "/cargo-target")
        .with_mounted_directory("/src", source)
        .with_workdir("/src")
}

/// Runs one Cargo subcommand in the workspace, in a target directory of its own.
async fn cargo(workspace: &Container, args: &[&str]) -> Result<Container, QueryError> {
    let mut command = vec!["cargo"];
    command.extend_from_slice(args);
    workspace
        .with_env_variable("CARGO_TARGET_DIR", format!("/cargo-target/{}", args[0]))
        .with_exec(command)
        .sync()
        .await
}
```

Run the program normally and the SDK will establish an implicit Dagger session:

```bash
cargo run
```

Run it through Dagger to reuse an existing session:

```bash
dagger run cargo run
```

See the crate documentation for raw GraphQL, typed execution errors, diagnostics,
trace propagation, compatibility policy, injected connections, and shutdown details.

Module authors use the `object`, `interface`, `enum_type`, `scalar`, and `methods`
attributes re-exported from this crate. Their exact-version procedural implementation
lives in `tokeira-dagger-sdk-macros`; applications should not add that companion directly.
The authoring syntax, state/dispatch model, and current verification boundary are
documented in [`../../docs/module-authoring.md`](../../docs/module-authoring.md).

Generated standalone clients also reuse this crate's owned lifecycle, Core bindings,
transport, errors, and IDs by identity. Their `dagger_client` namespace adds one bound
module without copying Core or merging dependency modules. See
[`../../docs/client-generation.md`](../../docs/client-generation.md).

## Features

| Cargo selection | Public surface | Intended use |
| --- | --- | --- |
| default | handwritten client plus generated core-schema bindings | normal SDK applications |
| `--features gen` | same generated surface, when defaults were disabled explicitly | selective workspace configurations |
| `--no-default-features` | owned client, raw GraphQL, diagnostics, errors, and scalar values | raw queries or smaller integrations that do not need typed core bindings |
| `--all-features` | the complete supported SDK surface | CI and release verification |

The generated types are re-exported at the crate root only when `gen` is enabled.
Disabling it never changes the raw request/response or client lifecycle contracts.

## Examples and contributing

Small examples are packaged with the crate. From the repository root, run the CI checks
above against any Cargo project with
`cargo run -p tokeira-dagger-sdk --example ci-checks -- path/to/project`. Standalone
application examples live in [`../../examples`](../../examples).

Development and verification commands are documented in
[`../../CONTRIBUTING.md`](../../CONTRIBUTING.md); generated-client maintenance and
release assembly are documented in [`../../docs/maintaining.md`](../../docs/maintaining.md).

## License

Licensed under the [Apache License, Version 2.0](LICENSE). The SDK derives from the Rust
SDK in the [Dagger repository](https://github.com/dagger/dagger); [`NOTICE`](NOTICE) carries
its attribution.
