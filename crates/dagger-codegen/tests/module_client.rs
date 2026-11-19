//! One module's typed client for a Cargo package that is not a module.
//!
//! A package calls a module through a client generated into a directory of its own:
//! `mod.rs` with the module's `Client`, and one file for each type the module owns,
//! bound as a module's dependencies are. These tests render clients from the schema an
//! engine serves for one, and check what the client declares, the names it sends, what
//! generation refuses, and that a package compiles its clients without warnings.
//!
//! `fixtures/module_clients/` holds what two Dang modules add to the schema of their
//! client, captured from an engine at v1.0.0-beta.27 with
//! `dagger -c 'module-source <module> | client-schema-introspection-json
//! --client-version v1.0.0-beta.27 | export <name>.json'`. `greeter` is the
//! acceptance's module (`.dagger/modules/ci/acceptance/greeter/main.dang`), and `naming-app`
//! has a type with an acronym in its name (`naming-app.dang` beside the fixtures). The
//! Core part of each capture is the checked `codegen/schema.json`, so the fixtures keep
//! the module's part alone and the tests splice it back. Refresh them from a directory
//! of new captures with `DAGGER_UPDATE_MODULE_CLIENT_SCHEMA_FIXTURES=<directory> cargo
//! test -p dagger-codegen --locked --test module_client`, which refuses a capture the
//! splice does not give back.

mod support;

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use dagger_codegen::diagnostic::{DiagnosticCode, DiagnosticSet};
use dagger_codegen::engine::{
    CandidateArtifactKind, ContentDomain, ModuleClientRenderInput, PostWorkPlan,
    RelativeOperationPath, RendererOutput, project_visible_schema, render_module_client,
};
use dagger_codegen::schema::canonical::SchemaName;
use dagger_codegen::target::CodegenTarget;

use support::{
    CORE_SCHEMA_BYTES, GREETER_CLIENT_SCHEMA_PART, NAMING_APP_CLIENT_SCHEMA_PART, TARGET_BYTES,
    add_constructed_object, add_unreturned_object, adversarial_dependency_core_schema,
    changed_dependency_core_schema, dependency_core_schema, dependency_field,
    dependency_object_type, module_client_schema, module_client_schema_part,
};

/// The directory the tests render a client into.
const ROOT: &str = "src/dagger_clients/bound";

/// Where the engine serves the fixtures' modules from: a local module's path in its
/// workspace, with no pin.
const GREETER_ADDRESS: &str = "/.dagger/modules/greeter";
const NAMING_APP_ADDRESS: &str = "/.dagger/modules/naming-app";

fn target() -> CodegenTarget {
    CodegenTarget::decode_exact(TARGET_BYTES).expect("checked target must decode")
}

fn render(
    schema: &[u8],
    alias: &str,
    address: &str,
    pin: &str,
) -> Result<RendererOutput, DiagnosticSet> {
    let plan = project_visible_schema(&target(), schema)?;
    let output = RelativeOperationPath::parse(ROOT).expect("fixture path must parse");
    render_module_client(ModuleClientRenderInput {
        schema: &plan,
        output: &output,
        sdk_dependency_alias: alias,
        address,
        pin,
    })
}

/// A rendered client's files by name.
fn files(output: &RendererOutput) -> BTreeMap<String, String> {
    output
        .artifacts
        .iter()
        .map(|(path, artifact)| {
            let name = path
                .as_str()
                .strip_prefix(&format!("{ROOT}/"))
                .expect("every artifact lies in the client's directory");
            let source = String::from_utf8(artifact.content.clone()).expect("Rust is UTF-8");
            (name.to_owned(), source)
        })
        .collect()
}

fn client(schema: &[u8], alias: &str, address: &str, pin: &str) -> BTreeMap<String, String> {
    files(&render(schema, alias, address, pin).expect("the module's client must render"))
}

fn greeter() -> BTreeMap<String, String> {
    client(
        &module_client_schema(GREETER_CLIENT_SCHEMA_PART),
        "dagger_sdk",
        GREETER_ADDRESS,
        "",
    )
}

fn naming_app() -> BTreeMap<String, String> {
    client(
        &module_client_schema(NAMING_APP_CLIENT_SCHEMA_PART),
        "dagger_sdk",
        NAMING_APP_ADDRESS,
        "",
    )
}

/// Source as its tokens, so a comparison ignores how either side is laid out. Group
/// delimiters and commas are dropped, and each standard item is written under its short
/// name, which is how the expectations below are written.
fn tokens(source: &str) -> String {
    fn flatten(stream: proc_macro2::TokenStream, tokens: &mut Vec<String>) {
        for token in stream {
            match token {
                proc_macro2::TokenTree::Group(group) => flatten(group.stream(), tokens),
                proc_macro2::TokenTree::Punct(punctuation) if punctuation.as_char() == ',' => {}
                token => tokens.push(token.to_string()),
            }
        }
    }
    fn flat(source: &str) -> String {
        let source = source.trim();
        let source = source.strip_suffix('{').unwrap_or(source);
        let stream = source
            .parse::<proc_macro2::TokenStream>()
            .unwrap_or_else(|error| panic!("`{source}` must tokenize: {error}"));
        let mut flat = Vec::new();
        flatten(stream, &mut flat);
        format!(" {} ", flat.join(" "))
    }

    let mut tokens = flat(source);
    for prefix in [
        "::core::clone::",
        "::core::convert::",
        "::core::future::",
        "::core::marker::",
        "::core::option::",
        "::core::result::",
        "::std::string::",
    ] {
        tokens = tokens.replace(&flat(prefix), " ");
    }
    tokens
}

