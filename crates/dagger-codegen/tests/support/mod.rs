//! Shared valid-first generators and deterministic recording doubles for codegen tests.

#![allow(
    dead_code,
    reason = "every integration-test binary compiles this shared fixture module and consumes its own subset"
)]

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use dagger_codegen::engine::ModuleAuthoringInput;
use dagger_codegen::module::{
    CfgEnvironment, FormatVersion as ModuleFormatVersion, ModulePackage, ModuleSourcePath,
    ModuleSourceSnapshot, PackageName, Sha256Digest as ModuleDigest, SourceDocument, TargetValue,
    source_snapshot_digest,
};
use dagger_codegen::projection::catalog::{CatalogDisposition, CatalogEntry};
use dagger_codegen::schema::raw::{
    DirectiveApplication, DirectiveApplicationArgument, FullType, TypeKind, TypeRef,
};
use dagger_codegen::target::CodegenTarget;
use proptest::prelude::*;
use proptest::test_runner::{Config, FileFailurePersistence};
use serde_json::{Value, json};

pub(crate) const PURE_CASES: u32 = 256;
pub(crate) const FILESYSTEM_CASES: u32 = 128;
pub(crate) const TARGET_BYTES: &[u8] = include_bytes!("../../../../codegen/target.json");
pub(crate) const CORE_SCHEMA_BYTES: &[u8] = include_bytes!("../../../../codegen/schema.json");

pub(crate) fn module_authoring_input() -> ModuleAuthoringInput {
    module_authoring_input_from(MODULE_AUTHORING_SOURCE)
}

pub(crate) const MODULE_AUTHORING_SOURCE: &str = r#"
#[dagger_sdk::object(root)]
pub struct FixtureRoot {
    #[dagger(field)]
    value: String,
}

#[dagger_sdk::methods]
impl FixtureRoot {
    #[dagger(constructor)]
    pub fn new() -> FixtureRoot { FixtureRoot { value: "fixture".to_owned() } }

    #[dagger(function, cache = "never", role = "check")]
    pub fn value(&self) -> String { self.value.clone() }

    #[dagger(function)]
    pub fn directory(
        &self,
        #[dagger(default_path = ".", ignore = ["target"])] source: dagger_sdk::Directory,
    ) -> dagger_sdk::Directory { source }
}
"#;

