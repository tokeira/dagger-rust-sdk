//! A later engine that serves the target's API view: what the generator accepts, what
//! it refuses, and the advice it gives an engine that serves another view.
//!
//! The drift fixture records the five descriptions that dagger/dagger's main branch
//! reworded, at commit 2a385e9, in its copy of the v1.0.0-beta.27 API view after that
//! release shipped. Applied to the checked schema, it reproduces the schema that commit
//! served byte for byte.

mod support;

use dagger_codegen::diagnostic::{DiagnosticCode, DiagnosticSet};
use dagger_codegen::engine::{
    OperationKind, OperationPlan, OperationProjectionRequest, PublishedSdkDependency,
    RelativeOperationPath, project_operation,
};
use dagger_codegen::target::CodegenTarget;
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest as _, Sha256};

use support::{
    CORE_SCHEMA_BYTES, TARGET_BYTES, VisibleSchemaCase, add_later_core_field,
    reword_core_documentation, visible_schema,
};

const DRIFT: &[u8] = include_bytes!("fixtures/later-engine-drift.json");
const LATER_VIEW_SHA256: &str = "6f76f6f5acdbe004cd53ea08ade63d933cb3d6e278eca3b836ce89d52f5f60f4";
const LATER_RELEASE: &str = "v1.0.0-beta.28+1a2b3c4d";

/// One description a later engine reworded, at a schema coordinate.
#[derive(Deserialize)]
struct Drift {
    coordinate: String,
    before: String,
    after: String,
}

fn drift() -> Vec<Drift> {
    serde_json::from_slice(DRIFT).expect("drift fixture must decode")
}

/// The checked schema as the later engine served it, built by byte replacement so the
/// result can be compared with the observed schema's digest.
fn later_view() -> Vec<u8> {
    let mut schema =
        String::from_utf8(CORE_SCHEMA_BYTES.to_vec()).expect("checked schema must be UTF-8");
    for entry in drift() {
        let before = serde_json::to_string(&entry.before).expect("description must encode");
        let after = serde_json::to_string(&entry.after).expect("description must encode");
        assert_eq!(
            schema.matches(&before).count(),
            1,
            "{} must be described exactly once",
            entry.coordinate
        );
        schema = schema.replacen(&before, &after, 1);
    }
    schema.into_bytes()
}

/// Applies the drift fixture to any schema document that carries the coordinates.
fn apply_drift(document: &mut Value) {
    for entry in drift() {
        let (owner, member) = entry
            .coordinate
            .split_once('.')
            .expect("drift coordinates name a member");
        let (field, argument) = match member.split_once('(') {
            Some((field, argument)) => (field, Some(argument.trim_end_matches(":)"))),
            None => (member, None),
        };
        let field = named(&mut definition(document, owner)["fields"], field);
        let description = match argument {
            Some(argument) => &mut named(&mut field["args"], argument)["description"],
            None => &mut field["description"],
        };
        assert_eq!(description.as_str(), Some(entry.before.as_str()));
        *description = Value::String(entry.after);
    }
}

fn edit(schema: &[u8], change: impl FnOnce(&mut Value)) -> Vec<u8> {
    let mut document: Value = serde_json::from_slice(schema).expect("schema fixture must decode");
    change(&mut document);
    serde_json::to_vec(&document).expect("schema fixture must encode")
}

fn definition<'a>(document: &'a mut Value, name: &str) -> &'a mut Value {
    named(&mut document["__schema"]["types"], name)
}

fn named<'a>(collection: &'a mut Value, name: &str) -> &'a mut Value {
    collection
        .as_array_mut()
        .expect("schema collection must be an array")
        .iter_mut()
        .find(|member| member["name"] == name)
        .unwrap_or_else(|| panic!("schema fixture must contain {name}"))
}

fn input_value(name: &str, type_ref: Value, default: Option<&str>) -> Value {
    json!({
        "name": name,
        "description": format!("A later engine's {name}."),
        "type": type_ref,
        "defaultValue": default,
        "isDeprecated": false,
        "deprecationReason": null,
        "directives": []
    })
}

fn required(type_ref: Value) -> Value {
    json!({"kind": "NON_NULL", "ofType": type_ref})
}

fn scalar(name: &str) -> Value {
    json!({"kind": "SCALAR", "name": name})
}

fn project(schema: &[u8], engine_version: Option<&str>) -> Result<OperationPlan, DiagnosticSet> {
    let target = CodegenTarget::decode_exact(TARGET_BYTES).expect("checked target must decode");
    let output = RelativeOperationPath::parse("generated").expect("fixture path must parse");
    let dependency = PublishedSdkDependency::Registry {
        registry: "crates-io".to_owned(),
        exact_version: "1.0.0-beta.11.rust.5".to_owned(),
    };
    project_operation(OperationProjectionRequest {
        target: &target,
        operation: OperationKind::GenerateLibrary,
        visible_schema_json: schema,
        module: None,
        output: &output,
        sdk_dependency: &dependency,
        authoring: None,
        engine_version,
    })
}