#[track_caller]
fn assert_binds(file: &str, source: &str, expected: &str) {
    assert!(
        tokens(source).contains(&tokens(expected)),
        "{file} lacks `{expected}` in:\n{source}"
    );
}

/// Every string literal of a file's code, outside its attributes: the names a binding
/// sends to the engine, and the address and pin of the client.
fn string_literals(source: &str) -> BTreeSet<String> {
    fn collect(stream: proc_macro2::TokenStream, literals: &mut BTreeSet<String>) {
        let mut stream = stream.into_iter().peekable();
        while let Some(token) = stream.next() {
            match token {
                proc_macro2::TokenTree::Punct(punctuation) if punctuation.as_char() == '#' => {
                    if matches!(
                        stream.peek(),
                        Some(proc_macro2::TokenTree::Punct(bang)) if bang.as_char() == '!'
                    ) {
                        stream.next();
                    }
                    stream.next();
                }
                proc_macro2::TokenTree::Group(group) => collect(group.stream(), literals),
                proc_macro2::TokenTree::Literal(literal) => {
                    if let Ok(string) = syn::parse_str::<syn::LitStr>(&literal.to_string()) {
                        literals.insert(string.value());
                    }
                }
                _ => {}
            }
        }
    }
    let mut literals = BTreeSet::new();
    collect(source.parse().expect("a binding tokenizes"), &mut literals);
    literals
}

fn name(value: &str) -> SchemaName {
    SchemaName::try_from(value).expect("fixture name is valid")
}

#[track_caller]
fn refusal(
    refused: &DiagnosticSet,
    code: DiagnosticCode,
) -> &dagger_codegen::diagnostic::Diagnostic {
    refused
        .diagnostics()
        .iter()
        .find(|diagnostic| diagnostic.code == code)
        .unwrap_or_else(|| panic!("no {code} in {refused:?}"))
}

fn coordinate(diagnostic: &dagger_codegen::diagnostic::Diagnostic) -> Option<&str> {
    diagnostic
        .coordinate
        .as_ref()
        .map(|coordinate| coordinate.as_str())
}

// The engine installs the bound module in its client's schema as it installs a
// dependency in a module's, so the module's types carry `@sourceMap(module:)` and are
// found by it. A client is `mod.rs` and one file for each of them, and nothing else:
// Core is the SDK's.
#[test]
fn a_client_is_mod_rs_and_one_file_for_each_type_the_module_owns() {
    let schema = module_client_schema(GREETER_CLIENT_SCHEMA_PART);
    let plan = project_visible_schema(&target(), &schema).expect("client schema must project");
    for owned in ["Greeter", "GreeterMessage", "GreeterNamed", "GreeterStyle"] {
        assert_eq!(plan.dependency_module(&name(owned)), Some("greeter"));
    }
    assert!(!plan.is_dependency_type(&name("Directory")));

    let output = render(&schema, "dagger_sdk", GREETER_ADDRESS, "").expect("client must render");
    assert_eq!(output.content_domain, ContentDomain::ModuleClient);
    let paths = output.artifacts.keys().cloned().collect::<BTreeSet<_>>();
    assert_eq!(
        paths
            .iter()
            .map(RelativeOperationPath::as_str)
            .collect::<Vec<_>>(),
        [
            "src/dagger_clients/bound/greeter.rs",
            "src/dagger_clients/bound/greeter_message.rs",
            "src/dagger_clients/bound/greeter_named.rs",
            "src/dagger_clients/bound/greeter_style.rs",
            "src/dagger_clients/bound/mod.rs",
        ]
    );
    assert!(
        output
            .artifacts
            .values()
            .all(|artifact| artifact.kind == CandidateArtifactKind::RustSource)
    );
    // Every file is generated Rust, to format before it is written.
    assert_eq!(
        output.post_work,
        [PostWorkPlan::FormatRust {
            files: paths.clone()
        }]
    );
    assert_eq!(output.vcs_generated, paths);
    assert!(output.vcs_ignored.is_empty());
    assert!(output.cargo_binary.is_none());

    let generated = files(&output);
    for (module, file) in [
        ("greeter", "greeter.rs"),
        ("greeter_message", "greeter_message.rs"),
        ("greeter_named", "greeter_named.rs"),
        ("greeter_style", "greeter_style.rs"),
    ] {
        assert_binds(
            "mod.rs",
            &generated["mod.rs"],
            &format!(
                "#[path = {file:?}] mod {module}; #[allow(unused_imports)] pub use self::{module}::*;"
            ),
        );
    }
    // The same schema in another order renders the same client.
    let mut reordered: serde_json::Value = serde_json::from_slice(&schema).expect("JSON");
    reordered["__schema"]["types"]
        .as_array_mut()
        .expect("types")
        .reverse();
    let reordered = serde_json::to_vec(&reordered).expect("JSON");
    assert_eq!(
        client(&reordered, "dagger_sdk", GREETER_ADDRESS, ""),
        generated
    );
}