pub(crate) fn module_authoring_input_from(source: &str) -> ModuleAuthoringInput {
    let path = ModuleSourcePath::new("src/lib.rs").expect("fixture source path must validate");
    let mut snapshot = ModuleSourceSnapshot {
        format_version: ModuleFormatVersion::current(),
        package: ModulePackage {
            name: PackageName::new("fixture").expect("fixture package must validate"),
            crate_root: path.clone(),
            edition: TargetValue::new("2024").expect("fixture edition must validate"),
        },
        cfg: CfgEnvironment {
            values: BTreeMap::new(),
            features: BTreeSet::new(),
        },
        documents: BTreeMap::from([(path.clone(), SourceDocument::new(path, source))]),
        digest: ModuleDigest::hash_bytes(b"pending fixture snapshot"),
    };
    snapshot.digest = source_snapshot_digest(&snapshot).expect("fixture snapshot must hash");
    ModuleAuthoringInput {
        source: snapshot,
        generator_digest: ModuleDigest::hash_bytes(b"fixture module generator"),
        sdk_dependency_alias: "dagger_sdk".to_owned(),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum VisibleSchemaCase {
    ExactCore,
    CompatibleExtension,
    EngineModuleExtension,
    CoreMutation,
    CoreOmission,
    UnresolvedReference,
    RustNameCollision,
}

/// The checked Core schema with the types of one more module, `minimal`: objects, an
/// interface, an enum, a scalar and an input object, reached through `Query.minimal`.
pub(crate) fn minimal_module_schema() -> Vec<u8> {
    let mut document: Value =
        serde_json::from_slice(CORE_SCHEMA_BYTES).expect("checked schema fixture must decode");
    add_minimal_module_types(&mut document);
    serde_json::to_vec(&document).expect("schema fixture must encode")
}

fn add_minimal_module_types(document: &mut Value) {
    add_scalar_type(document, "MinimalToken");
    add_enum_type(document, "MinimalState", &["READY", "BUSY"]);
    add_input_type(
        document,
        "MinimalConfig",
        vec![input_field(
            "enabled",
            scalar_type("Boolean", true),
            Some("true"),
        )],
    );
    add_object_type(
        document,
        "MinimalItem",
        vec![
            object_field("id", scalar_type("ID", false), vec![]),
            object_field("state", enum_type("MinimalState", false), vec![]),
        ],
    );
    add_object_type(
        document,
        "MinimalClient",
        vec![object_field("id", scalar_type("ID", false), vec![])],
    );
    add_interface_to_object(document, "MinimalItem", "MinimalNode");
    add_interface_type(document, "MinimalNode", "MinimalItem");
    add_object_type(
        document,
        "Minimal",
        vec![
            object_field("id", scalar_type("ID", false), vec![]),
            expected_type_field("sync", "Minimal"),
            object_field("message", scalar_type("String", false), vec![]),
            object_field("type", scalar_type("String", false), vec![]),
            object_field("token", scalar_type("MinimalToken", false), vec![]),
            object_field("helper", object_type("MinimalClient", false), vec![]),
            object_field("container", object_type("Container", false), vec![]),
            object_field("maybeContainer", object_type("Container", true), vec![]),
            object_field("node", interface_type("MinimalNode", false), vec![]),
            object_field(
                "item",
                object_type("MinimalItem", true),
                vec![argument(
                    "config",
                    input_object_type("MinimalConfig", true),
                    None,
                )],
            ),
            object_field(
                "items",
                list_type(object_type("MinimalItem", true), false),
                vec![],
            ),
            object_field(
                "useItem",
                scalar_type("String", false),
                vec![typed_id_argument("item", "MinimalItem", false)],
            ),
            object_field(
                "useItems",
                scalar_type("String", false),
                vec![typed_id_list_argument("items", "MinimalItem")],
            ),
            object_field(
                "search",
                scalar_type("String", false),
                vec![
                    argument("enabled", scalar_type("Boolean", true), Some("true")),
                    argument("count", scalar_type("Int", true), None),
                    argument("label", scalar_type("String", true), None),
                    typed_id_argument("item", "MinimalItem", true),
                ],
            ),
        ],
    );
    add_query_object_field(document, "minimal", "Minimal");
}

fn add_interface_to_object(document: &mut Value, object: &str, interface: &str) {
    let object = schema_array_mut(document, "types")
        .iter_mut()
        .find(|definition| definition["name"] == object)
        .expect("client fixture object must exist");
    object["interfaces"]
        .as_array_mut()
        .expect("object interfaces must be an array")
        .push(json!({"kind": "INTERFACE", "name": interface}));
}

fn add_interface_type(document: &mut Value, name: &str, possible_type: &str) {
    schema_array_mut(document, "types").push(json!({
        "kind": "INTERFACE",
        "name": name,
        "description": format!("Client fixture interface {name}."),
        "fields": [
            object_field("id", scalar_type("ID", false), vec![]),
            object_field("message", scalar_type("String", false), vec![])
        ],
        "inputFields": [],
        "interfaces": [],
        "enumValues": [],
        "possibleTypes": [{"kind": "OBJECT", "name": possible_type}],
        "directives": []
    }));
}

fn add_scalar_type(document: &mut Value, name: &str) {
    schema_array_mut(document, "types").push(json!({
        "kind": "SCALAR",
        "name": name,
        "description": format!("Client fixture scalar {name}."),
        "fields": [],
        "inputFields": [],
        "interfaces": [],
        "enumValues": [],
        "possibleTypes": [],
        "directives": []
    }));
}

fn add_query_object_field(document: &mut Value, field: &str, object: &str) {
    let query = schema_array_mut(document, "types")
        .iter_mut()
        .find(|definition| definition["name"] == "Query")
        .expect("checked schema must contain Query");
    query["fields"]
        .as_array_mut()
        .expect("Query fields must be an array")
        .push(object_field(field, object_type(object, false), vec![]));
}

fn add_object_field_to_type(document: &mut Value, owner: &str, field: Value) {
    let object = schema_array_mut(document, "types")
        .iter_mut()
        .find(|definition| definition["name"] == owner)
        .expect("client fixture object must exist");
    object["fields"]
        .as_array_mut()
        .expect("object fields must be an array")
        .push(field);
}

fn add_object_type(document: &mut Value, name: &str, fields: Vec<Value>) {
    schema_array_mut(document, "types").push(json!({
        "kind": "OBJECT",
        "name": name,
        "description": format!("Client fixture object {name}."),
        "fields": fields,
        "inputFields": [],
        "interfaces": [],
        "enumValues": [],
        "possibleTypes": [],
        "directives": []
    }));
}

fn add_enum_type(document: &mut Value, name: &str, values: &[&str]) {
    schema_array_mut(document, "types").push(json!({
        "kind": "ENUM",
        "name": name,
        "description": format!("Client fixture enum {name}."),
        "fields": [],
        "inputFields": [],
        "interfaces": [],
        "enumValues": values.iter().map(|value| json!({
            "name": value,
            "description": format!("Client fixture value {value}."),
            "isDeprecated": false,
            "deprecationReason": null,
            "directives": []
        })).collect::<Vec<_>>(),
        "possibleTypes": [],
        "directives": []
    }));
}

fn add_input_type(document: &mut Value, name: &str, fields: Vec<Value>) {
    schema_array_mut(document, "types").push(json!({
        "kind": "INPUT_OBJECT",
        "name": name,
        "description": format!("Client fixture input {name}."),
        "fields": [],
        "inputFields": fields,
        "interfaces": [],
        "enumValues": [],
        "possibleTypes": [],
        "directives": []
    }));
}

fn object_field(name: &str, type_ref: Value, args: Vec<Value>) -> Value {
    json!({
        "name": name,
        "description": format!("Client fixture field {name}."),
        "type": type_ref,
        "args": args,
        "isDeprecated": false,
        "deprecationReason": null,
        "directives": []
    })
}

fn expected_type_field(name: &str, target: &str) -> Value {
    let mut field = object_field(name, scalar_type("ID", false), vec![]);
    field["directives"] = json!([{
        "name": "expectedType",
        "args": [{"name": "name", "value": format!("\"{target}\"")}]
    }]);
    field
}

fn argument(name: &str, type_ref: Value, default: Option<&str>) -> Value {
    json!({
        "name": name,
        "description": format!("Client fixture argument {name}."),
        "type": type_ref,
        "defaultValue": default,
        "isDeprecated": false,
        "deprecationReason": null,
        "directives": []
    })
}

fn typed_id_argument(name: &str, target: &str, nullable: bool) -> Value {
    let mut argument = argument(name, scalar_type("ID", nullable), None);
    argument["directives"] = json!([{
        "name": "expectedType",
        "args": [{"name": "name", "value": format!("\"{target}\"")}]
    }]);
    argument
}

fn typed_id_list_argument(name: &str, target: &str) -> Value {
    let mut argument = argument(name, list_type(scalar_type("ID", true), false), None);
    argument["directives"] = json!([{
        "name": "expectedType",
        "args": [{"name": "name", "value": format!("\"{target}\"")}]
    }]);
    argument
}

fn input_field(name: &str, type_ref: Value, default: Option<&str>) -> Value {
    argument(name, type_ref, default)
}

fn scalar_type(name: &str, nullable: bool) -> Value {
    named_type("SCALAR", name, nullable)
}

fn enum_type(name: &str, nullable: bool) -> Value {
    named_type("ENUM", name, nullable)
}

fn object_type(name: &str, nullable: bool) -> Value {
    named_type("OBJECT", name, nullable)
}

fn interface_type(name: &str, nullable: bool) -> Value {
    named_type("INTERFACE", name, nullable)
}

fn input_object_type(name: &str, nullable: bool) -> Value {
    named_type("INPUT_OBJECT", name, nullable)
}

fn named_type(kind: &str, name: &str, nullable: bool) -> Value {
    let leaf = json!({"kind": kind, "name": name});
    if nullable {
        leaf
    } else {
        json!({"kind": "NON_NULL", "ofType": leaf})
    }
}

fn list_type(element: Value, nullable: bool) -> Value {
    let list = json!({"kind": "LIST", "ofType": element});
    if nullable {
        list
    } else {
        json!({"kind": "NON_NULL", "ofType": list})
    }
}

pub(crate) fn visible_schema(case: VisibleSchemaCase, permutation: u16) -> Vec<u8> {
    let mut document: Value =
        serde_json::from_slice(CORE_SCHEMA_BYTES).expect("checked schema fixture must decode");
    match case {
        VisibleSchemaCase::ExactCore => {}
        VisibleSchemaCase::CompatibleExtension => {
            add_extension(&mut document, "RustMode", true, None)
        }
        VisibleSchemaCase::EngineModuleExtension => {
            add_extension(&mut document, "RustMode", true, Some("rust-probe"))
        }
        VisibleSchemaCase::CoreMutation => make_container_platform_nullable(&mut document),
        VisibleSchemaCase::CoreOmission => {
            let query = schema_array_mut(&mut document, "types")
                .iter_mut()
                .find(|definition| definition["name"] == "Query")
                .expect("checked schema must contain Query");
            query["fields"]
                .as_array_mut()
                .expect("Query fields must be an array")
                .retain(|field| field["name"] != "address");
        }
        VisibleSchemaCase::UnresolvedReference => {
            add_query_field(&mut document, "rustMode", "MissingRustMode", None)
        }
        VisibleSchemaCase::RustNameCollision => {
            add_extension(&mut document, "RustMode", true, None);
            add_extension(&mut document, "Rust_Mode", false, None);
        }
    }
    permute_schema(&mut document, permutation);
    serde_json::to_vec(&document).expect("schema fixture must encode")
}

pub(crate) fn module_visible_schema(permutation: u16) -> Vec<u8> {
    let mut document: Value =
        serde_json::from_slice(CORE_SCHEMA_BYTES).expect("checked schema fixture must decode");
    for hidden in [
        "Host",
        "HostID",
        "Engine",
        "EngineID",
        "EngineCache",
        "EngineCacheID",
        "EngineCacheEntry",
        "EngineCacheEntryID",
        "EngineCacheEntrySet",
        "EngineCacheEntrySetID",
    ] {
        scrub_type(&mut document, hidden);
    }
    for hidden in [
        "Query.currentWorkspace",
        "Query.engineVolume",
        "Query.sshfsVolume",
        "Query.setSessionTitle",
        "Address.volume",
    ] {
        scrub_field(&mut document, hidden);
    }
    permute_schema(&mut document, permutation);
    serde_json::to_vec(&document).expect("module-visible schema fixture must encode")
}

/// Rewords reviewed core documentation the way a later engine's copy of the same API
/// view does, leaving every wire contract alone.
pub(crate) fn reword_core_documentation(schema: &[u8]) -> Vec<u8> {
    let mut document: Value = serde_json::from_slice(schema).expect("schema fixture must decode");
    schema_array_mut(&mut document, "types")
        .iter_mut()
        .find(|definition| definition["name"] == "Container")
        .expect("checked schema must contain Container")["description"] =
        json!("A container, as a later engine documents it.");
    core_field_mut(&mut document, "Container", "platform")["description"] =
        json!("The platform, as a later engine documents it.");
    serde_json::to_vec(&document).expect("reworded schema fixture must encode")
}

/// Adds a field a later engine serves on a reviewed core object, naming only reviewed
/// core types, which no binding compiled for the target consumes.
pub(crate) fn add_later_core_field(schema: &[u8]) -> Vec<u8> {
    let mut document: Value = serde_json::from_slice(schema).expect("schema fixture must decode");
    add_object_field_to_type(
        &mut document,
        "Container",
        object_field(
            "withLaterOption",
            object_type("Container", false),
            vec![argument("enabled", scalar_type("Boolean", true), None)],
        ),
    );
    serde_json::to_vec(&document).expect("extended schema fixture must encode")
}

/// Adds an operation-scoped enum and the query field that returns it, as a module
/// dependency contributes them.
pub(crate) fn add_dependency_extension(schema: &[u8]) -> Vec<u8> {
    let mut document: Value = serde_json::from_slice(schema).expect("schema fixture must decode");
    add_extension(&mut document, "RustMode", true, None);
    serde_json::to_vec(&document).expect("extended schema fixture must encode")
}

/// The schema `dagger generate` is served for a module that depends on three modules,
/// as the engine at the target serves it: every type a dependency owns carries
/// `@sourceMap(module:)` and implements Core's `Node`, and each dependency's
/// constructor is a field of `Query`.
///
/// - `greeter` has an object with a function of every shape a caller meets, a second
///   object, an interface that object implements, and an enum. Its constructor takes
///   optional arguments only.
/// - `vault` has a constructor with a required and an optional argument.
/// - `clock` has a constructor without arguments.
pub(crate) fn dependency_module_schema(permutation: u16) -> Vec<u8> {
    let mut document: Value = serde_json::from_slice(&module_visible_schema(0))
        .expect("module-visible schema fixture must decode");
    add_module_dependencies(&mut document);
    permute_schema(&mut document, permutation);
    serde_json::to_vec(&document).expect("dependency schema fixture must encode")
}

/// The same three dependencies beside the whole reviewed Core API, as a session that is
/// not a module's is served them.
pub(crate) fn dependency_core_schema(permutation: u16) -> Vec<u8> {
    let mut document: Value =
        serde_json::from_slice(CORE_SCHEMA_BYTES).expect("checked schema fixture must decode");
    add_module_dependencies(&mut document);
    permute_schema(&mut document, permutation);
    serde_json::to_vec(&document).expect("dependency schema fixture must encode")
}

/// The names of the standard prelude that a dependency's type can take, each as the main
/// object of a dependency called by the same name. `Sync` is an interface, which the
/// object `Send` implements. (`String` cannot occur: it is GraphQL's own scalar.)
pub(crate) const PRELUDE_NAMED_TYPES: &[&str] = &[
    "Box",
    "Clone",
    "Default",
    "From",
    "Future",
    "Into",
    "IntoIterator",
    "Iterator",
    "Ok",
    "Option",
    "Pin",
    "Result",
    "Send",
    "Some",
    "Vec",
];

/// The dependency schema with everything in a dependency's own naming that generated
/// Rust could trip over, each of which a module must still build beside:
///
/// - arguments named like the locals a binding's method has (`query`, `opts`, `value`,
///   `ids`, `id`), on an object, on an interface and on a constructor;
/// - functions named like methods every handle has already (`clone`, `into`,
///   `toOwned`, …) or that Clippy reads a convention into (`new`, `default`,
///   `intoParts`, `fromQuery`);
/// - documentation with an indented block and an untagged fence, which rustdoc would
///   compile as Rust;
/// - types named like items of the standard prelude, each with results and arguments
///   of every wrapper, so its binding writes every standard name it uses;
/// - a type whose Rust module is called what the module calls the SDK (`sdk_alias`).
pub(crate) fn adversarial_dependency_module_schema(sdk_alias: &str) -> Vec<u8> {
    adversarial_schema(&dependency_module_schema(0), sdk_alias)
}

/// The same awkward names beside the whole reviewed Core API, as the schema of a
/// module's client is served them.
pub(crate) fn adversarial_dependency_core_schema(sdk_alias: &str) -> Vec<u8> {
    adversarial_schema(&dependency_core_schema(0), sdk_alias)
}

fn adversarial_schema(base: &[u8], sdk_alias: &str) -> Vec<u8> {
    changed_schema(base, |document| {
        let clashing = || {
            vec![
                argument("query", scalar_type("String", false), None),
                argument("opts", scalar_type("String", false), None),
                argument("value", scalar_type("String", false), None),
                argument("ids", scalar_type("String", false), None),
                argument("id", scalar_type("String", false), None),
                argument("limit", scalar_type("Int", true), Some("1")),
                argument("builder", scalar_type("String", true), None),
            ]
        };
        // On the interface and its implementor: a value, a lazy object and a list.
        for owner in ["GreeterNamed", "GreeterMessage"] {
            add_object_field_to_type(
                document,
                owner,
                object_field("search", scalar_type("String", false), clashing()),
            );
            add_object_field_to_type(
                document,
                owner,
                object_field("clone", interface_type("GreeterNamed", false), vec![]),
            );
        }
        for (name, result) in [
            ("search", scalar_type("String", false)),
            ("searchLazy", object_type("Greeter", false)),
            (
                "searchMany",
                list_type(object_type("GreeterMessage", false), false),
            ),
            ("searchMaybe", object_type("GreeterMessage", true)),
        ] {
            add_object_field_to_type(document, "Greeter", object_field(name, result, clashing()));
        }
        for (name, result, args) in [
            ("clone", object_type("Greeter", false), vec![]),
            (
                "into",
                scalar_type("String", false),
                vec![argument(
                    "deep",
                    scalar_type("Boolean", true),
                    Some("false"),
                )],
            ),
            ("toOwned", scalar_type("String", false), vec![]),
            ("tryInto", scalar_type("String", false), vec![]),
            (
                "cloneFrom",
                scalar_type("String", false),
                vec![argument("other", scalar_type("String", false), None)],
            ),
            ("cloneInto", scalar_type("String", false), vec![]),
            ("new", scalar_type("String", false), vec![]),
            ("default", scalar_type("String", false), vec![]),
            ("drop", scalar_type("String", false), vec![]),
            ("intoParts", scalar_type("String", false), vec![]),
            ("fromQuery", scalar_type("String", false), vec![]),
            ("selection", scalar_type("String", false), vec![]),
            (
                "wide",
                scalar_type("String", false),
                (0..9)
                    .map(|index| argument(&format!("a{index}"), scalar_type("Int", false), None))
                    .collect(),
            ),
        ] {
            add_object_field_to_type(document, "Greeter", object_field(name, result, args));
        }
        let mut documented = object_field("documented", scalar_type("String", false), vec![]);
        documented["description"] = json!(
            "Says hello.\n\nExample:\n\n    dagger call documented --name=world\n\nOr:\n\n```\ndag.Greeter().Documented(ctx)\n```\n"
        );
        documented["args"] = json!([{
            "name": "name",
            "description": "The name.\n\n    --name=world",
            "type": {"kind": "SCALAR", "name": "String"},
            "defaultValue": null,
            "isDeprecated": false,
            "deprecationReason": null,
            "directives": []
        }]);
        add_object_field_to_type(document, "Greeter", documented);

        add_interface_type(document, "Sync", "Send");
        own(document, "Sync", "send");
        for type_name in PRELUDE_NAMED_TYPES
            .iter()
            .copied()
            .chain([sdk_alias_type(sdk_alias).as_str()])
        {
            let dependency = lower_first(type_name);
            add_object_type(
                document,
                type_name,
                vec![
                    object_field("id", scalar_type("ID", false), vec![]),
                    object_field("message", scalar_type("String", false), vec![]),
                    object_field("again", object_type(type_name, false), vec![]),
                    object_field("maybe", object_type(type_name, true), vec![]),
                    object_field(
                        "many",
                        list_type(object_type(type_name, false), false),
                        vec![argument(
                            "names",
                            list_type(scalar_type("String", false), false),
                            None,
                        )],
                    ),
                    object_field(
                        "label",
                        scalar_type("String", false),
                        vec![
                            argument("name", scalar_type("String", false), None),
                            typed_id_argument("other", type_name, false),
                            typed_id_list_argument("others", type_name),
                            typed_id_argument("also", type_name, true),
                            argument("tags", list_type(scalar_type("String", false), true), None),
                            argument("loud", scalar_type("Boolean", true), Some("false")),
                        ],
                    ),
                    expected_type_field("sync", type_name),
                ],
            );
            own(document, type_name, &dependency);
            implement_node(document, type_name);
            let mut constructor = object_field(
                &dependency,
                object_type(type_name, false),
                vec![
                    argument("query", scalar_type("String", false), None),
                    argument("opts", scalar_type("String", false), None),
                    argument("loud", scalar_type("Boolean", true), Some("false")),
                ],
            );
            constructor["directives"] = source_map_directives(&dependency);
            add_query_field_value(document, constructor);
        }
        add_interface_to_object(document, "Send", "Sync");
    })
}

/// The object type whose Rust module is called `sdk_alias`.
fn sdk_alias_type(sdk_alias: &str) -> String {
    sdk_alias
        .split('_')
        .map(|word| {
            let mut characters = word.chars();
            characters.next().map_or_else(String::new, |first| {
                first.to_uppercase().chain(characters).collect()
            })
        })
        .collect()
}

fn lower_first(name: &str) -> String {
    let mut characters = name.chars();
    characters.next().map_or_else(String::new, |first| {
        first.to_lowercase().chain(characters).collect()
    })
}

/// Applies `change` to the three dependencies beside the whole reviewed Core API, for
/// a test of what a session that is not a module's is served.
pub(crate) fn changed_dependency_core_schema(change: impl FnOnce(&mut Value)) -> Vec<u8> {
    changed_schema(&dependency_core_schema(0), change)
}

fn changed_schema(base: &[u8], change: impl FnOnce(&mut Value)) -> Vec<u8> {
    let mut document: Value =
        serde_json::from_slice(base).expect("dependency schema fixture must decode");
    change(&mut document);
    serde_json::to_vec(&document).expect("changed dependency schema fixture must encode")
}

/// What the `greeter` module adds to the schema of its client, as an engine at the
/// target serves it. See [`module_client_schema`].
pub(crate) const GREETER_CLIENT_SCHEMA_PART: &[u8] =
    include_bytes!("../fixtures/module_clients/greeter.json");

/// What the `naming-app` module adds to the schema of its client. The module has a type
/// with an acronym in its name, which the engine serves as `NamingAppHTTPFetcher`.
pub(crate) const NAMING_APP_CLIENT_SCHEMA_PART: &[u8] =
    include_bytes!("../fixtures/module_clients/naming-app.json");

/// The schema an engine at the target serves for a module's client, from the part the
/// module adds: `clientSchemaIntrospectionJson(clientVersion:)` of the module's source.
///
/// A captured client schema is the reviewed Core API with three additions, and the
/// fixtures keep only those, since the Core part is the checked `codegen/schema.json`
/// type for type: the module's types, the fields it adds to `Query`, and the interface
/// edges between its types and Core's. The edges are stated by the module's types
/// themselves, so this splices them back: a Core interface gains the module's types
/// that implement it, and a Core type gains the module's interfaces that list it.
/// [`module_client_schema_part`] is the inverse, and checks the round trip.
pub(crate) fn module_client_schema(part: &[u8]) -> Vec<u8> {
    let part: Value = serde_json::from_slice(part).expect("client schema part must decode");
    let mut document: Value =
        serde_json::from_slice(CORE_SCHEMA_BYTES).expect("checked schema fixture must decode");
    let added = part["types"]
        .as_array()
        .expect("client schema part lists the module's types")
        .clone();
    let reference = |definition: &Value| {
        json!({
            "kind": definition["kind"],
            "name": definition["name"],
            "interfaces": null,
            "possibleTypes": null,
            "directives": null
        })
    };
    let types = schema_array_mut(&mut document, "types");
    for definition in &added {
        for (edges, back) in [
            ("interfaces", "possibleTypes"),
            ("possibleTypes", "interfaces"),
        ] {
            // An interface's possible types are objects: an interface that extends
            // another is not one of them.
            if back == "possibleTypes" && definition["kind"] != "OBJECT" {
                continue;
            }
            for edge in definition[edges].as_array().into_iter().flatten() {
                let Some(core) = types.iter_mut().find(|core| core["name"] == edge["name"]) else {
                    continue;
                };
                let list = core[back]
                    .as_array_mut()
                    .expect("a Core type lists its interface edges");
                list.push(reference(definition));
                list.sort_by_key(|entry| entry["name"].as_str().map(str::to_owned));
            }
        }
    }
    types.extend(added);
    types.sort_by_key(|definition| definition["name"].as_str().map(str::to_owned));
    let query = types
        .iter_mut()
        .find(|definition| definition["name"] == "Query")
        .expect("checked schema must contain Query");
    query["fields"]
        .as_array_mut()
        .expect("Query fields must be an array")
        .extend(
            part["queryFields"]
                .as_array()
                .expect("client schema part lists the module's Query fields")
                .clone(),
        );
    serde_json::to_vec(&document).expect("client schema fixture must encode")
}

/// Reduces a client schema captured from an engine to the part its module adds, in the
/// form the fixtures keep.
///
/// Panics unless [`module_client_schema`] gives the captured schema back from that
/// part, type for type, once the introspection types an engine serves beside the
/// schema are set aside. So a fixture stands for exactly what the engine served.
pub(crate) fn module_client_schema_part(captured: &[u8]) -> Vec<u8> {
    let core: Value =
        serde_json::from_slice(CORE_SCHEMA_BYTES).expect("checked schema fixture must decode");
    let mut captured: Value =
        serde_json::from_slice(captured).expect("captured client schema must decode");
    let core_types = core["__schema"]["types"]
        .as_array()
        .expect("checked schema lists its types");
    let is_core = |name: &Value| core_types.iter().any(|core| &core["name"] == name);
    let owned = |definition: &Value| {
        definition["directives"]
            .as_array()
            .is_some_and(|directives| {
                directives
                    .iter()
                    .any(|directive| directive["name"] == "sourceMap")
            })
    };
    // `__Type` and its like describe introspection itself, and the checked schema
    // leaves them out.
    schema_array_mut(&mut captured, "types")
        .retain(|definition| is_core(&definition["name"]) || owned(definition));
    let types = captured["__schema"]["types"]
        .as_array()
        .expect("captured schema lists its types");
    let added = types
        .iter()
        .filter(|definition| !is_core(&definition["name"]))
        .cloned()
        .collect::<Vec<_>>();
    let core_query = core_types
        .iter()
        .find(|definition| definition["name"] == "Query")
        .expect("checked schema must contain Query");
    let query_fields = types
        .iter()
        .find(|definition| definition["name"] == "Query")
        .and_then(|query| query["fields"].as_array())
        .expect("captured schema must contain Query")
        .iter()
        .filter(|field| {
            !core_query["fields"]
                .as_array()
                .is_some_and(|fields| fields.iter().any(|core| core["name"] == field["name"]))
        })
        .cloned()
        .collect::<Vec<_>>();
    let mut part = serde_json::to_vec_pretty(&json!({
        "queryFields": query_fields,
        "types": added,
    }))
    .expect("client schema part must encode");
    part.push(b'\n');
    let spliced: Value = serde_json::from_slice(&module_client_schema(&part))
        .expect("spliced client schema must decode");
    assert_eq!(
        spliced["__schema"]["types"].as_array().map(Vec::len),
        captured["__schema"]["types"].as_array().map(Vec::len),
    );
    for (spliced, captured) in spliced["__schema"]["types"]
        .as_array()
        .into_iter()
        .flatten()
        .zip(
            captured["__schema"]["types"]
                .as_array()
                .into_iter()
                .flatten(),
        )
    {
        assert_eq!(
            spliced, captured,
            "the fixture's part does not give back the captured `{}`",
            captured["name"]
        );
    }
    for key in ["queryType", "directives"] {
        assert_eq!(spliced["__schema"][key], captured["__schema"][key], "{key}");
    }
    assert_eq!(spliced["__schemaVersion"], captured["__schemaVersion"]);
    part
}

/// Applies `change` to the unpermuted dependency schema, for a test that needs one
/// coordinate to differ.
pub(crate) fn changed_dependency_module_schema(change: impl FnOnce(&mut Value)) -> Vec<u8> {
    changed_schema(&dependency_module_schema(0), change)
}

/// Adds an object type and the `Query` field that returns it, as a module dependency
/// contributes them, with the type owned by `module` when one is given.
pub(crate) fn add_constructed_object(
    document: &mut Value,
    module: Option<&str>,
    type_name: &str,
    constructor: &str,
    fields: Vec<Value>,
) {
    let mut all = vec![object_field("id", scalar_type("ID", false), vec![])];
    all.extend(fields);
    add_object_type(document, type_name, all);
    if let Some(module) = module {
        own(document, type_name, module);
    }
    add_query_object_field(document, constructor, type_name);
}

/// Adds an object of `module` that no field returns. A module may declare a type that
/// none of its functions returns, and the engine serves it as it serves the module's
/// other objects: with an ID, `Node` among its interfaces, and `@sourceMap(module:)`.
/// Only the type's own binding names it.
pub(crate) fn add_unreturned_object(schema: &[u8], module: &str, type_name: &str) -> Vec<u8> {
    changed_schema(schema, |document| {
        add_object_type(
            document,
            type_name,
            vec![
                object_field("id", scalar_type("ID", false), vec![]),
                object_field("note", scalar_type("String", false), vec![]),
            ],
        );
        own(document, type_name, module);
        implement_node(document, type_name);
    })
}

/// Adds an enum outside the reviewed Core API that carries no `@sourceMap`, as a later
/// engine's own addition does.
pub(crate) fn add_unowned_enum(document: &mut Value, name: &str) {
    add_enum_type(document, name, &["FIRST", "SECOND"]);
}

/// A field as the dependency fixtures declare one: its name, type and arguments.
pub(crate) fn dependency_field(name: &str, type_ref: Value, args: Vec<Value>) -> Value {
    object_field(name, type_ref, args)
}

/// A non-null reference to a named object type.
pub(crate) fn dependency_object_type(name: &str) -> Value {
    object_type(name, false)
}

fn add_module_dependencies(document: &mut Value) {
    add_enum_type(document, "GreeterMode", &["LOUD", "QUIET"]);
    own(document, "GreeterMode", "greeter");

    // The interface's own functions, which the object that implements it has too: a
    // value from optional arguments, a value from required ones of every kind, the
    // interface itself as a lazy result, and a list of it.
    let named_fields = || {
        vec![
            object_field(
                "describe",
                scalar_type("String", false),
                vec![argument(
                    "prefix",
                    scalar_type("String", true),
                    Some("\"\""),
                )],
            ),
            object_field(
                "relabel",
                scalar_type("String", false),
                vec![
                    argument("label", scalar_type("String", false), None),
                    argument("tags", list_type(scalar_type("String", false), false), None),
                    typed_id_argument("source", "Directory", false),
                    argument("loud", scalar_type("Boolean", true), Some("false")),
                ],
            ),
            object_field(
                "renamed",
                interface_type("GreeterNamed", false),
                vec![argument("name", scalar_type("String", false), None)],
            ),
            object_field(
                "aliases",
                list_type(interface_type("GreeterNamed", false), false),
                vec![],
            ),
        ]
    };
    add_interface_type(document, "GreeterNamed", "GreeterMessage");
    own(document, "GreeterNamed", "greeter");
    for field in named_fields() {
        add_object_field_to_type(document, "GreeterNamed", field);
    }

    let mut message_fields = vec![
        object_field("id", scalar_type("ID", false), vec![]),
        object_field("message", scalar_type("String", false), vec![]),
        object_field("mode", enum_type("GreeterMode", false), vec![]),
    ];
    message_fields.extend(named_fields());
    add_object_type(document, "GreeterMessage", message_fields);
    own(document, "GreeterMessage", "greeter");
    add_interface_to_object(document, "GreeterMessage", "GreeterNamed");
    implement_node(document, "GreeterMessage");

    let mut source = typed_id_argument("source", "Directory", true);
    extend_directives(
        &mut source,
        &[directive_application("defaultPath", &[("path", "\"/\"")])],
    );
    add_object_type(
        document,
        "Greeter",
        vec![
            object_field("id", scalar_type("ID", false), vec![]),
            // A value from a required string.
            object_field(
                "hello",
                scalar_type("String", false),
                vec![argument("name", scalar_type("String", false), None)],
            ),
            // A value from a required argument, optional ones and an enum.
            object_field(
                "shout",
                scalar_type("String", false),
                vec![
                    argument("name", scalar_type("String", false), None),
                    argument("times", scalar_type("Int", true), Some("1")),
                    argument("mode", enum_type("GreeterMode", true), None),
                    argument(
                        "suffixes",
                        list_type(scalar_type("String", false), true),
                        None,
                    ),
                ],
            ),
            // An enum result and an enum argument.
            object_field("mode", enum_type("GreeterMode", false), vec![]),
            object_field(
                "withMode",
                object_type("Greeter", false),
                vec![argument("mode", enum_type("GreeterMode", false), None)],
            ),
            // `Void`, which the engine declares nullable.
            object_field("reset", scalar_type("Void", true), vec![]),
            // A lazy object of the dependency, a nullable one and a list of them.
            object_field(
                "message",
                object_type("GreeterMessage", false),
                vec![argument("name", scalar_type("String", false), None)],
            ),
            object_field("lastMessage", object_type("GreeterMessage", true), vec![]),
            object_field(
                "messages",
                list_type(object_type("GreeterMessage", false), false),
                vec![argument(
                    "names",
                    list_type(scalar_type("String", false), false),
                    None,
                )],
            ),
            // An interface of the dependency, as a result and as an argument.
            object_field("named", interface_type("GreeterNamed", false), vec![]),
            object_field(
                "describe",
                scalar_type("String", false),
                vec![typed_id_argument("named", "GreeterNamed", false)],
            ),
            // The dependency's own objects as arguments, one and many.
            object_field(
                "withMessage",
                object_type("Greeter", false),
                vec![typed_id_argument("message", "GreeterMessage", false)],
            ),
            object_field(
                "withMessages",
                object_type("Greeter", false),
                vec![typed_id_list_argument("messages", "GreeterMessage")],
            ),
            // Core objects: an argument, a lazy result, a nullable one and a list.
            object_field(
                "container",
                object_type("Container", false),
                vec![
                    typed_id_argument("source", "Directory", false),
                    typed_id_argument("base", "Container", true),
                ],
            ),
            object_field("maybeFile", object_type("File", true), vec![]),
            object_field(
                "files",
                list_type(object_type("File", false), false),
                vec![],
            ),
            // A Core enum and a Core scalar in both directions.
            object_field(
                "protocol",
                enum_type("NetworkProtocol", false),
                vec![argument(
                    "protocol",
                    enum_type("NetworkProtocol", false),
                    None,
                )],
            ),
            object_field(
                "platform",
                scalar_type("Platform", false),
                vec![argument("platform", scalar_type("Platform", true), None)],
            ),
            // An ID that names the object it returns.
            expected_type_field("sync", "Greeter"),
        ],
    );
    own(document, "Greeter", "greeter");
    implement_node(document, "Greeter");
    let mut constructor = object_field(
        "greeter",
        object_type("Greeter", false),
        vec![
            argument("greeting", scalar_type("String", true), Some("\"hello\"")),
            argument("mode", enum_type("GreeterMode", true), None),
            source,
        ],
    );
    constructor["directives"] = source_map_directives("greeter");
    add_query_field_value(document, constructor);

    add_object_type(
        document,
        "Vault",
        vec![
            object_field("id", scalar_type("ID", false), vec![]),
            object_field("name", scalar_type("String", false), vec![]),
        ],
    );
    own(document, "Vault", "vault");
    implement_node(document, "Vault");
    let mut constructor = object_field(
        "vault",
        object_type("Vault", false),
        vec![
            argument("name", scalar_type("String", false), None),
            typed_id_argument("mount", "Directory", false),
            argument("ttl", scalar_type("Int", true), Some("60")),
        ],
    );
    constructor["directives"] = source_map_directives("vault");
    add_query_field_value(document, constructor);

    add_object_type(
        document,
        "Clock",
        vec![
            object_field("id", scalar_type("ID", false), vec![]),
            object_field("now", scalar_type("String", false), vec![]),
        ],
    );
    own(document, "Clock", "clock");
    implement_node(document, "Clock");
    let mut constructor = object_field("clock", object_type("Clock", false), vec![]);
    constructor["directives"] = source_map_directives("clock");
    add_query_field_value(document, constructor);
}

/// Stamps a named type with the `@sourceMap(module:)` by which the engine states the
/// module that owns it.
fn own(document: &mut Value, type_name: &str, module: &str) {
    let definition = schema_array_mut(document, "types")
        .iter_mut()
        .find(|definition| definition["name"] == type_name)
        .expect("the schema must contain the type");
    definition["directives"] = source_map_directives(module);
}

/// Declares that an object implements Core's `Node`, as the engine infers for every
/// object with an `id`.
fn implement_node(document: &mut Value, object: &str) {
    add_interface_to_object(document, object, "Node");
    let node = schema_array_mut(document, "types")
        .iter_mut()
        .find(|definition| definition["name"] == "Node")
        .expect("checked schema must contain Node");
    node["possibleTypes"]
        .as_array_mut()
        .expect("an interface lists its possible types")
        .push(json!({"kind": "OBJECT", "name": object}));
}

fn add_query_field_value(document: &mut Value, field: Value) {
    let query = schema_array_mut(document, "types")
        .iter_mut()
        .find(|definition| definition["name"] == "Query")
        .expect("checked schema must contain Query");
    query["fields"]
        .as_array_mut()
        .expect("Query fields must be an array")
        .push(field);
}

/// One directive application as introspection serves it: the directive's name and its
/// arguments as encoded literals.
pub(crate) fn directive_application(name: &str, arguments: &[(&str, &str)]) -> Value {
    json!({
        "name": name,
        "args": arguments
            .iter()
            .map(|(name, value)| json!({"name": name, "value": value}))
            .collect::<Vec<_>>()
    })
}

/// Adds a module dependency the way the engine serves one to the module that depends on
/// it: the object type `Greeter`, its function `entries(source:)`, which takes a
/// directory, and the `Query.greeter` constructor. The engine describes how it fills in
/// and schedules the dependency's functions with module-authoring directives, which the
/// caller applies to the type, the function and the argument.
pub(crate) fn add_dependency_with_directives(
    schema: &[u8],
    on_type: &[Value],
    on_function: &[Value],
    on_argument: &[Value],
) -> Vec<u8> {
    let mut document: Value = serde_json::from_slice(schema).expect("schema fixture must decode");
    let mut source = typed_id_argument("source", "Directory", true);
    extend_directives(&mut source, on_argument);
    let mut entries = object_field(
        "entries",
        list_type(scalar_type("String", false), false),
        vec![source],
    );
    extend_directives(&mut entries, on_function);
    add_object_type(
        &mut document,
        "Greeter",
        vec![
            object_field("id", scalar_type("ID", false), vec![]),
            entries,
        ],
    );
    let greeter = schema_array_mut(&mut document, "types")
        .last_mut()
        .expect("the dependency type was just added");
    extend_directives(greeter, on_type);
    add_query_object_field(&mut document, "greeter", "Greeter");
    serde_json::to_vec(&document).expect("extended schema fixture must encode")
}

/// Adds a module dependency that declares an interface, as the engine serves it: the
/// object `Greeter`, the interface `GreeterNamed` that it implements and that its
/// function `named` returns, and the `Query.greeter` constructor. The engine stamps the
/// object and the interface with `@sourceMap(module:)`; the caller's directives go on
/// the interface beside it.
pub(crate) fn add_dependency_with_interface(schema: &[u8], on_interface: &[Value]) -> Vec<u8> {
    let mut document: Value = serde_json::from_slice(schema).expect("schema fixture must decode");
    add_interface_type(&mut document, "GreeterNamed", "Greeter");
    let interface = schema_array_mut(&mut document, "types")
        .last_mut()
        .expect("the dependency interface was just added");
    interface["directives"] = source_map_directives("greeter");
    extend_directives(interface, on_interface);
    add_object_type(
        &mut document,
        "Greeter",
        vec![
            object_field("id", scalar_type("ID", false), vec![]),
            object_field("message", scalar_type("String", false), vec![]),
            object_field("named", interface_type("GreeterNamed", false), vec![]),
        ],
    );
    let greeter = schema_array_mut(&mut document, "types")
        .last_mut()
        .expect("the dependency type was just added");
    greeter["directives"] = source_map_directives("greeter");
    add_interface_to_object(&mut document, "Greeter", "GreeterNamed");
    add_query_object_field(&mut document, "greeter", "Greeter");
    serde_json::to_vec(&document).expect("extended schema fixture must encode")
}

/// Adds a directive definition to the schema, with the locations it may appear at and
/// no arguments.
pub(crate) fn define_directive(schema: &[u8], name: &str, locations: &[&str]) -> Vec<u8> {
    let mut document: Value = serde_json::from_slice(schema).expect("schema fixture must decode");
    schema_array_mut(&mut document, "directives").push(json!({
        "name": name,
        "description": format!("Fixture directive {name}."),
        "locations": locations,
        "args": []
    }));
    serde_json::to_vec(&document).expect("changed schema fixture must encode")
}

/// Applies a directive to a field of the schema, or to one of the field's arguments.
pub(crate) fn apply_field_directive(
    schema: &[u8],
    owner: &str,
    field: &str,
    argument: Option<&str>,
    directive: Value,
) -> Vec<u8> {
    let mut document: Value = serde_json::from_slice(schema).expect("schema fixture must decode");
    let field = core_field_mut(&mut document, owner, field);
    let site = match argument {
        Some(argument) => field["args"]
            .as_array_mut()
            .expect("field arguments must be an array")
            .iter_mut()
            .find(|candidate| candidate["name"] == argument)
            .expect("the field must contain the argument"),
        None => field,
    };
    extend_directives(site, &[directive]);
    serde_json::to_vec(&document).expect("changed schema fixture must encode")
}

/// Applies a directive to a named type of the schema.
pub(crate) fn apply_type_directive(schema: &[u8], owner: &str, directive: Value) -> Vec<u8> {
    let mut document: Value = serde_json::from_slice(schema).expect("schema fixture must decode");
    let definition = schema_array_mut(&mut document, "types")
        .iter_mut()
        .find(|definition| definition["name"] == owner)
        .expect("the schema must contain the type");
    extend_directives(definition, &[directive]);
    serde_json::to_vec(&document).expect("changed schema fixture must encode")
}

fn extend_directives(site: &mut Value, directives: &[Value]) {
    site["directives"]
        .as_array_mut()
        .expect("a schema coordinate lists its directives")
        .extend(directives.iter().cloned());
}

/// A reviewed field that becomes nullable changes what every caller must accept, so it
/// changes the wire contract the bindings were compiled for.
fn make_container_platform_nullable(document: &mut Value) {
    let platform = core_field_mut(document, "Container", "platform");
    let non_null = platform["type"].clone();
    assert_eq!(
        non_null["kind"], "NON_NULL",
        "Container.platform must be required"
    );
    platform["type"] = non_null["ofType"].clone();
}

fn core_field_mut<'a>(document: &'a mut Value, owner: &str, field: &str) -> &'a mut Value {
    schema_array_mut(document, "types")
        .iter_mut()
        .find(|definition| definition["name"] == owner)
        .unwrap_or_else(|| panic!("checked schema must contain {owner}"))["fields"]
        .as_array_mut()
        .unwrap_or_else(|| panic!("{owner} fields must be an array"))
        .iter_mut()
        .find(|candidate| candidate["name"] == field)
        .unwrap_or_else(|| panic!("checked {owner} must have {field}"))
}

