//! Typed bindings for the types of the modules a module depends on.
//!
//! The engine serves a module the schema of Core plus its direct dependencies and
//! stamps each dependency's types with `@sourceMap(module:)`. Generation binds those
//! types typed, in the calling convention of the SDK's Core bindings, and keeps the
//! untyped handle for any other type outside the reviewed Core API. These tests render
//! a module beside three dependencies whose functions have every shape a caller meets,
//! and check ownership, the method each shape gets, and what generation refuses.
//!
//! `fixtures/dependency_bindings/` holds the rendered bindings as checked files, so a
//! change to the renderer shows in review as a change to what a module author compiles
//! against. Regenerate them with
//! `DAGGER_UPDATE_DEPENDENCY_BINDINGS_FIXTURE=1 cargo test -p dagger-codegen --locked
//! --test dependency_bindings`, then format them with `rustfmt --edition 2024`.

mod support;

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use dagger_codegen::diagnostic::{DiagnosticCode, DiagnosticSet};
use dagger_codegen::engine::{
    ModuleProjectionInput, OperationKind, OperationProjectionRequest, PublishedSdkDependency,
    RelativeOperationPath, project_operation, project_visible_schema,
};
use dagger_codegen::schema::canonical::SchemaName;
use dagger_codegen::target::CodegenTarget;

use support::{
    MODULE_AUTHORING_SOURCE, PRELUDE_NAMED_TYPES, TARGET_BYTES, add_constructed_object,
    adversarial_dependency_module_schema, apply_type_directive, changed_dependency_module_schema,
    dependency_core_schema, dependency_field, dependency_module_schema, dependency_object_type,
    directive_application, module_authoring_input_from,
};

/// The files that bind the fixture's dependency types, by the type each binds.
const BINDINGS: &[&str] = &[
    "clock.rs",
    "greeter.rs",
    "greeter_message.rs",
    "greeter_mode.rs",
    "greeter_named.rs",
    "vault.rs",
];

fn target() -> CodegenTarget {
    CodegenTarget::decode_exact(TARGET_BYTES).expect("checked target must decode")
}

/// Generates a module against `schema` with its SDK dependency keyed under `alias`,
/// and returns the files of `src/dagger_generated` by name.
fn generate(schema: &[u8], alias: &str) -> Result<BTreeMap<String, String>, DiagnosticSet> {
    let output = RelativeOperationPath::parse("candidate").expect("fixture path must parse");
    let module = ModuleProjectionInput {
        name: "rust-probe".to_owned(),
        original_name: "RustProbe".to_owned(),
        source_subpath: RelativeOperationPath::parse("modules/rust-probe")
            .expect("fixture path must parse"),
        source_digest: "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
            .to_owned(),
    };
    let dependency = PublishedSdkDependency::Registry {
        registry: "crates-io".to_owned(),
        exact_version: "1.0.0-beta.11.rust.5".to_owned(),
    };
    let mut authoring =
        module_authoring_input_from(&MODULE_AUTHORING_SOURCE.replace("dagger_sdk", alias));
    authoring.sdk_dependency_alias = alias.to_owned();
    let plan = project_operation(OperationProjectionRequest {
        target: &target(),
        operation: OperationKind::GenerateModule,
        visible_schema_json: schema,
        module: Some(&module),
        output: &output,
        sdk_dependency: &dependency,
        authoring: Some(&authoring),
        engine_version: None,
    })?;
    Ok(plan
        .artifacts()
        .iter()
        .filter_map(|(path, artifact)| {
            let name = path
                .as_str()
                .strip_prefix("candidate/src/dagger_generated/")
                .filter(|name| name.ends_with(".rs"))?;
            let source = String::from_utf8(artifact.content.clone()).expect("Rust is UTF-8");
            Some((name.to_owned(), source))
        })
        .collect())
}

fn bindings() -> BTreeMap<String, String> {
    generate(&dependency_module_schema(0), "dagger_sdk")
        .expect("a module beside its dependencies must generate")
}

/// Source as its tokens, so a comparison ignores how either side is laid out. Group
/// delimiters and commas are dropped: a formatter wraps a closure's body in braces and
/// adds trailing commas.
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

    // An expected fragment may stop at the brace that opens an item's body.
    let source = source.trim();
    let source = source.strip_suffix('{').unwrap_or(source);
    let stream = source
        .parse::<proc_macro2::TokenStream>()
        .unwrap_or_else(|error| panic!("`{source}` must tokenize: {error}"));
    let mut flat = Vec::new();
    flatten(stream, &mut flat);
    format!(" {} ", flat.join(" "))
}

/// The modules of the standard library that a binding names from the crate root, so
/// that no type of a dependency can stand in for what the binding means.
const ABSOLUTE: &[&str] = &[
    "::core::clone::",
    "::core::convert::",
    "::core::future::",
    "::core::iter::",
    "::core::marker::",
    "::core::option::",
    "::core::pin::",
    "::core::result::",
    "::std::boxed::",
    "::std::string::",
    "::std::vec::",
];

/// A binding's tokens with each standard item under its short name, which is how the
/// expectations below are written. `no_standard_name_is_written_bare` checks the
/// absolute paths themselves.
fn short(source: &str) -> String {
    let mut flat = tokens(source);
    for prefix in ABSOLUTE {
        flat = flat.replace(&tokens(prefix), " ");
    }
    flat
}

