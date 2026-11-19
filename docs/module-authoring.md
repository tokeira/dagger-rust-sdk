# Rust module authoring and verification

This guide records the durable authoring, generation, dispatch, local-checkpoint, and
stock-engine verification contracts for the Rust SDK. The public syntax is ordinary
typed Rust; the engine adapter and generated support remain implementation details.

## Create a module

Install the runtime into a Dagger workspace, then create the module:

```console
dagger install github.com/tokeira/dagger-rust-sdk/runtime@<ref>
dagger module init rust --name my-module
```

`<ref>` is a release tag such as `v0.7.1`, or a full commit of this repository.
Initialization writes the module's `Cargo.toml`, `dagger-module.toml`, and a starter
`src/lib.rs` whose root object is named after the module and whose crate documentation
is a placeholder for the module's description. The `Cargo.toml` depends on the
SDK that `<ref>` names, as [engine-integration.md](engine-integration.md#sdk-content)
describes. After changing exported items, run `dagger generate`, then call a function
with `dagger call my-module <function>`.

## Upgrade a module

Each SDK release targets one Dagger release, and from `v1.0.0-beta.22` each Dagger release
serves an API view of its own and keeps serving the views of earlier releases. The engine
gives a module the view of the release that its `dagger-module.toml` declares as
`engineVersion`. Initialization sets it to the SDK's target release, and it stays there:
generation refuses a module whose view is not the SDK's target, and names the
`engineVersion` to declare. A module that declares the target runs on an engine of that
release or of any later 1.x release. To move a module to a new SDK release:

1. `dagger module update rust-sdk --version <ref>` moves the workspace's `dagger.toml` to
   the new runtime.
2. In the module's `dagger-module.toml`, set `[runtime] source` to the same reference by
   hand, because `dagger module update` leaves it unchanged, and set `engineVersion` to
   the new SDK's target release, such as `"v1.0.0-beta.27"`.
3. Change the module's `Cargo.toml` dependency to the SDK that `<ref>` names, as
   [engine-integration.md](engine-integration.md#sdk-content) describes, then run
   `cargo update -p tokeira-dagger-sdk` in the module.
4. Run `dagger generate`, then build the module and adapt it to the API changes that the
   [changelog](../CHANGELOG.md) lists.

From `v1.0.0-beta.26`, the engine names a module's types, functions, arguments, and enum
members by new rules, as it does for every SDK, so a module that declares that release or a
later one may need its callers to change some spellings. A struct `HttpClient` in a module
`tiny` is `TinyHTTPClient` rather than `TinyHttpClient`, and an enum variant `GitHub` is the
member `GITHUB` rather than `GIT_HUB`. The module's Rust code does not change.

## Authoring surface

Exports are explicit. Rust visibility still determines whether generated sibling code
can access an item, but `pub` alone never exports it to Dagger. `tokeira-dagger-sdk`
re-exports the exact-version attributes; users do not depend on
`tokeira-dagger-sdk-macros` directly.

```rust
#[dagger_sdk::object(root, rename = "greeter")]
pub(crate) struct Greeter {
    #[dagger(field)]
    greeting: String,
    #[dagger(state)]
    private_revision: i64,
}

#[dagger_sdk::methods]
impl Greeter {
    #[dagger(constructor)]
    fn new(greeting: String) -> Self {
        Self {
            greeting,
            private_revision: 0,
        }
    }

    #[dagger(function)]
    fn greet(&self, #[dagger(default = "world")] name: String) -> String {
        format!("{}, {name}", self.greeting)
    }
}
```

`object`, `interface`, `enum_type`, and `scalar` mark exported type contracts;
`methods` checks constructors and functions. Nested `field`, `state`, `constructor`,
`function`, `context`, the [collection](#collections) markers, defaults, target
metadata, documentation, deprecation, and renames are interpreted once by the shared
authoring grammar. Unsupported shapes fail at authored coordinates rather than
degrading to untyped JSON.

A function's `role` gives it to a Dagger command. The engine checks the shape that each
role needs when it loads the module, as it does for every SDK:

| Metadata | Run by | Shape |
| --- | --- | --- |
| `role = "check"` | `dagger check` | returns nothing |
| `role = "generator"` | `dagger generate` | returns a `Changeset` and requires no arguments |
| `role = "up"` | `dagger start` | returns a `Service` and requires no arguments |
| `role = "agent"` | `dagger agent` | `fn expert(&self, base: Llm) -> Llm` |

An agent function adds the module's expertise to a conversation: its one required `Llm`
argument is the conversation, and it returns the conversation it extends. Its other
arguments are optional or an `Agent`, which receives the agent that owns the
conversation. `dagger agent` composes the agent functions of the workspace's modules into
one conversation. An agent usually gives that conversation its module's functions as
tools, through the receiver that the context holds:

```rust
#[dagger(function, role = "agent")]
fn expert(&self, base: Llm, #[dagger(context)] context: ModuleContext) -> Llm {
    base.with_tools(context.current_node())
        .with_system_prompt("You review releases.")
}
```

The agent function itself is not one of the tools. Like `Function.withAgent`, the role is
experimental upstream, so a later Dagger release may change it.

Rustdoc comments become descriptions. The `///` comment of an exported type, field, or
function describes it, and the crate root's `//!` comment describes the module: its first
line is the module's short description, and the rest is its long description. Only
comments and `doc` attributes whose value is a string literal are read, so a crate root
documented with `#![doc = include_str!(...)]` gives the module no description.

Persistent state is owned data reconstructed for the selected object. A public field
appears in the TypeDef; a private persistent field participates only in state encoding;
an ordinary unmarked field remains implementation detail. Generated Core and self handles
preserve their typed IDs and re-enter through the active session. A module can also call
the modules it depends on and keep their objects in private state, as
[Dependencies](#dependencies) describes.

Arguments, results, and fields can also be Core enums and scalars, such as
`NetworkProtocol`, `Platform`, `Json`, and `Bytes`. An enum crosses the boundary as the
name of its member and accepts the aliases the engine gives a member; a scalar keeps its
exact text, so a `Json` value is the text of a JSON document. A default names an enum
member or gives a scalar's text as a string literal:

```rust
#[dagger(function)]
fn listen(
    &self,
    #[dagger(default = NetworkProtocol::Udp)] protocol: NetworkProtocol,
    #[dagger(default = "linux/arm64")] platform: Platform,
) -> String {
    format!("{protocol:?} on {platform}")
}
```

Generation refuses a default that is not one of the enum's members, and a `Json` default
that is not JSON.

### Collections

A collection is a set of objects with unique keys, such as the packages or test files a
module discovers in a project. Callers see the operations the engine gives it: `keys`,
`list`, `get(key:)`, and `subset(keys:)`. `dagger check` and `dagger generate` gain a
flag, named after its items, that selects items by key. Mark the object `collection`,
its keys field `keys`, the function that returns one item `get`, and optionally a field
that receives the collection's changes `delta`:

```rust
#[dagger_sdk::object(collection)]
#[derive(Clone)]
pub(crate) struct Packages {
    #[dagger(field, keys)]
    names: Vec<String>,
    #[dagger(field, delta)]
    changes: Option<CollectionDelta>,
    base: CollectionBase,
}

#[dagger_sdk::methods]
impl Packages {
    #[dagger(function, get)]
    fn package(&self, name: String) -> Package {
        Package { name }
    }

    #[dagger(function)]
    fn without_examples(&self) -> Packages {
        Packages {
            names: self.names.iter().filter(|name| !name.starts_with("examples/")).cloned().collect(),
            ..self.clone()
        }
    }
}
```

The engine checks a collection's shape when it loads the module, as it does for every
SDK: the keys field holds a list of strings, numbers, booleans, a scalar, or an enum;
the get function takes one argument of the key type and returns an object; and no other
function is named `keys`, `list`, `get`, or `subset`. A member that its author leaves
unmarked keeps its default name, so a field named `keys` or a function named `get`
needs no marker. Generation refuses `keys` or `delta` on anything but an exposed field,
`get` on anything but a function, and either outside a collection.

The `CollectionBase` field carries the collection that a value was derived from. The
engine fills it, and computes the delta, the keys added and removed since the base,
against it. A value built from the receiver, with `self.clone()` or `..self.clone()`,
keeps its base, as `without_examples` does. A value built afresh starts from
`CollectionBase::default()` and is its own base. A collection declares exactly one
such field, and no other object declares one.

### Dependencies

A module calls another module through typed bindings that `dagger generate` writes into
its own crate. Declare the dependency in the module's `dagger-module.toml`, under the
name the module will call it by:

```toml
[[dependencies]]
  name = "greeter"
  source = "../greeter"
```

Then run `dagger generate`. Each type of the dependency is bound in a file of
`src/dagger_generated/` and exported from `dagger_generated`. The engine names the types
after the dependency: its main object is `Greeter`, and its other types begin with that
name, such as `GreeterMessage` and `GreeterMode`. Run `dagger generate` again whenever the
dependency's functions or types change, and commit the result. Bindings that no longer
match the dependency still compile, and a call through them fails at run time with the
engine's error.

The dependency's main object comes from the query root that the context holds:

```rust
use dagger_generated::{GreeterMode, GreeterShoutOpts, ModuleContext, QueryGreeterOpts};

#[dagger(function)]
async fn welcome(
    &self,
    name: String,
    #[dagger(context)] context: ModuleContext,
) -> Result<String, dagger_sdk::ModuleError> {
    let greeter = context
        .query()
        .greeter_opts(&QueryGreeterOpts::default().with_greeting("hi"));
    let loud = greeter.with_mode(GreeterMode::Loud);
    let twice = GreeterShoutOpts::default().with_times(2);
    Ok(loud.shout_opts(name, &twice).await?)
}
```

`context.query()` has one method for each dependency, named after it, which takes the
arguments of the dependency's constructor. Calls follow the convention of the Core
bindings:

- Required arguments are positional, in the alphabetical order of their GraphQL names,
  not the order the dependency declares them in. `vault(name:, mount:)` is
  `query.vault(mount, name)`.
- A function with optional arguments has a second method, `<function>_opts`, that
  borrows an options struct after the required arguments. The struct is named after the
  owner and the function, `GreeterShoutOpts`, or `Query<Dependency>Opts` for a
  constructor. It starts from `default()` and has one `with_<argument>` setter for each
  optional argument; an argument that is not set is left out of the request.
- A string argument takes anything that converts to a `String`, and an object argument
  takes the object's handle or its ID.
- A function that returns an object returns a lazy handle and sends nothing. A function
  that returns a value, a nullable object, or a list is `async` and returns
  `Result<T, QueryError>`, which `?` converts to a `ModuleError` that keeps the
  dependency's message.
- An enum of the dependency is a Rust enum whose variants are its members.
- An interface of the dependency is a trait, `GreeterNamed`, and a handle,
  `GreeterNamedClient`, that implements it. A function that returns the interface
  returns the handle. The dependency's objects that implement the interface implement
  the trait and convert to the handle where the interface is an argument.

An object can keep a dependency's objects and enums in private state, where they survive
from one call to the next:

```rust
#[dagger_sdk::object]
pub(crate) struct Welcome {
    #[dagger(state)]
    greeter: Greeter,
    #[dagger(state)]
    mode: GreeterMode,
}
```

`context.query().r#ref::<T>(id)` builds a handle from an ID without a request, and
`context.query().load::<T>(id).await` first checks that the object exists and has that
type. Both accept every type that implements `dagger_generated::ModuleLoadable`: Core's
handles and the handles of the module's dependencies.

Some rules follow from the engine and from Rust:

- The engine refuses a dependency's type in the module's own API: in an exported
  function's arguments or result, and in a `#[dagger(field)]`. Generation and the build
  succeed, and the module then fails to load with the engine's reason. Keep such a value
  in `#[dagger(state)]`, and return its data or a Core object.
- A function that every handle already has through Rust's prelude is bound with a
  trailing underscore: `clone_`, `clone_from_`, `clone_into_`, `into_`, `to_owned_`, and
  `try_into_`. `handle.clone()` still clones the handle. The `_opts` method keeps the
  plain name, `clone_opts`. The same holds for a dependency of one of these names:
  `context.query().clone_()` constructs a dependency called `clone`. When a required
  argument is itself called `opts`, the options parameter is `opts_`.
- Generation refuses a dependency called `load` or `ref`, whose constructor would
  redefine a method of the query root, and a dependency type that would be bound as
  `ModuleContext`, `ModuleQuery`, `ModuleLoadable`, or another name or file of the
  generated support. It also refuses a type of the module's own with the name of one of
  a dependency's types. In each case, declare the dependency under another `name`, or
  rename the local type.
- A dependency type called `Result`, `Option`, or `Vec` is written by its path,
  `dagger_generated::Result`, wherever the module names it as a field's type. The bare
  name is the standard type.
- A dependency function that names a type with no typed binding fails generation at the
  function's coordinate with `SCHEMA_FIELD_UNMAPPED`. Such a type is neither part of the
  Core API the SDK targets nor owned by a dependency: a later engine's addition, for
  example.

Compared with a Go module, these are left out:

- A module cannot call its own functions through the query root.
- A dependency's object cannot be returned through an interface the module declares.
- A dependency's handle does not implement the traits of the Core interfaces it
  implements, such as `Node`. It still converts to their IDs.
- The runtime refuses a module scope with clients recorded for it.
- There is no `As<Interface>()` and no `Loaded[T]`. To use a value as an interface it
  implements, build the interface's handle from the value's ID:
  `context.query().r#ref::<GreeterNamedClient>(message.id().await?)`. The same call
  passes a Core object where a dependency's interface is expected.

## Descriptor and generated ownership

The source compiler walks the selected Cargo package, respects the compilation cfg,
resolves the transitive local type closure, and produces one canonical descriptor.
Registration, introspection, codecs, the dispatch registry, and the module entrypoint
are projections of that descriptor. Procedural macros emit only typed crate-local
access and invocation bridges, so authors never maintain a parallel schema or switch
statement.

Generated assets carry exact target, source, generator, descriptor, and content
identities, and the identity of the schema they consume: the SDK's reviewed definitions
of its target's Core API, and the types the engine adds for the module's dependencies.
Documentation that a later engine's copy of the target's view rewords, and fields and
optional arguments it adds that the SDK does not bind, leave that identity unchanged, so
the committed assets stay current on that engine. Checked mode compares committed assets
directly. Regeneration is permitted only when one of those owning inputs changes, selects
only the affected asset domain, and replaces or removes only paths proven by the prior
compatible manifest. User-owned or unknown bytes are preserved and diagnosed.

The generator's identity is the SDK source and toolchain the runtime seals, not the
platform its tools were built for, so files generated on one engine platform are
current on every other. A prior manifest is compatible when it covers the same
operation and output root. A runtime upgrade changes the generator, and may change
the target or SDK dependency; `dagger generate` then replaces exactly the files the
prior manifest owns.

## Dispatch and errors

The entrypoint reads an active call once into a typed envelope. An empty parent name
selects registration; an empty function name remains the root constructor selector.
For invocation, parent state and the complete named argument set are validated before
user code runs. Context parameters are injected and absent from TypeDefs. When the
engine refuses the registration, for example because a function does not have the
shape its role requires, the entrypoint returns the engine's reason as the
registration's error, as the Go SDK does, so the command that loaded the module shows it.

Each call owns its parent, decoded values, session context, cancellation signal, and
result sink. The entrypoint installs a termination-signal producer, so `SIGTERM` and
`SIGINT` trip the same `ModuleCancellation` the dispatcher and authored code observe —
an intentional extension beyond the definitive Go SDK, which offers module authors no
cancellation. A cancellation that wins while the authored future is running drops that
future and publishes one structured cancelled error as the call's terminal record;
once the authored future has completed, its outcome is published without racing
cancellation. Successful values and unit, structured application errors, contained
panics, cancellation, encoding failures, publication failures, and session-close
failures remain distinct, and every call still publishes at most one terminal
outcome. Diagnostics
retain authored and wire coordinates plus typed safe causes without rendering tokens,
credential-bearing URLs, arbitrary panic payloads, host paths, or unbounded values.

## Development build and test: engine-free and Rust-first

During implementation, run the narrowest owning package or fixture. The local
development loop uses Cargo directly from the repository root and the direct Go ABI
package under `runtime`. It exercises the production source compiler, descriptor,
projections, registration, codecs, context, dispatcher, entrypoint adapter, and
result sink with Rust values and a recording transport.

The development contract is strict:

- no Dagger engine process, CLI/module invocation, or network-backed engine graph;
- no unrelated SDK builder, test, generation, or distribution-wide build;
- checked Core and module assets unless an owning digest changed;
- one scoped regeneration decision when change genuinely requires it;
- locked package checks, all module properties, bounded compile fixtures, direct
  adapter tests, formatting, Clippy, rustdoc, Cargo Deny, public package contents, and
  byte-clean output.

The focused test slices are:

```console
cargo test -p dagger-sdk-engine \
  --test initialization_properties \
  --test runner_properties --locked
cargo test -p dagger-codegen \
  --test module_authoring_assets \
  --test module_diagnostics --locked
cargo test -p tokeira-dagger-sdk --test module_authoring_compile --locked
```

The complete local gate additionally runs the canonical workspace format, check, test,
warning-denied Clippy/rustdoc, Cargo Deny, package, direct Go ABI, generated
drift/ownership, and clean-output checks once. It does not continuously
regenerate checked assets.

If a proposed local check genuinely requires an engine, keep the direct model gap
explicit and add the smallest focused engine-backed regression test that owns that
boundary. Release readiness is a separate, smaller check: the
[stock-engine acceptance](engine-integration.md#stock-engine-acceptance) installs the
runtime from a pushed commit, then initializes, regenerates, loads, and calls a module
and exercises a plain client. It verifies the module-authoring path end to end without
replacing the faster compiler and dispatch tests above.