fn scrub_type(document: &mut Value, hidden: &str) {
    let definitions = std::mem::take(schema_array_mut(document, "types"));
    *schema_array_mut(document, "types") = definitions
        .into_iter()
        .filter_map(|mut definition| {
            let kind = definition["kind"].as_str().unwrap_or_default();
            let name = definition["name"].as_str().unwrap_or_default().to_owned();
            if kind == "SCALAR" {
                return (name != hidden).then_some(definition);
            }
            retain_without_type(&mut definition, "fields", hidden, field_references_type);
            retain_without_type(
                &mut definition,
                "inputFields",
                hidden,
                input_value_references_type,
            );
            if let Some(values) = definition
                .get_mut("enumValues")
                .and_then(Value::as_array_mut)
            {
                values.retain(|value| value["name"].as_str() != Some(hidden));
            }
            let empty = ["fields", "inputFields", "enumValues"].iter().all(|key| {
                definition
                    .get(*key)
                    .and_then(Value::as_array)
                    .is_none_or(Vec::is_empty)
            });
            (name != hidden && !empty).then_some(definition)
        })
        .collect();
}

fn retain_without_type(
    definition: &mut Value,
    collection: &str,
    hidden: &str,
    references: fn(&Value, &str) -> bool,
) {
    if let Some(values) = definition.get_mut(collection).and_then(Value::as_array_mut) {
        values.retain(|value| !references(value, hidden));
    }
}