#[track_caller]
fn assert_binds(file: &str, source: &str, expected: &str) {
    assert!(
        short(source).contains(&tokens(expected)),
        "{file} lacks `{expected}`"
    );
}

/// The inherent methods of `owner` in a binding, in source order.
fn inherent_methods(source: &str, owner: &str) -> Vec<String> {
    let file = syn::parse_file(source).expect("a binding is valid Rust");
    file.items
        .iter()
        .filter_map(|item| match item {
            syn::Item::Impl(implementation) if implementation.trait_.is_none() => {
                Some(implementation)
            }
            _ => None,
        })
        .filter(|implementation| match implementation.self_ty.as_ref() {
            syn::Type::Path(path) => path.path.is_ident(owner),
            _ => false,
        })
        .flat_map(|implementation| &implementation.items)
        .filter_map(|item| match item {
            syn::ImplItem::Fn(function) => Some(function.sig.ident.to_string()),
            _ => None,
        })
        .collect()
}

fn name(value: &str) -> SchemaName {
    SchemaName::try_from(value).expect("fixture name is valid")
}

// Ownership is the engine's statement, read from `@sourceMap`, wherever the types sit
// in the schema: the same types are bound typed, in the same bytes, in every order.
#[test]
fn ownership_follows_the_source_map_whatever_the_schema_order() {
    let reference = bindings();
    for bound in BINDINGS {
        assert!(
            reference.contains_key(*bound),
            "{bound} must bind a dependency type"
        );
    }
    for permutation in [1, 7, 64, 513, u16::MAX] {
        // The whole reviewed Core API, which `project_visible_schema` verifies against,
        // with the same dependencies.
        let plan = project_visible_schema(&target(), &dependency_core_schema(permutation))
            .expect("schema projects");
        for (type_name, module) in [
            ("Greeter", "greeter"),
            ("GreeterMessage", "greeter"),
            ("GreeterNamed", "greeter"),
            ("GreeterMode", "greeter"),
            ("Vault", "vault"),
            ("Clock", "clock"),
        ] {
            assert_eq!(plan.dependency_module(&name(type_name)), Some(module));
        }
        // A reviewed Core type is no dependency's, whatever directive it carries.
        assert_eq!(plan.dependency_module(&name("Container")), None);
        assert_eq!(
            generate(&dependency_module_schema(permutation), "dagger_sdk")
                .expect("a permuted schema must generate"),
            reference,
            "permutation {permutation} changed the generated files"
        );
    }
}

// The engine stamps a dependency's interface with `@sourceMap` as it stamps its objects,
// and the interface is the dependency's by it. The directive says who owns a type and
// nothing about what a caller sends, so a reviewed Core interface that carries it is
// accepted and stays Core's.
#[test]
fn a_source_map_makes_an_interface_a_dependency_s_unless_it_is_core_s() {
    let plan = project_visible_schema(&target(), &dependency_core_schema(0))
        .expect("a dependency's interface must be accepted with its source map");
    assert_eq!(
        plan.dependency_module(&name("GreeterNamed")),
        Some("greeter")
    );

    let on_core = apply_type_directive(
        &dependency_core_schema(0),
        "Exportable",
        directive_application("sourceMap", &[("module", "\"greeter\"")]),
    );
    let plan = project_visible_schema(&target(), &on_core)
        .expect("ownership metadata takes no part in a Core coordinate's contract");
    assert_eq!(plan.dependency_module(&name("Exportable")), None);
}

// A type outside the reviewed Core API that carries no `@sourceMap` is no dependency's:
// a later engine's own addition, for one. It keeps the untyped handle, beside typed
// dependency bindings in the same module.
#[test]
fn an_extension_type_without_a_source_map_keeps_the_untyped_handle() {
    let schema = changed_dependency_module_schema(|document| {
        add_constructed_object(
            document,
            None,
            "LaterThing",
            "laterThing",
            vec![dependency_field(
                "container",
                dependency_object_type("Container"),
                vec![],
            )],
        );
    });
    let generated = generate(&schema, "dagger_sdk").expect("the module must generate");
    let later = &generated["later_thing.rs"];
    for expected in [
        "pub struct LaterThing { query: dagger_sdk::QueryBuilder }",
        "pub fn from_query(query: dagger_sdk::QueryBuilder) -> Self { Self { query } }",
        "pub fn selection(&self) -> &dagger_sdk::QueryBuilder { &self.query }",
        "pub fn container(&self) -> dagger_sdk::QueryBuilder {",
    ] {
        assert!(
            later.contains(expected),
            "later_thing.rs lacks `{expected}`"
        );
    }
    // The dependency bindings beside it are the ones rendered without it, apart from
    // the schema identity their headers state.
    let reference = bindings();
    let body = |source: &str| {
        source
            .lines()
            .filter(|line| !line.starts_with("// @generated"))
            .collect::<Vec<_>>()
            .join("\n")
    };
    for bound in BINDINGS {
        assert_eq!(body(&generated[*bound]), body(&reference[*bound]));
    }
}