#[test]
fn the_drift_fixture_reproduces_the_later_engine_schema() {
    let digest = Sha256::digest(later_view())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    assert_eq!(digest, LATER_VIEW_SHA256);
}

// The later engine's copy of the target view is the same wire contract with other
// words, so it projects exactly the plan the checked schema projects, whatever release
// the engine states.
#[test]
fn a_later_engine_copy_of_the_target_view_projects_the_target_plan() {
    let expected = project(CORE_SCHEMA_BYTES, None).expect("checked schema must project");
    for engine_version in [None, Some(LATER_RELEASE)] {
        let plan = project(&later_view(), engine_version)
            .unwrap_or_else(|diagnostics| panic!("later view was refused: {diagnostics:?}"));
        assert_eq!(plan, expected);
    }

    let extended = visible_schema(VisibleSchemaCase::EngineModuleExtension, 0);
    let expected = project(&extended, None).expect("extended schema must project");
    let plan = project(&edit(&extended, apply_drift), Some(LATER_RELEASE))
        .unwrap_or_else(|diagnostics| panic!("later view was refused: {diagnostics:?}"));
    assert_eq!(plan, expected);
}

// Documentation, deprecation and additions that no binding compiled for the target can
// reach leave the plan unchanged.
#[test]
fn later_documentation_and_unconsumed_additions_project_the_target_plan() {
    let baseline = visible_schema(VisibleSchemaCase::EngineModuleExtension, 0);
    let expected = project(&baseline, None).expect("target view must project");
    let cases = [
        (
            "reworded documentation",
            reword_core_documentation(&baseline),
        ),
        (
            "a new deprecation",
            edit(&baseline, |document| {
                let field = named(&mut definition(document, "Container")["fields"], "withExec");
                field["isDeprecated"] = json!(true);
                field["deprecationReason"] = json!("Use a later field.");
                field["directives"] = json!([{
                    "name": "deprecated",
                    "args": [{"name": "reason", "value": "\"Use a later field.\""}]
                }]);
            }),
        ),
        ("a new core field", add_later_core_field(&baseline)),
        (
            "a new optional argument",
            edit(&baseline, |document| {
                named(&mut definition(document, "Container")["fields"], "withExec")["args"]
                    .as_array_mut()
                    .expect("arguments must be an array")
                    .push(input_value("later", scalar("Boolean"), Some("false")));
            }),
        ),
        (
            "a new optional input field",
            edit(&baseline, |document| {
                definition(document, "BuildArg")["inputFields"]
                    .as_array_mut()
                    .expect("input fields must be an array")
                    .push(input_value("later", scalar("String"), None));
            }),
        ),
    ];
    for (change, schema) in cases {
        let plan = project(&schema, Some(LATER_RELEASE))
            .unwrap_or_else(|diagnostics| panic!("{change} was refused: {diagnostics:?}"));
        assert_eq!(plan, expected, "{change} changed the plan");
    }
}