fn field_references_type(field: &Value, hidden: &str) -> bool {
    type_ref_references(&field["type"], hidden)
        || field["args"].as_array().is_some_and(|arguments| {
            arguments
                .iter()
                .any(|arg| input_value_references_type(arg, hidden))
        })
}

fn input_value_references_type(value: &Value, hidden: &str) -> bool {
    type_ref_references(&value["type"], hidden)
}

fn type_ref_references(type_ref: &Value, hidden: &str) -> bool {
    type_ref["name"].as_str() == Some(hidden)
        || type_ref
            .get("ofType")
            .is_some_and(|nested| type_ref_references(nested, hidden))
}

fn scrub_field(document: &mut Value, hidden: &str) {
    let Some((type_name, field_name)) = hidden.split_once('.') else {
        return;
    };
    let Some(definition) = schema_array_mut(document, "types")
        .iter_mut()
        .find(|definition| definition["name"].as_str() == Some(type_name))
    else {
        return;
    };
    if let Some(fields) = definition.get_mut("fields").and_then(Value::as_array_mut) {
        fields.retain(|field| field["name"].as_str() != Some(field_name));
    }
}

fn add_extension(document: &mut Value, name: &str, add_field: bool, module: Option<&str>) {
    let directives = module.map_or_else(|| json!([]), source_map_directives);
    schema_array_mut(document, "types").push(json!({
        "kind": "ENUM",
        "name": name,
        "description": "Operation-scoped Rust fixture enum.",
        "enumValues": [{
            "name": "READY",
            "description": "Ready for the operation.",
            "isDeprecated": false,
            "deprecationReason": null,
            "directives": []
        }],
        "interfaces": [],
        "possibleTypes": [],
        "directives": directives
    }));
    if add_field {
        add_query_field(document, "rustMode", name, module);
    }
}