// Every function of a dependency's object is one method, with an `_opts` variant
// exactly when it has an optional argument, and nothing else is public on the handle:
// a `selection` or `from_query` could be a dependency's own function.
#[test]
fn each_dependency_function_is_one_method_and_one_more_with_options() {
    let generated = bindings();
    assert_eq!(
        inherent_methods(&generated["greeter.rs"], "Greeter"),
        [
            "__from_query",
            "container",
            "container_opts",
            "describe",
            "files",
            "hello",
            "id",
            "last_message",
            "maybe_file",
            "message",
            "messages",
            "mode",
            "named",
            "platform",
            "platform_opts",
            "protocol",
            "reset",
            "shout",
            "shout_opts",
            "sync",
            "with_message",
            "with_messages",
            "with_mode",
        ]
    );
    assert_eq!(
        inherent_methods(&generated["greeter_named.rs"], "GreeterNamedClient"),
        [
            "__from_query",
            "aliases",
            "describe",
            "describe_opts",
            "id",
            "message",
            "relabel",
            "relabel_opts",
            "renamed",
        ]
    );
    assert_eq!(
        inherent_methods(&generated["clock.rs"], "Clock"),
        ["__from_query", "id", "now"]
    );
    for (file, source) in &generated {
        if BINDINGS.contains(&file.as_str()) {
            assert!(!tokens(source).contains(" fn selection "), "{file}");
            assert!(!tokens(source).contains(" fn from_query "), "{file}");
        }
    }
    assert_binds(
        "greeter.rs",
        &generated["greeter.rs"],
        "#[doc(hidden)] pub fn __from_query(query: dagger_sdk::QueryBuilder) -> Self { Self { query } }",
    );
}