// The record that owns a client's files recognises a generated file by its first line.
#[test]
fn every_file_of_a_client_begins_with_the_generated_marker() {
    for generated in [greeter(), naming_app()] {
        for (file, source) in &generated {
            let mut lines = source.lines();
            let first = lines.next().unwrap_or_default();
            assert!(
                first.starts_with("// @generated {\"format\":\"dagger-rust-visible-v1\""),
                "{file} begins `{first}`"
            );
            assert!(
                lines.next().is_some_and(|line| line.starts_with("//! ")),
                "{file} documents itself after the marker"
            );
            syn::parse_file(source).unwrap_or_else(|error| panic!("{file} must parse: {error}"));
        }
    }
}

// `Client::new` takes the package's SDK client and does no I/O: it starts a query that
// carries the module's address and pin, and the SDK serves the module before the first
// request. The address and pin are written as Rust string literals whatever they hold.
#[test]
fn a_client_is_made_from_the_sdk_client_with_the_module_s_address_and_pin() {
    assert_binds(
        "mod.rs",
        &greeter()["mod.rs"],
        "#[derive(Clone)]
        pub struct Client {
            query: ::dagger_sdk::QueryBuilder,
        }",
    );
    assert_binds(
        "mod.rs",
        &greeter()["mod.rs"],
        "#[must_use]
        pub fn new(client: &::dagger_sdk::Client) -> Self {
            Self {
                query: client.generated_module_query_builder(\"/.dagger/modules/greeter\", \"\"),
            }
        }",
    );
    let schema = module_client_schema(GREETER_CLIENT_SCHEMA_PART);
    let address = "github.com/example/modules/greeter@main";
    let pin = "0123456789abcdef0123456789abcdef01234567";
    let pinned = client(&schema, "dagger_sdk", address, pin);
    assert_binds(
        "mod.rs",
        &pinned["mod.rs"],
        &format!("client.generated_module_query_builder({address:?}, {pin:?})"),
    );
    // Only `mod.rs` states where the module is served from.
    for (file, source) in &pinned {
        assert_eq!(
            source.contains(address) || source.contains(pin),
            file == "mod.rs",
            "{file}"
        );
    }
    let awkward = "/a\"){evil}\n\\ \u{1f600}";
    let quoted = client(&schema, "dagger_sdk", awkward, "p\"in\\");
    let literals = string_literals(&quoted["mod.rs"]);
    assert!(literals.contains(awkward), "{literals:?}");
    assert!(literals.contains("p\"in\\"), "{literals:?}");
}

// The module's constructor is a method of `Client`, as a dependency's is a method of
// `ModuleQuery`: one method, and one more with the optional arguments when it has any.
#[test]
fn the_constructor_is_a_method_of_the_client_with_and_without_options() {
    let generated = greeter();
    for expected in [
        "impl super::Client {",
        "#[must_use] pub fn greeter(&self) -> super::Greeter {
            let __query = self.__query_builder().select(\"greeter\");
            super::Greeter::__from_query(__query)
        }",
        "#[must_use] pub fn greeter_opts(&self, opts: &QueryGreeterOpts) -> super::Greeter {
            let __query = self.__query_builder().select(\"greeter\");
            let __query = if let Option::Some(__value) = &opts.greeting { __query.argument(\"greeting\", __value) } else { __query };
            super::Greeter::__from_query(__query)
        }",
        "#[derive(Clone, Debug, Default)] #[non_exhaustive] pub struct QueryGreeterOpts {",
        "pub greeting: Option<String>,",
        "pub fn with_greeting(mut self, value: impl Into<String>) -> Self { self.greeting = Option::Some(Into::into(value)); self }",
    ] {
        assert_binds("greeter.rs", &generated["greeter.rs"], expected);
    }
    // An object that no `Query` field returns has no constructor.
    assert!(!generated["greeter_message.rs"].contains("super::Client"));

    // No arguments: no options type and no second method.
    let generated = naming_app();
    assert_binds(
        "naming_app.rs",
        &generated["naming_app.rs"],
        "impl super::Client {
            #[cfg_attr(not(doctest), doc = \"Selects GraphQL field `namingApp` on `Query`.\")]
            #[must_use]
            pub fn naming_app(&self) -> super::NamingApp {
                let __query = self.__query_builder().select(\"namingApp\");
                super::NamingApp::__from_query(__query)
            }
        }",
    );
    assert!(!generated["naming_app.rs"].contains("QueryNamingAppOpts"));
    assert!(!generated["naming_app.rs"].contains("naming_app_opts"));

    // Required arguments, and an optional one.
    let generated = client(&dependency_core_schema(0), "dagger_sdk", "/vault", "");
    for expected in [
        "#[must_use] pub fn vault(&self, mount: impl Into<dagger_sdk::IdInput<dagger_sdk::Directory>>, name: impl Into<String>) -> super::Vault {
            let __query = self.__query_builder().select(\"vault\");
            let __query = __query.generated_argument_id_shape(\"mount\", Into::<dagger_sdk::IdInput<dagger_sdk::Directory>>::into(mount));
            let __query = __query.argument(\"name\", Into::<String>::into(name));
            super::Vault::__from_query(__query)
        }",
        "#[must_use] pub fn vault_opts(&self, mount: impl Into<dagger_sdk::IdInput<dagger_sdk::Directory>>, name: impl Into<String>, opts: &QueryVaultOpts) -> super::Vault {",
        "pub fn with_ttl(mut self, value: i64) -> Self { self.ttl = Option::Some(value); self }",
    ] {
        assert_binds("vault.rs", &generated["vault.rs"], expected);
    }
}

// `Client::r#ref` and `Client::load` are generic over a trait the client defines. The
// SDK's handles implement it through the SDK's own loading, and each handle of the
// module implements it in its binding: a reference keeps no ID, and a load asks for
// `id` through the type's inline fragment and keeps the ID it gets back.
#[test]
fn core_and_module_handles_load_from_an_id_through_the_client() {
    let generated = greeter();
    for expected in [
        "pub trait ClientLoadable: Sized {
            #[doc(hidden)]
            fn __client_ref(query: &::dagger_sdk::QueryBuilder, id: ::dagger_sdk::IdInput<Self>) -> Self;
            #[doc(hidden)]
            fn __client_load(
                query: &::dagger_sdk::QueryBuilder,
                id: ::dagger_sdk::IdInput<Self>,
            ) -> impl Future<Output = Result<Self, ::dagger_sdk::QueryError>> + Send;
        }",
        "impl<T> ClientLoadable for T where T: ::dagger_sdk::Loadable + Send + 'static {
            fn __client_ref(query: &::dagger_sdk::QueryBuilder, id: ::dagger_sdk::IdInput<Self>) -> Self {
                query.generated_query_root().r#ref(id)
            }
            fn __client_load(
                query: &::dagger_sdk::QueryBuilder,
                id: ::dagger_sdk::IdInput<Self>,
            ) -> impl Future<Output = Result<Self, ::dagger_sdk::QueryError>> + Send {
                let root = query.generated_query_root();
                async move { root.load(id).await }
            }
        }",
        "pub fn r#ref<T: ClientLoadable>(&self, id: impl Into<::dagger_sdk::IdInput<T>>) -> T {
            T::__client_ref(self.__query_builder(), Into::into(id))
        }",
        "pub async fn load<T: ClientLoadable>(&self, id: impl Into<::dagger_sdk::IdInput<T>>) -> Result<T, ::dagger_sdk::QueryError> {
            T::__client_load(self.__query_builder(), Into::into(id)).await
        }",
        "#[doc(hidden)] fn __query_builder(&self) -> &::dagger_sdk::QueryBuilder { &self.query }",
    ] {
        assert_binds("mod.rs", &generated["mod.rs"], expected);
    }
    for (file, handle, wire_name) in [
        ("greeter.rs", "Greeter", "Greeter"),
        ("greeter_message.rs", "GreeterMessage", "GreeterMessage"),
        ("greeter_named.rs", "GreeterNamedClient", "GreeterNamed"),
    ] {
        assert_binds(
            file,
            &generated[file],
            &format!(
                "impl super::ClientLoadable for {handle} {{
                    fn __client_ref(__query: &dagger_sdk::QueryBuilder, __id: dagger_sdk::IdInput<Self>) -> Self {{
                        Self::__from_query(__query.generated_lazy_reentry_builder(__id, {wire_name:?}))
                    }}
                    fn __client_load(
                        __query: &dagger_sdk::QueryBuilder,
                        __id: dagger_sdk::IdInput<Self>,
                    ) -> impl Future<Output = Result<Self, dagger_sdk::QueryError>> + Send {{
                        let __query = Clone::clone(__query);
                        async move {{
                            let __id: dagger_sdk::Id = __query
                                .generated_lazy_reentry_builder(__id, {wire_name:?})
                                .select(\"id\")
                                .execute()
                                .await?;
                            Result::Ok(Self::__from_query(__query.generated_reentry_builder(__id, {wire_name:?})))
                        }}
                    }}
                }}"
            ),
        );
    }
    // An enum has no ID to load.
    assert!(!generated["greeter_style.rs"].contains("ClientLoadable"));
    // Nothing of a client names what a module's generated code has in its place.
    for (file, source) in &generated {
        for foreign in ["ModuleQuery", "ModuleLoadable", "__module_"] {
            assert!(!source.contains(foreign), "{file} names `{foreign}`");
        }
    }
}

