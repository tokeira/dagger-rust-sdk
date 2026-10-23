# Rust SDK for Dagger

[![Build status](https://badge.buildkite.com/1c19045900aa4fa682a5abe778c6aacd38caa9770ca8b7a90b.svg?branch=main)](https://buildkite.com/tokeira/dagger-rust-sdk)
[![crates.io](https://img.shields.io/crates/v/tokeira-dagger-sdk)](https://crates.io/crates/tokeira-dagger-sdk)
[![docs.rs](https://img.shields.io/docsrs/tokeira-dagger-sdk)](https://docs.rs/tokeira-dagger-sdk)
[![License: Apache-2.0](https://img.shields.io/badge/license-Apache--2.0-blue)](LICENSE)
[![MSRV 1.99.1](https://img.shields.io/badge/MSRV-1.99.1-orange?logo=rust)](Cargo.toml)
[![Dagger v1.0.0-beta.21](https://img.shields.io/badge/Dagger-v1.0.0--beta.21-informational)](https://github.com/dagger/dagger/tree/v1.0.0-beta.21)
[![Built with Codex & Claude](https://img.shields.io/badge/built%20with-Codex%20%26%20Claude-blueviolet)](#built-with-claude)

Rust SDK for [Dagger](https://dagger.io), maintained by Tokeira. Not affiliated with
Dagger, Inc.

This workspace contains the SDK, its code generator, module-authoring runtime,
standalone-client generator, and engine integration. The current SDK version is `0.2.0`
and targets the Dagger `v1.0.0-beta.21` engine contract.

Application users normally depend on [`tokeira-dagger-sdk`](crates/dagger-sdk). The client is
owned and asynchronous: it supports generated and raw GraphQL operations,
caller-supplied connections, exact CLI provisioning, existing `dagger run` sessions,
typed failures, diagnostics, W3C context propagation, and explicit shared shutdown.

A CI pipeline is an ordinary Rust program. This one gates a change on formatting, lints,
and tests: the three checks run concurrently in containers on a pinned toolchain, Cargo's
downloads and builds stay cached between runs, and a failing check fails the run with the
tool's own output.

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

The program is [`crates/dagger-sdk/examples/ci-checks.rs`](crates/dagger-sdk/examples/ci-checks.rs).
Run it from this directory against any Cargo project:

```bash
cargo run -p tokeira-dagger-sdk --example ci-checks -- path/to/project
```

Applications depend on `tokeira-dagger-sdk` from crates.io, and code names it
`dagger_sdk`. [`crates/dagger-sdk/README.md`](crates/dagger-sdk/README.md) covers
installation and application usage.

| Capability | Current boundary |
| --- | --- |
| Client API | Generated Core types and raw GraphQL over one owned asynchronous session |
| Connection safety | Exact engine version/revision checks, checksum-verified CLI downloads, loopback-only authenticated transport, and credential-safe diagnostics |
| Module support | Typed authoring attributes, descriptor generation, registration, dispatch, cancellation, and single-result publication |
| Standalone clients | Complete Core plus at most one independently pinned module, reusing the public client lifecycle; the runtime refuses client scopes on stock engines until their generation there is defined |
| Runtime | Installs on stock Dagger engines as an SDK module and builds its SDK content from its own commit |

Module authors should start with [`docs/module-authoring.md`](docs/module-authoring.md).
Standalone-client users should read [`docs/client-generation.md`](docs/client-generation.md).
[`docs/architecture.md`](docs/architecture.md) records ownership and safety boundaries, while
[`CONTRIBUTING.md`](CONTRIBUTING.md) and [`docs/maintaining.md`](docs/maintaining.md) contain the
pinned development and maintenance procedures.

The same work can ship as a Dagger module, which any CI runner, or a developer's machine,
calls with one command. This module tests a service against a disposable PostgreSQL database,
builds it into a minimal image that runs as a non-root user, and publishes the image with the
registry password passed as a secret:

<!-- The ci module's acceptance runs the module below as written. -->

```rust
//! CI for the `api` service: tests against a real database, then a minimal image.

// Generated by `dagger generate`; regenerate after changing exported items.
pub mod dagger_generated;

use dagger_generated::ModuleContext;
use dagger_sdk::{Container, Directory, ModuleError, Query, Secret};

/// Every step builds on one toolchain, pinned like the service's `rust-toolchain.toml`.
const RUST_IMAGE: &str = "rust:1.99.1-bookworm";

#[dagger_sdk::object(root, rename = "Ci")]
pub struct Ci {}

#[dagger_sdk::methods]
impl Ci {
    #[dagger(constructor, rename = "ci")]
    pub fn new() -> Ci {
        Ci {}
    }

    /// Runs the test suite against a disposable PostgreSQL service.
    #[dagger(function)]
    pub async fn test(
        &self,
        #[dagger(context)] context: ModuleContext,
        #[dagger(default_path = "/", ignore = ["**/target"])] source: Directory,
    ) -> Result<String, ModuleError> {
        let dag = context.query();
        let postgres = dag
            .container()
            .from("postgres:17")
            .with_env_variable("POSTGRES_PASSWORD", "ci")
            .with_exposed_port(5432)
            .as_service();
        let report = toolchain(dag, source)
            .with_service_binding("db", postgres)
            .with_env_variable("DATABASE_URL", "postgres://postgres:ci@db:5432/postgres")
            .with_exec(vec!["cargo", "test", "--locked"])
            .stdout()
            .await?;
        Ok(report)
    }

    /// Builds the release binary into a minimal image that runs as a non-root user.
    #[dagger(function)]
    pub fn build(
        &self,
        #[dagger(context)] context: ModuleContext,
        #[dagger(default_path = "/", ignore = ["**/target"])] source: Directory,
    ) -> Container {
        let dag = context.query();
        let api = toolchain(dag, source)
            .with_exec(vec!["cargo", "build", "--release", "--locked"])
            .file("target/release/api");
        dag.container()
            .from("gcr.io/distroless/cc-debian12:nonroot")
            .with_file("/usr/local/bin/api", api)
            .with_entrypoint(vec!["/usr/local/bin/api"])
    }

    /// Publishes the image and returns its digest-pinned reference.
    #[dagger(function)]
    pub async fn publish(
        &self,
        #[dagger(context)] context: ModuleContext,
        #[dagger(default_path = "/", ignore = ["**/target"])] source: Directory,
        address: String,
        username: String,
        password: Secret,
    ) -> Result<String, ModuleError> {
        let reference = self
            .build(context, source)
            .with_registry_auth(&address, password, username)
            .publish(&address)
            .await?;
        Ok(reference)
    }
}

/// The source on the pinned toolchain, with Cargo's downloads cached between runs.
fn toolchain(dag: &Query, source: Directory) -> Container {
    let registry = dag.cache_volume("cargo-registry");
    dag.container()
        .from(RUST_IMAGE)
        .with_mounted_cache(registry, "/usr/local/cargo/registry")
        .with_directory("/src", source)
        .with_workdir("/src")
}
```

In a workspace with the Rust SDK installed from `github.com/tokeira/dagger-rust-sdk/runtime`,
create the module with `dagger module init rust --name ci`, replace its `src/lib.rs` with
the code above, and run `dagger generate`. Each function's `source` defaults to the
workspace, so a CI job names only what changes:

```bash
dagger call ci test
dagger call ci publish --address=ghcr.io/your-org/api:1.0.0 \
  --username=ci-bot --password=env://REGISTRY_TOKEN
```

The standalone application examples under [`examples`](examples) each have their own
manifest; their exact commands are listed in [`examples/README.md`](examples/README.md).

## Development and releases

Development is engine-free and Rust-first: Cargo checks, generated-file checks, and the
runtime's direct Go tests run without an engine. Add a focused engine-backed regression
only when the direct production harness cannot represent the boundary. CI runs the same
gates through this repository's own Rust Dagger module, as [`docs/ci.md`](docs/ci.md)
describes. Runtime changes and every release are proven on a stock engine of the target
release, as [`docs/engine-integration.md`](docs/engine-integration.md) describes, and the
module runs that acceptance too.

A release publishes `tokeira-dagger-sdk-macros` and then `tokeira-dagger-sdk` to
crates.io at one version and tags the commit, and the runtime installs from that tag.
[`docs/maintaining.md`](docs/maintaining.md) covers the procedure and the release
identity.

## Built with Claude

Going forward, the overwhelming majority of contributions to this SDK will come from
[Claude](https://www.anthropic.com/claude), the AI model made by
[Anthropic](https://www.anthropic.com), working through
[Claude Code](https://claude.com/claude-code). The maintainer directs the work, reviews
every change, and approves every merge and release.

Commits credit every agent that took part with a `Co-authored-by` or `Assisted-by`
trailer, such as `Co-authored-by: Claude <noreply@anthropic.com>`, and pull requests name
them, as [`AGENTS.md`](AGENTS.md) sets out. Earlier work in this history credits Codex and
Kiro in the same way.

## License

Licensed under the [Apache License, Version 2.0](LICENSE). The SDK derives from the Rust
SDK in the [Dagger repository](https://github.com/dagger/dagger); [`NOTICE`](NOTICE) carries
its attribution.