fn add_query_field(document: &mut Value, field_name: &str, type_name: &str, module: Option<&str>) {
    let query = schema_array_mut(document, "types")
        .iter_mut()
        .find(|definition| definition["name"] == "Query")
        .expect("checked schema must contain Query");
    query["fields"]
        .as_array_mut()
        .expect("Query fields must be an array")
        .push(json!({
            "name": field_name,
            "description": "Operation-scoped Rust fixture field.",
            "type": {"kind": "NON_NULL", "ofType": {"kind": "ENUM", "name": type_name}},
            "args": [],
            "isDeprecated": false,
            "deprecationReason": null,
            "directives": module.map_or_else(|| json!([]), source_map_directives)
        }));
}

fn source_map_directives(module: &str) -> Value {
    json!([{
        "name": "sourceMap",
        "args": [{
            "name": "module",
            "value": serde_json::to_string(module).expect("fixture module must encode")
        }]
    }])
}

fn permute_schema(document: &mut Value, permutation: u16) {
    let types = schema_array_mut(document, "types");
    if !types.is_empty() {
        let offset = usize::from(permutation) % types.len();
        types.rotate_left(offset);
    }
    for definition in types {
        for collection in ["fields", "inputFields", "enumValues"] {
            let Some(values) = definition.get_mut(collection).and_then(Value::as_array_mut) else {
                continue;
            };
            if !values.is_empty() {
                let offset = usize::from(permutation.rotate_left(3)) % values.len();
                values.rotate_left(offset);
            }
        }
    }
    let directives = schema_array_mut(document, "directives");
    if !directives.is_empty() {
        let offset = usize::from(permutation.rotate_left(7)) % directives.len();
        directives.rotate_left(offset);
    }
}