// A client's API is the module alone, as the `Client` Dagger's Go SDK generates for a
// package holds only the bound module. `mod.rs` re-exports the module's types and
// nothing of the SDK, which it reaches under the name the package gives it.
#[test]
fn a_client_exports_the_module_alone_and_reaches_the_sdk_under_its_alias() {
    let generated = greeter();
    let index = syn::parse_file(&generated["mod.rs"]).expect("mod.rs is valid Rust");
    let mut exported = BTreeSet::new();
    for item in &index.items {
        if let syn::Item::Use(item) = item {
            let syn::UseTree::Path(path) = &item.tree else {
                panic!("mod.rs re-exports something that is not its own module's");
            };
            assert_eq!(path.ident, "self");
            let syn::UseTree::Path(module) = path.tree.as_ref() else {
                panic!("mod.rs re-exports something that is not a type file's");
            };
            assert!(matches!(module.tree.as_ref(), syn::UseTree::Glob(_)));
            exported.insert(module.ident.to_string());
        }
    }
    assert_eq!(
        exported,
        [
            "greeter",
            "greeter_message",
            "greeter_named",
            "greeter_style"
        ]
        .map(str::to_owned)
        .into()
    );

    let schema = module_client_schema(GREETER_CLIENT_SCHEMA_PART);
    let renamed = client(&schema, "renamed_sdk", GREETER_ADDRESS, "");
    for (file, source) in &renamed {
        assert!(
            !source.contains("dagger_sdk"),
            "{file} names the SDK's library"
        );
        assert_eq!(
            source.replace("renamed_sdk", "dagger_sdk"),
            generated[file],
            "{file}"
        );
    }
    let refused = render(&schema, "not-an-identifier", GREETER_ADDRESS, "")
        .expect_err("the SDK's name must be a Rust identifier");
    let diagnostic = refusal(&refused, DiagnosticCode::RustNameInvalid);
    assert_eq!(coordinate(diagnostic), Some("client.sdk-dependency"));
}