// One case for each way the projection says a field is executed.
#[test]
fn each_field_strategy_renders_in_the_core_calling_convention() {
    let generated = bindings();
    let greeter = &generated["greeter.rs"];
    for expected in [
        // A value executes, and a string argument takes anything that converts.
        "pub async fn hello(&self, name: impl Into<String>) -> Result<String, dagger_sdk::QueryError> {
            let __query = self.query.select(\"hello\");
            let __query = __query.argument(\"name\", Into::<String>::into(name));
            __query.execute().await
        }",
        // Optional arguments are borrowed, and an absent one is left out of the request.
        "pub async fn shout_opts(&self, name: impl Into<String>, opts: &GreeterShoutOpts) -> Result<String, dagger_sdk::QueryError> {
            let __query = self.query.select(\"shout\");
            let __query = if let Option::Some(__value) = &opts.mode { __query.argument(\"mode\", __value) } else { __query };
            let __query = __query.argument(\"name\", Into::<String>::into(name));
            let __query = if let Option::Some(__value) = &opts.suffixes { __query.argument(\"suffixes\", __value) } else { __query };
            let __query = if let Option::Some(__value) = &opts.times { __query.argument(\"times\", __value) } else { __query };
            __query.execute().await
        }",
        "pub struct GreeterShoutOpts {",
        "pub mode: Option<super::GreeterMode>,",
        "pub suffixes: Option<Vec<String>>,",
        "pub times: Option<i64>,",
        "pub fn with_mode(mut self, value: super::GreeterMode) -> Self { self.mode = Option::Some(value); self }",
        "pub fn with_suffixes(mut self, value: Vec<impl Into<String>>) -> Self {
            self.suffixes = Option::Some(Iterator::collect(Iterator::map(IntoIterator::into_iter(value), Into::into)));
            self
        }",
        "pub fn with_times(mut self, value: i64) -> Self { self.times = Option::Some(value); self }",
        // The dependency's enum, out and in.
        "pub async fn mode(&self) -> Result<super::GreeterMode, dagger_sdk::QueryError> {
            let __query = self.query.select(\"mode\");
            __query.execute().await
        }",
        "#[must_use] pub fn with_mode(&self, mode: super::GreeterMode) -> super::Greeter {
            let __query = self.query.select(\"withMode\");
            let __query = __query.argument(\"mode\", mode);
            super::Greeter::__from_query(__query)
        }",
        // `Void` has its own execution, since the engine answers it with an object.
        "pub async fn reset(&self) -> Result<(), dagger_sdk::QueryError> {
            let __query = self.query.select(\"reset\");
            __query.execute_void().await
        }",
        // A non-null object extends the selection and sends nothing.
        "#[must_use] pub fn message(&self, name: impl Into<String>) -> super::GreeterMessage {
            let __query = self.query.select(\"message\");
            let __query = __query.argument(\"name\", Into::<String>::into(name));
            super::GreeterMessage::__from_query(__query)
        }",
        // A nullable object and a list of objects fetch IDs and re-enter each, keeping
        // the ID the query returned.
        "pub async fn last_message(&self) -> Result<Option<super::GreeterMessage>, dagger_sdk::QueryError> {
            let __query = self.query.select(\"lastMessage\");
            let __ids: Option<dagger_sdk::Id> = __query.select(\"id\").execute().await?;
            Result::Ok(Option::map(__ids, |__value_0| super::GreeterMessage::__from_query(
                __query.generated_reentry_builder(__value_0, \"GreeterMessage\")
            )))
        }",
        "pub async fn messages(&self, names: Vec<impl Into<String>>) -> Result<Vec<super::GreeterMessage>, dagger_sdk::QueryError> {
            let __query = self.query.select(\"messages\");
            let names = Iterator::collect::<Vec<String>>(Iterator::map(IntoIterator::into_iter(names), Into::into));
            let __query = __query.argument(\"names\", names);
            let __ids: Vec<dagger_sdk::Id> = __query.select(\"id\").execute().await?;
            Result::Ok(Iterator::collect(Iterator::map(IntoIterator::into_iter(__ids), |__value_0| super::GreeterMessage::__from_query(
                __query.generated_reentry_builder(__value_0, \"GreeterMessage\")
            ))))
        }",
        // The dependency's interface, as a result and as an argument.
        "#[must_use] pub fn named(&self) -> super::GreeterNamedClient {
            let __query = self.query.select(\"named\");
            super::GreeterNamedClient::__from_query(__query)
        }",
        "pub async fn describe(&self, named: impl Into<dagger_sdk::IdInput<super::GreeterNamedClient>>) -> Result<String, dagger_sdk::QueryError> {
            let __query = self.query.select(\"describe\");
            let __query = __query.generated_argument_id_shape(\"named\", Into::<dagger_sdk::IdInput<super::GreeterNamedClient>>::into(named));
            __query.execute().await
        }",
        // The dependency's objects as arguments, resolved when the request is built.
        "#[must_use] pub fn with_message(&self, message: impl Into<dagger_sdk::IdInput<super::GreeterMessage>>) -> super::Greeter {
            let __query = self.query.select(\"withMessage\");
            let __query = __query.generated_argument_id_shape(\"message\", Into::<dagger_sdk::IdInput<super::GreeterMessage>>::into(message));
            super::Greeter::__from_query(__query)
        }",
        "#[must_use] pub fn with_messages(&self, messages: Vec<Option<dagger_sdk::IdInput<super::GreeterMessage>>>) -> super::Greeter {
            let __query = self.query.select(\"withMessages\");
            let __query = __query.generated_argument_id_shape(\"messages\", messages);
            super::Greeter::__from_query(__query)
        }",
        // Core objects: the SDK builds its own handles, lazily and from returned IDs.
        "#[must_use] pub fn container_opts(&self, source: impl Into<dagger_sdk::IdInput<dagger_sdk::Directory>>, opts: &GreeterContainerOpts) -> dagger_sdk::Container {
            let __query = self.query.select(\"container\");
            let __query = if let Option::Some(__value) = &opts.base {
                __query.generated_argument_id_shape(\"base\", Clone::clone(__value))
            } else { __query };
            let __query = __query.generated_argument_id_shape(\"source\", Into::<dagger_sdk::IdInput<dagger_sdk::Directory>>::into(source));
            __query.generated_core_handle::<dagger_sdk::Container>()
        }",
        "pub base: Option<dagger_sdk::IdInput<dagger_sdk::Container>>,",
        "pub async fn maybe_file(&self) -> Result<Option<dagger_sdk::File>, dagger_sdk::QueryError> {
            let __query = self.query.select(\"maybeFile\");
            let __ids: Option<dagger_sdk::Id> = __query.select(\"id\").execute().await?;
            Result::Ok(Option::map(__ids, |__value_0| __query.generated_reentry_builder(__value_0, \"File\").generated_core_handle::<dagger_sdk::File>()))
        }",
        "pub async fn files(&self) -> Result<Vec<dagger_sdk::File>, dagger_sdk::QueryError> {
            let __query = self.query.select(\"files\");
            let __ids: Vec<dagger_sdk::Id> = __query.select(\"id\").execute().await?;
            Result::Ok(Iterator::collect(Iterator::map(IntoIterator::into_iter(__ids), |__value_0| __query.generated_reentry_builder(__value_0, \"File\").generated_core_handle::<dagger_sdk::File>())))
        }",
        // A Core enum and a Core scalar are the SDK's own types.
        "pub async fn protocol(&self, protocol: dagger_sdk::NetworkProtocol) -> Result<dagger_sdk::NetworkProtocol, dagger_sdk::QueryError> {",
        "pub async fn platform_opts(&self, opts: &GreeterPlatformOpts) -> Result<dagger_sdk::Platform, dagger_sdk::QueryError> {",
        // An ID that names its object re-enters as that object.
        "pub async fn sync(&self) -> Result<super::Greeter, dagger_sdk::QueryError> {
            let __query = self.query.select(\"sync\");
            let __id: dagger_sdk::Id = __query.execute().await?;
            Result::Ok(super::Greeter::__from_query(__query.generated_reentry_builder(__id, \"Greeter\")))
        }",
    ] {
        assert_binds("greeter.rs", greeter, expected);
    }
}

// A handle is passed wherever its ID is expected: as itself, and as each interface it
// implements, the dependency's or Core's.
#[test]
fn a_handle_converts_to_its_id_and_to_the_interfaces_it_implements() {
    let generated = bindings();
    let message = &generated["greeter_message.rs"];
    for expected in [
        "impl dagger_sdk::IntoID<dagger_sdk::Id> for GreeterMessage {",
        "Box::pin(async move { self.id().await })",
        "impl From<GreeterMessage> for dagger_sdk::IdInput<GreeterMessage> {
            fn from(value: GreeterMessage) -> Self { dagger_sdk::IdInput::generated_lazy(value) }
        }",
        "impl From<GreeterMessage> for dagger_sdk::IdInput<super::GreeterNamedClient> {",
        "impl From<GreeterMessage> for dagger_sdk::IdInput<dagger_sdk::NodeClient> {",
        // The dependency's interface is a trait the object implements; Core's is not.
        "impl super::GreeterNamed for GreeterMessage {",
    ] {
        assert_binds("greeter_message.rs", message, expected);
    }
    assert!(!tokens(message).contains(" impl dagger_sdk :: Node for "));
}

// An interface follows the Core shape: a trait of its fields, and a client handle that
// implements it.
#[test]
fn an_interface_is_a_trait_and_a_client_handle() {
    let generated = bindings();
    let named = &generated["greeter_named.rs"];
    for expected in [
        "pub trait GreeterNamed: Clone + Send + Sync {",
        "fn message(&self) -> impl Future<Output = Result<String, dagger_sdk::QueryError>> + Send;",
        "fn describe_opts(&self, opts: &GreeterNamedDescribeOpts) -> impl Future<Output = Result<String, dagger_sdk::QueryError>> + Send;",
        "pub struct GreeterNamedClient { query: dagger_sdk::QueryBuilder }",
        "impl From<GreeterNamedClient> for dagger_sdk::IdInput<GreeterNamedClient> {",
        "impl super::GreeterNamed for GreeterNamedClient {",
        "fn message(&self) -> impl Future<Output = Result<String, dagger_sdk::QueryError>> + Send {
            let __query = self.query.select(\"message\");
            async move { __query.execute().await }
        }",
        "fn describe_opts(&self, opts: &super::GreeterNamedDescribeOpts) -> impl Future<Output = Result<String, dagger_sdk::QueryError>> + Send {",
        // A trait method's future is `Send`, so what it takes by conversion is too, and
        // the request is built before the future exists.
        "fn relabel(&self, label: impl Into<String> + Send, source: impl Into<dagger_sdk::IdInput<dagger_sdk::Directory>> + Send, tags: Vec<impl Into<String> + Send>)
            -> impl Future<Output = Result<String, dagger_sdk::QueryError>> + Send;",
        "fn relabel(&self, label: impl Into<String> + Send, source: impl Into<dagger_sdk::IdInput<dagger_sdk::Directory>> + Send, tags: Vec<impl Into<String> + Send>)
            -> impl Future<Output = Result<String, dagger_sdk::QueryError>> + Send {
            let __query = self.query.select(\"relabel\");
            let __query = __query.argument(\"label\", Into::<String>::into(label));
            let __query = __query.generated_argument_id_shape(\"source\", Into::<dagger_sdk::IdInput<dagger_sdk::Directory>>::into(source));
            let tags = Iterator::collect::<Vec<String>>(Iterator::map(IntoIterator::into_iter(tags), Into::into));
            let __query = __query.argument(\"tags\", tags);
            async move { __query.execute().await }
        }",
        // An interface result is the interface's client, lazily or re-entered from IDs.
        "#[must_use] fn renamed(&self, name: impl Into<String> + Send) -> super::GreeterNamedClient;",
        "fn renamed(&self, name: impl Into<String> + Send) -> super::GreeterNamedClient {
            let __query = self.query.select(\"renamed\");
            let __query = __query.argument(\"name\", Into::<String>::into(name));
            super::GreeterNamedClient::__from_query(__query)
        }",
        "fn aliases(&self) -> impl Future<Output = Result<Vec<super::GreeterNamedClient>, dagger_sdk::QueryError>> + Send {
            let __query = self.query.select(\"aliases\");
            async move {
                let __ids: Vec<dagger_sdk::Id> = __query.select(\"id\").execute().await?;
                Result::Ok(Iterator::collect(Iterator::map(IntoIterator::into_iter(__ids), |__value_0| super::GreeterNamedClient::__from_query(
                    __query.generated_reentry_builder(__value_0, \"GreeterNamed\")
                ))))
            }
        }",
    ] {
        assert_binds("greeter_named.rs", named, expected);
    }
}

// An enum crosses the wire by its GraphQL names, through serde as the SDK re-exports
// it, since a module's crate has no `serde` of its own.
#[test]
fn an_enum_carries_its_wire_names() {
    let generated = bindings();
    let mode = &generated["greeter_mode.rs"];
    for expected in [
        "#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, dagger_sdk::__private::serde::Deserialize, dagger_sdk::__private::serde::Serialize)]",
        "#[serde(crate = \"dagger_sdk::__private::serde\")]",
        "pub enum GreeterMode {",
        "#[serde(rename = \"LOUD\")] Loud,",
        "#[serde(rename = \"QUIET\")] Quiet,",
    ] {
        assert_binds("greeter_mode.rs", mode, expected);
    }
}

// A module may key its SDK dependency under any name, and every path a binding takes
// into the SDK goes through that name, the serde path a derive is given included.
#[test]
fn typed_bindings_reach_the_sdk_under_the_module_alias() {
    let default = bindings();
    let renamed = generate(&dependency_module_schema(0), "renamed_sdk")
        .expect("a module that renames the SDK must generate");
    for bound in BINDINGS {
        assert!(
            !renamed[*bound].contains("dagger_sdk"),
            "{bound} names the SDK other than by the module's alias"
        );
        assert_eq!(
            renamed[*bound].replace("renamed_sdk", "dagger_sdk"),
            default[*bound]
        );
    }
}

fn adversarial() -> BTreeMap<String, String> {
    generate(
        &adversarial_dependency_module_schema("dagger_sdk"),
        "dagger_sdk",
    )
    .expect("a module must generate beside a dependency's most awkward names")
}

/// A binding's tokens outside its attributes, in order. A derive names its macros
/// bare, and those live in another namespace than the types a dependency adds.
fn code_tokens(source: &str) -> Vec<String> {
    fn flatten(stream: proc_macro2::TokenStream, tokens: &mut Vec<String>) {
        let mut stream = stream.into_iter().peekable();
        while let Some(token) = stream.next() {
            match token {
                proc_macro2::TokenTree::Punct(punctuation) if punctuation.as_char() == '#' => {
                    // `#[..]`, or `#![..]` for the file's own documentation.
                    if matches!(
                        stream.peek(),
                        Some(proc_macro2::TokenTree::Punct(bang)) if bang.as_char() == '!'
                    ) {
                        stream.next();
                    }
                    stream.next();
                }
                proc_macro2::TokenTree::Group(group) => flatten(group.stream(), tokens),
                token => tokens.push(token.to_string()),
            }
        }
    }
    let mut flat = Vec::new();
    flatten(source.parse().expect("a binding tokenizes"), &mut flat);
    flat
}

// A binding is compiled in a file of its own, where the bound type is in scope under
// its own name. A dependency called `result` has the main object `Result`, and a bare
// `Result<..>` in that file would mean the handle. So every standard item is named
// from the crate root, in every binding, and a method of a prelude trait is called by
// its path.
#[test]
fn no_standard_name_is_written_bare() {
    const STANDARD: &[&str] = &[
        "Box",
        "Clone",
        "Copy",
        "Default",
        "Err",
        "From",
        "Future",
        "Into",
        "IntoIterator",
        "Iterator",
        "None",
        "Ok",
        "Option",
        "Pin",
        "Result",
        "Send",
        "Some",
        "String",
        "Sync",
        "ToOwned",
        "ToString",
        "TryFrom",
        "TryInto",
        "Vec",
    ];
    const PRELUDE_CALLS: &[&str] = &[
        "clone",
        "collect",
        "into",
        "into_iter",
        "map",
        "to_owned",
        "to_string",
        "try_into",
    ];
    let generated = adversarial();
    let mut checked = 0;
    for (file, source) in &generated {
        // The support the module compiler writes is not compiled beside a type's name.
        if file == "mod.rs" || file.starts_with("module_") {
            continue;
        }
        checked += 1;
        let tokens = code_tokens(source);
        for (index, token) in tokens.iter().enumerate() {
            let after_path = index >= 2 && tokens[index - 1] == ":" && tokens[index - 2] == ":";
            // A name directly under `super` or the SDK is a binding, not a standard item.
            let binding = index >= 3
                && after_path
                && matches!(tokens[index - 3].as_str(), "super" | "dagger_sdk");
            if STANDARD.contains(&token.as_str()) {
                // `Sync` is the fixture's interface: a trait and a client in `sync.rs`.
                let own = (PRELUDE_NAMED_TYPES.contains(&token.as_str()) || token == "Sync")
                    && file == &format!("{}.rs", snake(token));
                // In the file of a type so named, the name is also the handle's. That
                // file is checked by building it, with the representative module.
                assert!(
                    binding || after_path || own,
                    "{file} writes `{token}` bare at token {index}: {}",
                    tokens[index.saturating_sub(6)..(index + 4).min(tokens.len())].join(" ")
                );
            }
            if index >= 1
                && tokens[index - 1] == "."
                && PRELUDE_CALLS.contains(&token.as_str())
                && tokens.get(index + 1).is_none_or(|next| next != ":")
            {
                // `self.clone` is a field named like the method, never a call.
                let field = tokens.get(index + 1).is_some_and(|next| next == "=");
                assert!(
                    field,
                    "{file} calls the prelude method `{token}` by name at token {index}: {}",
                    tokens[index.saturating_sub(6)..(index + 4).min(tokens.len())].join(" ")
                );
            }
        }
    }
    assert!(checked >= BINDINGS.len() + PRELUDE_NAMED_TYPES.len());
}

fn snake(name: &str) -> String {
    let mut snake = String::new();
    for (index, character) in name.chars().enumerate() {
        if character.is_uppercase() && index > 0 {
            snake.push('_');
        }
        snake.extend(character.to_lowercase());
    }
    snake
}

// A method's own locals have names GraphQL cannot give an argument, and the parameter
// that carries the optional arguments moves aside for a required argument called
// `opts`. Either would otherwise be shadowed, or bound twice.
#[test]
fn an_argument_named_like_a_local_of_the_method_is_bound_beside_it() {
    let generated = adversarial();
    for expected in [
        "pub async fn search(&self, id: impl Into<String>, ids: impl Into<String>, opts: impl Into<String>, query: impl Into<String>, value: impl Into<String>) -> Result<String, dagger_sdk::QueryError> {
            let __query = self.query.select(\"search\");
            let __query = __query.argument(\"id\", Into::<String>::into(id));
            let __query = __query.argument(\"ids\", Into::<String>::into(ids));
            let __query = __query.argument(\"opts\", Into::<String>::into(opts));
            let __query = __query.argument(\"query\", Into::<String>::into(query));
            let __query = __query.argument(\"value\", Into::<String>::into(value));
            __query.execute().await
        }",
        "pub async fn search_opts(&self, id: impl Into<String>, ids: impl Into<String>, opts: impl Into<String>, query: impl Into<String>, value: impl Into<String>, opts_: &GreeterSearchOpts) -> Result<String, dagger_sdk::QueryError> {
            let __query = self.query.select(\"search\");
            let __query = if let Option::Some(__value) = &opts_.builder { __query.argument(\"builder\", __value) } else { __query };
            let __query = __query.argument(\"id\", Into::<String>::into(id));
            let __query = __query.argument(\"ids\", Into::<String>::into(ids));
            let __query = if let Option::Some(__value) = &opts_.limit { __query.argument(\"limit\", __value) } else { __query };
            let __query = __query.argument(\"opts\", Into::<String>::into(opts));
            let __query = __query.argument(\"query\", Into::<String>::into(query));
            let __query = __query.argument(\"value\", Into::<String>::into(value));
            __query.execute().await
        }",
        // A handle is re-entered from the IDs the method's own locals hold.
        "let __ids: Vec<dagger_sdk::Id> = __query.select(\"id\").execute().await?;",
        "let __ids: Option<dagger_sdk::Id> = __query.select(\"id\").execute().await?;",
    ] {
        assert_binds("greeter.rs", &generated["greeter.rs"], expected);
    }
    // The interface's trait and each implementation name the parameter alike.
    for file in ["greeter_named.rs", "greeter_message.rs"] {
        for expected in [
            "value: impl Into<String> + Send, opts_: &super::GreeterNamedSearchOpts",
            "let __query = if let Option::Some(__value) = &opts_.builder { __query.argument(\"builder\", __value) } else { __query };",
            "async move { __query.execute().await }",
        ] {
            assert_binds(file, &generated[file], expected);
        }
    }
    assert_binds(
        "greeter_named.rs",
        &generated["greeter_named.rs"],
        "value: impl Into<String> + Send, opts_: &GreeterNamedSearchOpts",
    );
    // Without such an argument the parameter keeps its plain name.
    assert_binds(
        "greeter.rs",
        &generated["greeter.rs"],
        "pub async fn shout_opts(&self, name: impl Into<String>, opts: &GreeterShoutOpts)",
    );
}

// Every handle can already be cloned and converted through the standard prelude. A
// dependency's `clone` or `into` gets a trailing underscore, so that `handle.clone()`
// still clones the handle; its `_opts` form has no such rival and keeps the name.
#[test]
fn a_function_named_like_a_prelude_method_gets_a_trailing_underscore() {
    let generated = adversarial();
    let methods = inherent_methods(&generated["greeter.rs"], "Greeter");
    for (escaped, plain) in [
        ("clone_", "clone"),
        ("clone_from_", "clone_from"),
        ("clone_into_", "clone_into"),
        ("into_", "into"),
        ("to_owned_", "to_owned"),
        ("try_into_", "try_into"),
    ] {
        assert!(methods.iter().any(|method| method == escaped), "{escaped}");
        assert!(!methods.iter().any(|method| method == plain), "{plain}");
    }
    assert!(methods.iter().any(|method| method == "into_opts"));
    // Names that no prelude trait gives a handle stay as the dependency wrote them.
    for kept in [
        "new",
        "default",
        "drop",
        "into_parts",
        "from_query",
        "selection",
    ] {
        assert!(methods.iter().any(|method| method == kept), "{kept}");
    }
    assert_binds(
        "greeter.rs",
        &generated["greeter.rs"],
        "#[must_use] pub fn clone_(&self) -> super::Greeter {
            let __query = self.query.select(\"clone\");
            super::Greeter::__from_query(__query)
        }",
    );
    // The interface's trait is escaped as its client is, or a handle that implements
    // it would have two methods called `clone` once the trait is in scope.
    for expected in [
        "#[must_use] fn clone_(&self) -> super::GreeterNamedClient;",
        "#[must_use] pub fn clone_(&self) -> super::GreeterNamedClient {",
    ] {
        assert_binds("greeter_named.rs", &generated["greeter_named.rs"], expected);
    }
    // Clippy reads conventions into `new`, `default`, `into_*` and `from_*`, and counts
    // arguments. The dependency's author chose them, so the generated module allows it.
    for lint in [
        "clippy::new_ret_no_self",
        "clippy::should_implement_trait",
        "clippy::too_many_arguments",
        "clippy::wrong_self_convention",
    ] {
        assert!(generated["mod.rs"].contains(lint), "{lint}");
    }
}

// The escape cannot meet a name the dependency itself uses: its own `clone_` projects
// to the Rust name `clone` as well, and the projection's name map refuses the pair.
#[test]
fn a_function_that_would_meet_an_escaped_name_is_refused() {
    let schema = changed_dependency_module_schema(|document| {
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
    let refused =
        generate(&schema, "dagger_sdk").expect_err("two methods of one handle cannot share a name");
    let diagnostic = refused
        .diagnostics()
        .iter()
        .find(|diagnostic| diagnostic.code == DiagnosticCode::RustNameCollision)
        .unwrap_or_else(|| panic!("{refused:?}"));
    assert!(
        diagnostic
            .coordinate
            .as_ref()
            .is_some_and(|coordinate| coordinate.as_str().starts_with("Probe.clone")),
        "{diagnostic:?}"
    );
}

// A dependency's documentation is prose for the reader. Emitted as plain `#[doc]`, an
// indented block or an untagged fence in it would be compiled by the module's
// `cargo test` as a Rust doctest, so it is left out when doctests are collected.
#[test]
fn dependency_documentation_is_not_collected_as_doctests() {
    let generated = adversarial();
    for (file, source) in &generated {
        if file == "mod.rs" || file.starts_with("module_") {
            continue;
        }
        let flat = tokens(source);
        assert!(
            !flat.contains(" # doc = "),
            "{file} emits documentation that rustdoc would test"
        );
    }
    // The text itself is kept whole, for the reader of the module's documentation.
    let greeter = &generated["greeter.rs"];
    assert!(tokens(greeter).contains(" # cfg_attr not doctest doc = "));
    for kept in [
        "    dagger call documented --name=world",
        "dag.Greeter().Documented(ctx)",
        "    --name=world",
    ] {
        assert!(greeter.contains(kept), "{kept}");
    }
}

// A dependency type is bound in a Rust module named after it, declared in `mod.rs`,
// where a type called what the module calls the SDK would otherwise take the SDK's
// place in every path that begins with that name.
#[test]
fn the_generated_module_names_the_sdk_from_the_crate_root() {
    let generated = adversarial();
    assert!(generated.contains_key("dagger_sdk.rs"));
    let index = &generated["mod.rs"];
    assert!(index.contains("mod dagger_sdk;"));
    assert!(index.contains("pub use ::dagger_sdk::__private::*;"));
    assert!(index.contains(
        "pub use ::dagger_sdk::{CurrentCall, ModuleCancellation, ModuleError, ModuleErrorDetail, TelemetryContext};"
    ));
    // The one path left that begins with the bare name means the type's own module.
    assert_eq!(
        index
            .replace("::dagger_sdk::", "")
            .matches("dagger_sdk::")
            .collect::<Vec<_>>(),
        ["dagger_sdk::"]
    );
    assert!(index.contains("pub use dagger_sdk::*;"));
}

// A type whose binding would take the name or the file of the module's own generated
// support is refused where the dependency states it.
#[test]
fn a_dependency_type_named_like_generated_support_is_refused() {
    for (type_name, code, stated) in [
        (
            "ModuleContext",
            DiagnosticCode::RustNameCollision,
            "would be bound as `ModuleContext`",
        ),
        (
            "ModuleQuery",
            DiagnosticCode::RustNameCollision,
            "would be bound as `ModuleQuery`",
        ),
        (
            "ModuleCodec",
            DiagnosticCode::OperationArtifactCollision,
            "would be bound in `module_codec.rs`",
        ),
    ] {
        let schema = changed_dependency_module_schema(|document| {
            add_constructed_object(document, Some("pack"), type_name, "pack", vec![]);
        });
        let refused = generate(&schema, "dagger_sdk")
            .expect_err("a binding that shadows generated support must be refused");
        let diagnostic = refused
            .diagnostics()
            .iter()
            .find(|diagnostic| diagnostic.code == code)
            .unwrap_or_else(|| panic!("{type_name}: {refused:?}"));
        assert_eq!(
            diagnostic
                .coordinate
                .as_ref()
                .map(|coordinate| coordinate.as_str()),
            Some(type_name)
        );
        assert!(diagnostic.message.contains(stated), "{diagnostic:?}");
        assert!(
            diagnostic.message.contains("of module dependency `pack`"),
            "{diagnostic:?}"
        );
        assert!(
            diagnostic
                .message
                .contains("declare the dependency under another name"),
            "{diagnostic:?}"
        );
    }
}

// A dependency function that names a type with no typed binding cannot be called
// typed, and an untyped method would return a value no other binding accepts.
#[test]
fn a_dependency_function_that_names_an_unowned_type_is_refused_at_its_coordinate() {
    let schema = changed_dependency_module_schema(|document| {
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
    let refused =
        generate(&schema, "dagger_sdk").expect_err("a typed method cannot name an untyped handle");
    let diagnostic = refused
        .diagnostics()
        .iter()
        .find(|diagnostic| diagnostic.code == DiagnosticCode::SchemaFieldUnmapped)
        .unwrap_or_else(|| panic!("{refused:?}"));
    assert_eq!(
        diagnostic
            .coordinate
            .as_ref()
            .map(|coordinate| coordinate.as_str()),
        Some("Probe.thing")
    );
    assert!(diagnostic.message.contains("`LaterThing`"));
}

// The checked files are what a module author compiles against, kept for review.
#[test]
fn checked_dependency_bindings_match_the_renderer() {
    let generated = bindings();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/dependency_bindings");
    let update = std::env::var_os("DAGGER_UPDATE_DEPENDENCY_BINDINGS_FIXTURE").is_some();
    for bound in BINDINGS {
        let fixture = root.join(bound);
        if update {
            fs::create_dir_all(&root).expect("fixture directory is writable");
            fs::write(&fixture, &generated[*bound]).expect("fixture is writable");
        }
        let checked = fs::read_to_string(&fixture)
            .unwrap_or_else(|error| panic!("{} must exist: {error}", fixture.display()));
        assert_eq!(
            checked.lines().take(2).collect::<Vec<_>>(),
            generated[*bound].lines().take(2).collect::<Vec<_>>(),
            "{bound} states another schema or target than the renderer's"
        );
        assert_eq!(
            tokens(&checked),
            tokens(&generated[*bound]),
            "{bound} drifted from the renderer"
        );
    }
    let checked = fs::read_dir(&root)
        .expect("fixture directory is readable")
        .map(|entry| {
            entry
                .expect("fixture entry is readable")
                .file_name()
                .to_string_lossy()
                .into_owned()
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(
        checked,
        BINDINGS.iter().map(|bound| (*bound).to_owned()).collect()
    );
}