fn schema_array_mut<'a>(document: &'a mut Value, name: &str) -> &'a mut Vec<Value> {
    document["__schema"][name]
        .as_array_mut()
        .expect("checked schema collection must be an array")
}

pub(crate) fn pure_config() -> Config {
    Config {
        cases: PURE_CASES,
        failure_persistence: Some(Box::new(FileFailurePersistence::Direct(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/proptest-regressions/foundations.txt"
        )))),
        ..Config::default()
    }
}

pub(crate) fn filesystem_config() -> Config {
    Config {
        cases: FILESYSTEM_CASES,
        failure_persistence: Some(Box::new(FileFailurePersistence::Direct(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/proptest-regressions/filesystem.txt"
        )))),
        ..Config::default()
    }
}

pub(crate) fn name_strategy() -> impl Strategy<Value = String> {
    "[A-Za-z_][A-Za-z0-9_]{0,31}"
}

pub(crate) fn default_literal_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("null".to_owned()),
        any::<bool>().prop_map(|value| value.to_string()),
        any::<i32>().prop_map(|value| value.to_string()),
        "[A-Za-z0-9 _-]{0,24}".prop_map(|value| format!("\"{value}\"")),
    ]
}

pub(crate) fn target_strategy() -> impl Strategy<Value = CodegenTarget> {
    Just(
        CodegenTarget::decode_exact(include_bytes!("../../../../codegen/target.json"))
            .expect("checked target must decode"),
    )
}