/// The lints that attributes allow, each by its path.
fn allowed_lints(attributes: &[syn::Attribute]) -> BTreeSet<String> {
    let mut allowed = BTreeSet::new();
    for attribute in attributes {
        if attribute.path().is_ident("allow") {
            attribute
                .parse_nested_meta(|meta| {
                    allowed.insert(
                        meta.path
                            .segments
                            .iter()
                            .map(|segment| segment.ident.to_string())
                            .collect::<Vec<_>>()
                            .join("::"),
                    );
                    Ok(())
                })
                .expect("an allow lists lint names");
        }
    }
    allowed
}

// A client compiles as part of its author's crate, whatever the crate calls of it. So
// `mod.rs` allows, for the whole client, what generated code cannot avoid, and each
// re-export allows `unused_imports`: nothing else in the client names a type that none
// of the module's functions returns, and the crate need not name it either.
#[test]
fn a_client_allows_the_lints_generated_code_cannot_avoid() {
    let index = greeter()["mod.rs"].clone();
    let file = syn::parse_file(&index).expect("mod.rs is valid Rust");
    let reexports = file
        .items
        .iter()
        .filter_map(|item| match item {
            syn::Item::Use(reexport) => Some(allowed_lints(&reexport.attrs)),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        reexports,
        vec![BTreeSet::from(["unused_imports".to_owned()]); 4],
        "each of the greeter's four type files is re-exported with `unused_imports` allowed"
    );
    assert_eq!(
        allowed_lints(&file.attrs),
        [
            "clippy::module_inception",
            "clippy::new_ret_no_self",
            "clippy::should_implement_trait",
            "clippy::too_many_arguments",
            "clippy::wrong_self_convention",
            "dead_code",
            "deprecated",
        ]
        .map(str::to_owned)
        .into()
    );
}

// The engine names a module's types by the module's own rules, whatever a client's
// version would make of them: `HTTPFetcher` of `naming-app` is `NamingAppHTTPFetcher`,
// where the casing of this SDK's target would give `NamingAppHttpFetcher`. A binding
// sends exactly the names the schema carries, in the fields it selects and in the type
// it names to load an object by ID, which becomes the inline fragment of `node(id:)`.
// Only the Rust names are in Rust's casing.
#[test]
fn an_acronym_type_is_sent_under_the_name_the_engine_serves() {
    let generated = naming_app();
    assert_eq!(
        generated.keys().map(String::as_str).collect::<Vec<_>>(),
        ["mod.rs", "naming_app.rs", "naming_app_http_fetcher.rs"]
    );
    assert_eq!(
        string_literals(&generated["mod.rs"]),
        [NAMING_APP_ADDRESS, ""].map(str::to_owned).into()
    );
    // The main object: its constructor, its function and its arguments, and its own
    // type wherever it is loaded by ID.
    assert_eq!(
        string_literals(&generated["naming_app.rs"]),
        ["NamingApp", "fetcher", "id", "namingApp", "retries", "url"]
            .map(str::to_owned)
            .into()
    );
    // The acronym type: its fields, and its wire name wherever it is loaded by ID.
    let fetcher = &generated["naming_app_http_fetcher.rs"];
    assert_eq!(
        string_literals(fetcher),
        ["NamingAppHTTPFetcher", "id", "retries", "url"]
            .map(str::to_owned)
            .into()
    );
    for expected in [
        "pub struct NamingAppHttpFetcher {",
        // `node(id:)` re-entry, with and without keeping the ID.
        "Self::__from_query(__query.generated_lazy_reentry_builder(__id, \"NamingAppHTTPFetcher\"))",
        "Result::Ok(Self::__from_query(__query.generated_reentry_builder(__id, \"NamingAppHTTPFetcher\")))",
        // The module boundary's decoder loads it by the same name.
        "dagger_sdk::__private::decode_handle_input(value, context, \"NamingAppHTTPFetcher\")",
        "pub async fn url(&self) -> Result<String, dagger_sdk::QueryError> {
            let __query = self.query.select(\"url\");
            __query.execute().await
        }",
        "pub async fn retries(&self) -> Result<i64, dagger_sdk::QueryError> {
            let __query = self.query.select(\"retries\");
            __query.execute().await
        }",
    ] {
        assert_binds("naming_app_http_fetcher.rs", fetcher, expected);
    }
    for expected in [
        "#[must_use] pub fn fetcher(&self, url: impl Into<String>) -> super::NamingAppHttpFetcher {
            let __query = self.query.select(\"fetcher\");
            let __query = __query.argument(\"url\", Into::<String>::into(url));
            super::NamingAppHttpFetcher::__from_query(__query)
        }",
        "pub fn fetcher_opts(&self, url: impl Into<String>, opts: &NamingAppFetcherOpts) -> super::NamingAppHttpFetcher {",
        "let __query = if let Option::Some(__value) = &opts.retries { __query.argument(\"retries\", __value) } else { __query };",
    ] {
        assert_binds("naming_app.rs", &generated["naming_app.rs"], expected);
    }
    // No file sends the name in Rust's casing.
    for (file, source) in &generated {
        assert!(
            !string_literals(source).contains("NamingAppHttpFetcher"),
            "{file} sends a name the engine does not serve"
        );
    }
}

// `mod.rs` defines `Client` and `ClientLoadable` and re-exports the module's types
// beside them, so a type of those names cannot be bound, and neither can one whose
// file would be `mod.rs`. The names are the module's own, so the refusal states that
// no client can be generated, at the type's coordinate.
#[test]
fn a_type_named_like_the_client_s_own_items_is_refused() {
    for (module, type_name, code, stated) in [
        (
            "client",
            "Client",
            DiagnosticCode::RustNameCollision,
            // The projection keeps this name from every generated namespace already.
            "Rust identifier `Client` is already reserved",
        ),
        (
            "client",
            "ClientLoadable",
            DiagnosticCode::RustNameCollision,
            "type `ClientLoadable` of module `client` would be bound as `ClientLoadable`, which every generated client defines; no client can be generated for a module with a type of this name",
        ),
        (
            "mod",
            "Mod",
            DiagnosticCode::OperationArtifactCollision,
            "type `Mod` of module `mod` would be bound in `mod.rs`, which holds the generated client; no client can be generated for a module with a type of this name",
        ),
    ] {
        let schema = changed_dependency_core_schema(|document| {
            add_constructed_object(document, Some(module), type_name, "pack", vec![]);
        });
        let refused = render(&schema, "dagger_sdk", "/pack", "")
            .expect_err("a type that shadows the client's own items must be refused");
        let diagnostic = refusal(&refused, code);
        assert_eq!(coordinate(diagnostic), Some(type_name), "{refused:?}");
        assert!(diagnostic.message.contains(stated), "{diagnostic:?}");
    }
    // What a module's generated code defines is no name of a client's.
    let schema = changed_dependency_core_schema(|document| {
        add_constructed_object(document, Some("pack"), "ModuleQuery", "pack", vec![]);
        add_constructed_object(document, Some("pack"), "ModuleLoadable", "packed", vec![]);
    });
    let generated = client(&schema, "dagger_sdk", "/pack", "");
    assert!(generated.contains_key("module_query.rs"));
    assert!(generated.contains_key("module_loadable.rs"));
}

// The module's constructor is a method of `Client` named after the module, so a module
// called `new`, `ref` or `load` would define a method of `Client` a second time.
#[test]
fn a_constructor_named_like_a_method_of_the_client_is_refused() {
    for (module, type_name, method) in [
        ("new", "New", "new"),
        ("ref", "Ref", "ref"),
        ("load", "Load", "load"),
    ] {
        let schema = changed_dependency_core_schema(|document| {
            add_constructed_object(document, Some(module), type_name, module, vec![]);
        });
        let refused = render(&schema, "dagger_sdk", "/pack", "")
            .expect_err("a constructor that redefines a method of the client must be refused");
        let diagnostic = refusal(&refused, DiagnosticCode::RustNameCollision);
        assert_eq!(
            coordinate(diagnostic),
            Some(format!("Query.{module}").as_str())
        );
        assert_eq!(
            diagnostic.message,
            format!(
                "the constructor of module `{module}` would be bound as `Client::{method}`, which every generated client defines; no client can be generated for a module of this name"
            )
        );
    }
    // A module called `clone` is constructed by `clone_`, since a client can be cloned.
    let schema = changed_dependency_core_schema(|document| {
        add_constructed_object(document, Some("clone"), "Clone", "clone", vec![]);
    });
    let generated = client(&schema, "dagger_sdk", "/clone", "");
    assert_binds(
        "clone.rs",
        &generated["clone.rs"],
        "#[must_use] pub fn clone_(&self) -> super::Clone {",
    );
}

// A schema with no module in it has nothing to bind a client to.
#[test]
fn a_schema_without_a_module_is_refused() {
    let refused = render(CORE_SCHEMA_BYTES, "dagger_sdk", "/none", "")
        .expect_err("Core alone has no module to generate a client for");
    let diagnostic = refusal(&refused, DiagnosticCode::ClientModuleRootInvalid);
    assert_eq!(coordinate(diagnostic), Some("Query"));
}

// The types are bound by the renderer of a module's dependencies, which refuses the
// same things here: a function that names a type with no typed binding, at the
// function's coordinate, and two functions that would be one method.
#[test]
fn a_client_refuses_what_the_bindings_of_a_dependency_refuse() {
    let schema = changed_dependency_core_schema(|document| {
        add_constructed_object(document, None, "LaterThing", "laterThing", vec![]);
        add_constructed_object(
            document,
            Some("probe"),
            "Probe",
            "probe",
            vec![dependency_field(
                "thing",
                dependency_object_type("LaterThing"),
                vec![],
            )],
        );
    });
    let refused = render(&schema, "dagger_sdk", "/probe", "")
        .expect_err("a typed method cannot name an untyped handle");
    let diagnostic = refusal(&refused, DiagnosticCode::SchemaFieldUnmapped);
    assert_eq!(coordinate(diagnostic), Some("Probe.thing"));
    assert!(diagnostic.message.contains("`LaterThing`"));

    let schema = changed_dependency_core_schema(|document| {
        add_constructed_object(
            document,
            Some("probe"),
            "Probe",
            "probe",
            vec![
                dependency_field("clone", dependency_object_type("Probe"), vec![]),
                dependency_field("clone_", dependency_object_type("Probe"), vec![]),
            ],
        );
    });
    let refused = render(&schema, "dagger_sdk", "/probe", "")
        .expect_err("two functions cannot be bound as one method");
    refusal(&refused, DiagnosticCode::RustNameCollision);
}

/// The source of a package that calls its clients as an author would: the constructor
/// with and without its optional arguments, functions that return a value and an
/// object, `r#ref` and `load` for the module's handles and for Core's, and the acronym
/// module. `awkward` is a client the package never calls.
fn package_main(alias: &str) -> String {
    format!(
        "mod awkward;
mod greeter;
mod naming_app;

async fn calls() -> Result<(), Box<dyn std::error::Error>> {{
    let client = {alias}::connect().await?;

    let greeter = greeter::Client::new(&client);
    let answer: String = greeter
        .greeter_opts(&greeter::QueryGreeterOpts::default().with_greeting(\"hi\"))
        .hello(\"Ian\")
        .await?;
    let plain: greeter::Greeter = greeter.greeter();
    let ended: String = plain
        .hello_opts(\"Ian\", &greeter::GreeterHelloOpts::default().with_ending(\"!\"))
        .await?;
    let message: greeter::GreeterMessage = plain.message(\"Ian\");
    let name: String = message.name().await?;
    let text: String = message.text().await?;

    let id: {alias}::Id = message.id().await?;
    let referenced: greeter::GreeterMessage = greeter.r#ref(message.clone());
    let loaded: greeter::GreeterMessage = greeter.load(referenced).await?;
    let named: greeter::GreeterNamedClient = plain.named(loaded);
    let style: greeter::GreeterStyle = plain.restyle(greeter::GreeterStyle::Plain).await?;
    let stamped: {alias}::Directory = plain.stamp(client.query().directory());
    let directory: {alias}::Directory = greeter.load(stamped).await?;
    let entries: Vec<String> = plain
        .entries_opts(
            &greeter::GreeterEntriesOpts::default()
                .with_source(greeter.r#ref::<{alias}::Directory>(directory).into()),
        )
        .await?;

    let app = naming_app::Client::new(&client).naming_app();
    let fetcher: naming_app::NamingAppHttpFetcher = app.fetcher(\"https://example.com\");
    let url: String = fetcher.url().await?;
    let retries: i64 = app
        .fetcher_opts(
            \"https://example.com\",
            &naming_app::NamingAppFetcherOpts::default().with_retries(5),
        )
        .retries()
        .await?;

    println!(
        \"{{answer}} {{ended}} {{name}} {{text}} {{id:?}} {{style:?}} {{entries:?}} {{url}} {{retries}} {{:?}}\",
        named.name().await?
    );
    Ok(())
}}

fn main() {{
    let _calls: std::pin::Pin<Box<dyn std::future::Future<Output = _>>> = Box::pin(calls());
}}
"
    )
}

// Generated code is compiled as part of its author's crate, in the crate's edition and
// under the crate's lints. A package that declares each client as a private module and
// calls part of one must build with no warning from rustc or Clippy, in each edition an
// author may use, with the SDK under whatever name the package gives it. The greeter
// here also has a type that none of its functions returns, which only its own file
// names, so that file's re-export is never used.
#[test]
fn a_package_compiles_its_clients_without_warnings_in_each_edition() {
    let temporary = tempfile::tempdir().expect("fixture root must be available");
    let root = temporary.path();
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let sdk = manifest_dir
        .parent()
        .expect("codegen and SDK crates share a parent")
        .join("dagger-sdk");
    // The workspace lockfile pins the SDK's dependency versions, so offline resolution
    // does not depend on what the registry has yanked since.
    let workspace = manifest_dir
        .parent()
        .and_then(Path::parent)
        .expect("codegen sits two levels below the workspace root");
    fs::copy(workspace.join("Cargo.lock"), root.join("Cargo.lock"))
        .expect("workspace lockfile must copy");
    fs::write(
        root.join("Cargo.toml"),
        "[workspace]\nresolver = \"2\"\nmembers = [\"edition-2021\", \"edition-2024\"]\n",
    )
    .expect("fixture workspace must write");

    for (package, edition, alias) in [
        ("edition-2021", "2021", "renamed_sdk"),
        ("edition-2024", "2024", "dagger_sdk"),
    ] {
        let package_root = root.join(package);
        fs::create_dir_all(package_root.join("src")).expect("package source root must exist");
        fs::write(
            package_root.join("Cargo.toml"),
            format!(
                "[package]\nname = {package:?}\nversion = \"0.0.0\"\nedition = {edition:?}\n\n[dependencies]\n{alias} = {{ package = \"tokeira-dagger-sdk\", path = {sdk:?} }}\n"
            ),
        )
        .expect("package manifest must write");
        fs::write(package_root.join("src/main.rs"), package_main(alias))
            .expect("package source must write");
        for (module, schema, address) in [
            (
                "greeter",
                add_unreturned_object(
                    &module_client_schema(GREETER_CLIENT_SCHEMA_PART),
                    "greeter",
                    "GreeterNote",
                ),
                GREETER_ADDRESS,
            ),
            (
                "naming_app",
                module_client_schema(NAMING_APP_CLIENT_SCHEMA_PART),
                NAMING_APP_ADDRESS,
            ),
            // Every name a module's author can choose that generated Rust could trip
            // over, with a type whose file is named as the SDK is.
            (
                "awkward",
                adversarial_dependency_core_schema(alias),
                "/awkward",
            ),
        ] {
            let directory = package_root.join("src").join(module);
            fs::create_dir_all(&directory).expect("client directory must exist");
            for (file, source) in client(&schema, alias, address, "") {
                fs::write(directory.join(file), source).expect("client source must write");
            }
        }
    }

    let target_dir = root.join("target");
    let lints = Command::new(env!("CARGO"))
        .args([
            "clippy",
            "--workspace",
            "--all-targets",
            "--offline",
            "--quiet",
            "--",
            "-D",
            "warnings",
        ])
        .env("CARGO_TARGET_DIR", &target_dir)
        .current_dir(root)
        .output()
        .expect("offline fixture clippy must start");
    assert!(
        lints.status.success(),
        "a package does not compile its clients cleanly:\n{}",
        String::from_utf8_lossy(&lints.stderr)
    );
}

// The fixtures are what an engine at the target served, less the Core part it shares
// with the checked schema. Given a directory of new captures, this rewrites them, and
// refuses a capture that the tests' splice does not give back type for type.
#[test]
fn the_schema_fixtures_are_the_module_s_part_of_an_engine_s_client_schema() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/module_clients");
    if let Some(captures) = std::env::var_os("DAGGER_UPDATE_MODULE_CLIENT_SCHEMA_FIXTURES") {
        for fixture in ["greeter.json", "naming-app.json"] {
            let captured = fs::read(Path::new(&captures).join(fixture))
                .unwrap_or_else(|error| panic!("capture {fixture} must be readable: {error}"));
            fs::write(root.join(fixture), module_client_schema_part(&captured))
                .expect("fixture is writable");
        }
    }
    for (fixture, part, module, owned) in [
        (
            "greeter.json",
            GREETER_CLIENT_SCHEMA_PART,
            "greeter",
            &["Greeter", "GreeterMessage", "GreeterNamed", "GreeterStyle"][..],
        ),
        (
            "naming-app.json",
            NAMING_APP_CLIENT_SCHEMA_PART,
            "naming-app",
            &["NamingApp", "NamingAppHTTPFetcher"][..],
        ),
    ] {
        let document: serde_json::Value = serde_json::from_slice(part).expect("fixture decodes");
        let types = document["types"].as_array().expect("fixture lists types");
        assert_eq!(
            types
                .iter()
                .map(|definition| definition["name"].as_str().unwrap_or_default())
                .collect::<Vec<_>>(),
            owned,
            "{fixture}"
        );
        // The engine's own statement of ownership, on every type.
        for definition in types {
            assert_eq!(
                definition["directives"],
                serde_json::json!([{
                    "name": "sourceMap",
                    "args": [{"name": "module", "value": format!("{module:?}")}]
                }]),
                "{fixture}: {}",
                definition["name"]
            );
        }
        // Reducing the spliced schema gives the fixture back.
        assert_eq!(
            module_client_schema_part(&module_client_schema(part)),
            part,
            "{fixture} is not in the form the reduction writes"
        );
    }
}
