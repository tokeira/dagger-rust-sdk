# Module clients

A Cargo package that is not a Dagger module calls a module through a typed client that
`dagger module client add` generates into the package. The client is ordinary Rust in
the package's own source tree. It uses the session of the package's `dagger_sdk::Client`,
and the session has the engine serve the module before the client's first request.

A Dagger module calls another module through its dependencies instead, which the same
command records. The [module authoring guide](module-authoring.md#dependencies) describes
them.

## Add a client

The package is in a Dagger workspace that has the runtime installed from a Git reference,
as the [module authoring guide](module-authoring.md#create-a-module) describes. From the
package's directory, name the module by its path or its Git reference:

```console
cd app
dagger module client add ../greeter
```

The engine finds the nearest directory with a `Cargo.toml`, records the client for it in
the workspace's `dagger.toml`, and generates the package's clients:

```toml
[sdks.rust.scopes.app]
clients = ["./.dagger/modules/greeter"]
```

Then declare the generated module in the package's source, once, by hand. Generation
never edits source the author wrote:

```rust
mod dagger_clients;
```

The package also needs an asynchronous runtime, such as Tokio with its `macros` and
`rt-multi-thread` features. Generation adds the SDK itself, as
[Cargo.toml](#cargotoml) describes.

## Call a module

Each client is a Rust module of `dagger_clients`, named after the Dagger module it
calls:

```rust
mod dagger_clients;

use dagger_clients::greeter;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = dagger_sdk::connect().await?;

    let greeter = greeter::Client::new(&client);
    let with_opts = greeter
        .greeter_opts(&greeter::QueryGreeterOpts::default().with_greeting("hi"))
        .hello("Ian")
        .await?;
    println!("{with_opts}");

    let plain = greeter.greeter();
    let message = plain.message("Ian");
    let id = message.id().await?;
    let loaded: greeter::GreeterMessage = greeter.load(id).await?;
    println!("{}", loaded.text().await?);

    client.close().await?;
    Ok(())
}
```

- `greeter::Client::new(&client)` builds the module's client on the session of `client`.
  It sends nothing. The session asks the engine to serve the module before the first
  request made through the client, or through any handle built from it, and does so
  once for each module however many clients and handles share the session.
- The client has one method for the module's constructor, named after the module, which
  returns the module's main object. A constructor with optional arguments has a second
  method, `greeter_opts`, that borrows a `QueryGreeterOpts`.
- The module's types are exported from the client's Rust module: `greeter::Greeter`,
  `greeter::GreeterMessage`, `greeter::GreeterHelloOpts`. The engine names them after
  the module. Calls follow the convention of the Core bindings and of a module's
  dependencies: required arguments are positional in the alphabetical order of their
  GraphQL names, optional arguments go in an options struct passed to an `_opts` method,
  a function that returns an object returns a lazy handle, and one that returns a value
  is `async`. The [module authoring guide](module-authoring.md#dependencies) gives the
  convention in full.
- `greeter.r#ref::<T>(id)` builds a handle from an ID without a request, and
  `greeter.load::<T>(id).await` first checks that the object exists and has that type.
  Both take an ID or a handle, and accept every type that implements the client's
  `ClientLoadable`: the module's handles and Core's.
- A client offers its own module only. Core stays on `client.query()`, and a Core object
  passes to and from a module's functions as the same `dagger_sdk` type.
- Close the SDK client when the work is done. Every module client and handle shares its
  session.

A request through a module client can fail in three ways of its own. Each is a
`QueryError::ModuleClient` whose source is the failed request's error:

- `load module "<address>"`: the engine could not serve the module.
- `generated client for "<address>" is out of date; regenerate it`: the client asked for
  a function or a field the module no longer has.
- `the engine does not support generated module clients (serveModule is required)`: the
  engine's API has no `serveModule`.

A failed load is tried again by the next request.

## What generation writes

```text
app/
├── Cargo.toml                      the SDK dependency
└── src/
    ├── main.rs                     the author's
    └── dagger_clients/
        ├── .dagger-generated.json
        ├── mod.rs                  one `pub mod` for each client
        └── greeter/
            ├── .dagger-generated.json
            ├── mod.rs              Client, r#ref, load, ClientLoadable
            ├── greeter.rs          the main object and the constructor
            └── greeter_message.rs  one file for each of the module's other types
```

Everything under `src/dagger_clients/` is generated. Commit it with the package, and do
not edit it.

- A client's directory is the module's name as a Rust identifier: `naming-app` is
  `naming_app`. A name that is a Rust keyword is declared with `r#`, in a directory
  without it.
- Requests name the module's types and functions exactly as the engine serves them. A
  module `naming-app` with a type `HTTPFetcher` is served the type
  `NamingAppHTTPFetcher`. Its binding is the Rust type `NamingAppHttpFetcher` in
  `naming_app_http_fetcher.rs`, and every request the binding sends says
  `NamingAppHTTPFetcher`.
- A local module's client holds the module's path from the workspace root, and a Git
  module's client holds its reference and the commit `dagger.lock` pins it to. The
  engine serves the module from there at run time. When the workspace itself comes
  from a Git repository rather than a local directory, a module of that repository at
  the workspace's commit and ref is held by its path, as a local module is, so that
  its client does not change with every commit.
- Every generated Rust file begins with `// @generated`. Each directory holds a record,
  `.dagger-generated.json`, of the files generation wrote there and the SHA-256 of each:

  ```json
  {
    "version": 1,
    "files": {
      "mod.rs": "11e05cdcbb63a577fcdee64761802a08a05f56367f7d98d49bb4e5eb59943987"
    }
  }
  ```

  Generation replaces or removes a file only when it begins with the marker and still
  has its recorded hash.
- `src/dagger_clients/mod.rs` declares each client as `#[rustfmt::skip] pub mod <name>;`,
  in the order of the names, and each client's own `mod.rs` allows the lints generated
  code cannot avoid, such as `dead_code` in a binary that calls part of a client. The
  package's own `cargo fmt --check` and `cargo clippy -- -D warnings` pass with nothing
  to configure.

## Cargo.toml

Generated clients are written for one release of the SDK, so the package depends on
exactly that release. Generation writes the dependency, from the reference the runtime
is installed from, as [engine-integration.md](engine-integration.md#sdk-content)
describes:

```toml
dagger-sdk = { package = "tokeira-dagger-sdk", version = "=X.Y.Z" }
```

- A package that already depends on `tokeira-dagger-sdk` keeps the dependency's key, and
  so the name its code calls the SDK by, and its other fields, such as `features`. Only
  the fields that say where the package comes from and which release are replaced. A
  package without the dependency gains `dagger-sdk`.
- A dependency inherited from a Cargo workspace, `{ workspace = true }`, is accepted when
  the workspace's `[workspace.dependencies]` states exactly the release. Otherwise
  generation is refused with the entry to write there. Generation never edits the
  workspace's manifest.
- `Cargo.lock` is never read or written. The next `cargo` command brings it up to date.
- `Cargo.toml` is edited only when a client is generated.

## Keep clients current

A client is generated code for the module as it was. When the module's functions or
types change, regenerate, and commit the result:

- `dagger generate` regenerates the package's clients, and restores a generated file
  that was deleted.
- `dagger module client update` reads each client's module again, moves a Git module to
  the latest commit of its reference in `dagger.lock`, and regenerates. Name a recorded
  module to update that one alone.
- `dagger module client rm <module>` removes a client. Name the module as `dagger.toml`
  records it, such as `./.dagger/modules/greeter`. Generation removes the client's
  unchanged files and its line in `src/dagger_clients/mod.rs`, and keeps the client's
  record, emptied. Removing the last client also removes `src/dagger_clients/mod.rs`;
  remove `mod dagger_clients;` from the package's source then. `Cargo.toml` keeps the
  SDK dependency.

Moving the workspace to another release of the runtime changes the release the clients
are written for. The next `dagger generate` writes that release into `Cargo.toml` and
regenerates the clients for it.

## What is refused

Each refusal happens before the first write, so the package is left as it was.

- An edited generated file:

  ```text
  src/dagger_clients/greeter/greeter_style.rs contains user changes; refusing to overwrite it
  ```

  Restore the file, or delete it and run `dagger generate`. The same holds for a file
  under `src/dagger_clients/` that generation would write and that does not begin with
  the marker. An edited file of a removed client is not refused: it stays where it is.
- A runtime installed from a local path, which names neither a release nor a commit for
  the package to depend on. The message names the runtime as the workspace installs it:

  ```text
  Rust SDK "rust-sdk" is installed from a local path; install it from a Git reference to initialize modules or generate clients
  ```

- A package that is a module still configured by `dagger.json`: its directory holds
  `dagger.json`, or a `dagger.json` above it names the package as the module's
  `source`, as in the layout of Dagger's guide to migrating from `dagger.json`, where
  the module's source is `.dagger`. A module's clients are its dependencies, so the
  advice is the migration:

  ```text
  this package is the source of the module configured by /dagger.json, where the Rust SDK does not write dependencies; run `dagger module migrate /`, which moves its configuration to dagger-module.toml and records its dependencies as clients, then `dagger module client add` again from the module's directory, the workspace root, not from this package; a coding agent passes `-y` to apply each, or `--no-apply` to preview the migration
  ```

  The [module authoring guide](module-authoring.md#dependencies) describes a module's
  dependencies.
- A directory whose `Cargo.toml` has no `[package]`, such as the root of a Cargo
  workspace:

  ```text
  Cargo.toml has no [package]; module clients are generated into a Cargo package
  ```

- An inherited SDK dependency that does not state the release:

  ```text
  the package inherits `dagger-sdk` from its Cargo workspace, which does not state the SDK release these clients are generated for; write `dagger-sdk = { package = "tokeira-dagger-sdk", version = "=X.Y.Z" }` under [workspace.dependencies] in ../Cargo.toml
  ```

  When the Cargo workspace's manifest is not above the package in the Dagger workspace,
  generation cannot read it, and the message gives the entry to write in the package's
  own `[dependencies]` instead.
- A `dagger-sdk` dependency that is another package:

  ```text
  the `dagger-sdk` dependency of Cargo.toml is not the `tokeira-dagger-sdk` package; generated clients need the SDK, and add it under that name
  ```

- Two modules whose names give one Rust module, such as a local `greeter` and a Git
  `greeter`:

  ```text
  modules `greeter` and `greeter` would both be the Rust module `greeter`; a package can have a client for only one of them
  ```

- A module whose name gives the Rust module `mod`, which would meet the clients' own
  `mod.rs`:

  ```text
  module `mod` would be the Rust module `mod`, which Rust would look for in both src/dagger_clients/mod.rs and src/dagger_clients/mod/mod.rs; no client can be generated for it
  ```

- A Git module that is not pinned to a full commit, and a module that is neither a Git
  nor a local module.
- A module with a type that would be bound as `Client` or `ClientLoadable`, or in
  `mod.rs`, and a module whose constructor would be bound as `Client::new`,
  `Client::ref`, or `Client::load`. The message names the type or the constructor and
  says that no client can be generated for the module.
- A module function that names a type with no typed binding, with
  `SCHEMA_FIELD_UNMAPPED` at the function's coordinate, as for a module's dependencies.
- A path under `src/dagger_clients/` that crosses a symbolic link.

## Contributor checkpoint

Module-client development is Rust-first and engine-free. From the repository root, run only
the affected typed slices and reuse current checked assets and the materialized fixture
baseline:

```console
cargo test -p dagger-codegen --test module_client --locked
cargo test -p dagger-sdk-engine --test package_clients --locked
```

Investigate fixture sequencing and the owning input before broadening a failed slice.
Do not use an engine run as a local checkpoint substitute. The ordinary completed-engine
check runs one isolated external Rust consumer and leaves focused engine regressions to
their owning tests.