// A change to the wire contract of a reviewed core definition is refused, whether the
// definition loses something callers rely on or gains something they must now send.
#[test]
fn later_wire_changes_to_reviewed_core_are_refused() {
    let baseline = visible_schema(VisibleSchemaCase::EngineModuleExtension, 0);
    let missing = DiagnosticCode::SchemaCoreCoordinateMissing;
    let incompatible = DiagnosticCode::SchemaCoreCoordinateIncompatible;
    let cases = [
        (
            "a removed field",
            missing,
            "visible schema omits a reviewed core coordinate",
            edit(&baseline, |document| {
                definition(document, "Container")["fields"]
                    .as_array_mut()
                    .expect("fields must be an array")
                    .retain(|field| field["name"] != "withExec");
            }),
        ),
        (
            "a removed enum value",
            missing,
            "visible schema omits a reviewed core coordinate",
            edit(&baseline, |document| {
                definition(document, "NetworkProtocol")["enumValues"]
                    .as_array_mut()
                    .expect("enum values must be an array")
                    .retain(|value| value["name"] != "UDP");
            }),
        ),
        (
            "a nullable field",
            incompatible,
            "visible schema changes the wire contract of a reviewed core coordinate",
            edit(&baseline, |document| {
                let field = named(&mut definition(document, "Container")["fields"], "platform");
                field["type"] = scalar("Platform");
            }),
        ),
        (
            "a retyped argument",
            incompatible,
            "visible schema changes the wire contract of a reviewed core coordinate",
            edit(&baseline, |document| {
                let field = named(&mut definition(document, "Container")["fields"], "withExec");
                named(&mut field["args"], "args")["type"] =
                    json!({"kind": "LIST", "ofType": required(scalar("String"))});
            }),
        ),
        (
            "a changed default",
            incompatible,
            "visible schema changes the wire contract of a reviewed core coordinate",
            edit(&baseline, |document| {
                let field = named(&mut definition(document, "Container")["fields"], "withExec");
                named(&mut field["args"], "useEntrypoint")["defaultValue"] = json!("true");
            }),
        ),
        (
            "a new enum value",
            incompatible,
            "visible schema adds a value to a reviewed core enum",
            edit(&baseline, |document| {
                definition(document, "NetworkProtocol")["enumValues"]
                    .as_array_mut()
                    .expect("enum values must be an array")
                    .push(json!({
                        "name": "SCTP",
                        "description": "A later engine's protocol.",
                        "isDeprecated": false,
                        "deprecationReason": null,
                        "directives": []
                    }));
            }),
        ),
        (
            "a new required argument",
            incompatible,
            "visible schema adds a required argument to a reviewed core field",
            edit(&baseline, |document| {
                named(&mut definition(document, "Container")["fields"], "withExec")["args"]
                    .as_array_mut()
                    .expect("arguments must be an array")
                    .push(input_value("later", required(scalar("Boolean")), None));
            }),
        ),
        (
            "a new required input field",
            incompatible,
            "visible schema adds a required field to a reviewed core input object",
            edit(&baseline, |document| {
                definition(document, "BuildArg")["inputFields"]
                    .as_array_mut()
                    .expect("input fields must be an array")
                    .push(input_value("later", required(scalar("String")), None));
            }),
        ),
    ];
    for (change, code, message, schema) in cases {
        let diagnostics =
            project(&schema, Some(LATER_RELEASE)).expect_err(&format!("{change} must be refused"));
        assert!(
            diagnostics
                .diagnostics()
                .iter()
                .any(|diagnostic| diagnostic.code == code && diagnostic.message == message),
            "{change} gave {diagnostics:?}"
        );
    }
}

// An engine that serves another view is told what to change: a module declares the
// target release when the engine could serve it, and otherwise the engine is older than
// the target and only another engine helps.
#[test]
fn another_view_gets_advice_that_fits_the_engine() {
    let target = CodegenTarget::decode_exact(TARGET_BYTES).expect("checked target must decode");
    let required = format!("v{}", target.schema_version());
    let declare = |view: &str| {
        format!(
            "the engine's schema states the {view} API view, and this SDK requires the \
             {required} view; a module selects its view with engineVersion, so set \
             engineVersion = \"{required}\" in its dagger-module.toml"
        )
    };
    let upgrade = |release: &str, view: &str| {
        format!(
            "the engine is {release}, older than {required}, the earliest Dagger release this \
             SDK supports, and its schema states the {view} API view; run the module on a \
             {required} or later engine"
        )
    };
    let cases = [
        // A later engine serves a module the view its engineVersion declares.
        (
            "v1.0.0-beta.28",
            Some(LATER_RELEASE),
            declare("v1.0.0-beta.28"),
        ),
        // An earlier engine caps every view at its own release.
        (
            "v1.0.0-beta.25",
            Some("v1.0.0-beta.25+0123abcd"),
            upgrade("v1.0.0-beta.25+0123abcd", "v1.0.0-beta.25"),
        ),
        // Before v1.0.0-beta.22 every engine served one view, named v1.0.0.
        (
            "v1.0.0",
            Some("v1.0.0-beta.21+e5f6a7b8"),
            upgrade("v1.0.0-beta.21+e5f6a7b8", "v1.0.0"),
        ),
        // Without a release to compare, the module advice is the one that can help.
        ("v1.0.0-beta.25", None, declare("v1.0.0-beta.25")),
        ("v1.0.0-beta.25", Some("devel"), declare("v1.0.0-beta.25")),
    ];
    for (view, engine_version, advice) in cases {
        let schema = edit(
            &visible_schema(VisibleSchemaCase::EngineModuleExtension, 0),
            |document| document["__schemaVersion"] = Value::from(view),
        );
        let diagnostics = project(&schema, engine_version)
            .expect_err(&format!("the {view} view must be refused"));
        assert!(
            diagnostics.diagnostics().iter().any(|diagnostic| {
                diagnostic.code == DiagnosticCode::TargetIdentityInvalid
                    && diagnostic.message == advice
            }),
            "{view} on {engine_version:?} gave {diagnostics:?}"
        );
    }
}