pub(crate) fn wrapper_strategy() -> impl Strategy<Value = TypeRef> {
    let leaf = name_strategy().prop_map(|name| TypeRef {
        kind: Some(TypeKind::Scalar),
        name: Some(name),
        of_type: None,
        interfaces: None,
        possible_types: None,
        directives: None,
        unknown_fields: BTreeMap::new(),
    });
    leaf.prop_recursive(8, 64, 2, |inner| {
        prop_oneof![
            inner.clone().prop_map(|inner| TypeRef {
                kind: Some(TypeKind::List),
                name: None,
                of_type: Some(Box::new(inner)),
                interfaces: None,
                possible_types: None,
                directives: None,
                unknown_fields: BTreeMap::new(),
            }),
            inner.prop_map(|inner| TypeRef {
                kind: Some(TypeKind::NonNull),
                name: None,
                of_type: Some(Box::new(inner)),
                interfaces: None,
                possible_types: None,
                directives: None,
                unknown_fields: BTreeMap::new(),
            }),
        ]
    })
}

pub(crate) fn directive_strategy() -> impl Strategy<Value = DirectiveApplication> {
    (
        name_strategy(),
        prop::collection::vec((name_strategy(), default_literal_strategy()), 0..4),
    )
        .prop_map(|(name, arguments)| DirectiveApplication {
            name,
            args: arguments
                .into_iter()
                .map(|(name, value)| DirectiveApplicationArgument {
                    name,
                    value: Some(value),
                    unknown_fields: BTreeMap::new(),
                })
                .collect(),
            unknown_fields: BTreeMap::new(),
        })
}

pub(crate) fn raw_schema_fragment_strategy() -> impl Strategy<Value = FullType> {
    (
        name_strategy(),
        prop::option::of(".{0,32}"),
        prop::collection::vec(directive_strategy(), 0..3),
    )
        .prop_map(|(name, description, _directives)| FullType {
            kind: Some(TypeKind::Scalar),
            name: Some(name),
            description,
            fields: None,
            input_fields: None,
            interfaces: None,
            enum_values: None,
            possible_types: None,
            directives: None,
            unknown_fields: BTreeMap::new(),
        })
}

pub(crate) fn canonical_schema_fragment_strategy() -> impl Strategy<Value = String> {
    name_strategy().prop_filter("canonical fragments exclude introspection names", |name| {
        !name.starts_with("__")
    })
}

pub(crate) fn catalog_strategy() -> impl Strategy<Value = Vec<CatalogEntry>> {
    prop::collection::btree_map(
        name_strategy(),
        prop_oneof![
            Just(CatalogDisposition::Emitted),
            Just(CatalogDisposition::RuntimeProvided),
            Just(CatalogDisposition::PolicyRecorded),
        ],
        0..24,
    )
    .prop_map(|entries| {
        entries
            .into_iter()
            .map(|(schema_id, disposition)| CatalogEntry {
                schema_id,
                disposition,
                reason: None,
            })
            .collect()
    })
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ArtifactSet(pub BTreeMap<PathBuf, Vec<u8>>);

pub(crate) fn artifact_set_strategy() -> impl Strategy<Value = ArtifactSet> {
    prop::collection::btree_map(
        "[a-z][a-z0-9_-]{0,12}\\.rs",
        prop::collection::vec(any::<u8>(), 0..64),
        0..8,
    )
    .prop_map(|entries| {
        ArtifactSet(
            entries
                .into_iter()
                .map(|(path, bytes)| (path.into(), bytes))
                .collect(),
        )
    })
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CapabilityMapping {
    pub capability_id: String,
    pub schema_id: String,
}

pub(crate) fn capability_mapping_strategy() -> impl Strategy<Value = CapabilityMapping> {
    (name_strategy(), name_strategy()).prop_map(|(capability_id, schema_id)| CapabilityMapping {
        capability_id: format!("schema/engine-schema/{capability_id}"),
        schema_id,
    })
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct EvidenceRecord {
    pub subject: String,
    pub digest: [u8; 32],
}

pub(crate) fn evidence_record_strategy() -> impl Strategy<Value = EvidenceRecord> {
    (name_strategy(), any::<[u8; 32]>())
        .prop_map(|(subject, digest)| EvidenceRecord { subject, digest })
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct RecordingSelection {
    pub fields: Vec<String>,
}

impl RecordingSelection {
    pub(crate) fn select(&mut self, wire_name: impl Into<String>) {
        self.fields.push(wire_name.into());
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct RecordingSession {
    pub executions: Vec<Vec<String>>,
}

impl RecordingSession {
    pub(crate) fn execute(&mut self, selection: &RecordingSelection) {
        self.executions.push(selection.fields.clone());
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct RecordingFormatter {
    pub candidates: Vec<Vec<u8>>,
}

impl RecordingFormatter {
    pub(crate) fn format(&mut self, candidate: &[u8]) -> Vec<u8> {
        self.candidates.push(candidate.to_vec());
        candidate.to_vec()
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct RecordingFilesystem {
    pub files: BTreeMap<PathBuf, Vec<u8>>,
    pub events: Vec<String>,
}

impl RecordingFilesystem {
    pub(crate) fn write(&mut self, path: impl Into<PathBuf>, bytes: Vec<u8>) {
        let path = path.into();
        self.events.push(format!("write:{}", path.display()));
        self.files.insert(path, bytes);
    }

    pub(crate) fn read(&mut self, path: &Path) -> Option<Vec<u8>> {
        self.events.push(format!("read:{}", path.display()));
        self.files.get(path).cloned()
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct RecordingPublicationLock {
    held: Arc<AtomicBool>,
    acquisitions: Arc<AtomicUsize>,
}

impl RecordingPublicationLock {
    pub(crate) fn try_acquire(&self) -> Option<RecordingPublicationGuard> {
        self.held
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .ok()
            .map(|_| {
                self.acquisitions.fetch_add(1, Ordering::Relaxed);
                RecordingPublicationGuard {
                    held: Arc::clone(&self.held),
                }
            })
    }

    pub(crate) fn acquisitions(&self) -> usize {
        self.acquisitions.load(Ordering::Relaxed)
    }
}

#[derive(Debug)]
pub(crate) struct RecordingPublicationGuard {
    held: Arc<AtomicBool>,
}

impl Drop for RecordingPublicationGuard {
    fn drop(&mut self) {
        self.held.store(false, Ordering::Release);
    }
}
