//! Module-authoring directives on a dependency's coordinates and on reviewed Core's.
//!
//! The target defines ten directives with which the engine describes how it fills in
//! and schedules a module's functions: `@agent`, `@cache`, `@up`, `@keys`, `@get` and
//! `@delta` on a field, `@defaultAddress`, `@defaultPath` and `@ignorePatterns` on an
//! argument, and `@collection` on an object type. At the target, core/typedef.go applies
//! the function's (`Function.Directives`) and the argument's
//! (`FunctionArg.Directives`), and core/object.go the type's
//! (`ModuleObject.TypeDefinition`). A module that depends on another module is served
//! that module's coordinates with them. None changes what a caller sends, so generation
//! accepts each of them there, where its definition places it, and refuses each of them
//! on a coordinate of the reviewed Core API.

mod support;

use dagger_codegen::diagnostic::{DiagnosticCode, DiagnosticSet};
use dagger_codegen::directive::{DirectiveApplicationPolicy, DirectivePolicy};
use dagger_codegen::engine::{
    ModuleProjectionInput, OperationKind, OperationProjectionRequest, PublishedSdkDependency,
    RelativeOperationPath, VisibleSchemaPlan, project_operation, project_visible_schema,
};
use dagger_codegen::schema::canonical::{SchemaCoordinate, SchemaName};
use dagger_codegen::target::CodegenTarget;
use serde_json::Value;

use support::{
    CORE_SCHEMA_BYTES, TARGET_BYTES, add_dependency_with_directives, apply_field_directive,
    apply_type_directive, directive_application, module_authoring_input, module_visible_schema,
};

/// Where a directive's definition lets the engine apply it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Site {
    Type,
    Function,
    Argument,
}

/// The ten module-authoring directives, each where the target's definition places it,
/// with arguments as the engine encodes them.
fn module_authoring_directives() -> Vec<(Site, Value)> {
    vec![
        (Site::Type, directive_application("collection", &[])),
        (Site::Function, directive_application("agent", &[])),
        (
            Site::Function,
            directive_application("cache", &[("policy", "PER_SESSION"), ("ttl", "\"1m0s\"")]),
        ),
        (Site::Function, directive_application("delta", &[])),
        (Site::Function, directive_application("get", &[])),
        (Site::Function, directive_application("keys", &[])),
        (Site::Function, directive_application("up", &[])),
        (
            Site::Argument,
            directive_application("defaultAddress", &[("address", "\"alpine:3\"")]),
        ),
        (
            Site::Argument,
            directive_application("defaultPath", &[("path", "\"/\"")]),
        ),
        (
            Site::Argument,
            directive_application("ignorePatterns", &[("patterns", "[\"target\"]")]),
        ),
    ]
}

fn name(directive: &Value) -> &str {
    directive["name"]
        .as_str()
        .expect("a directive application has a name")
}

fn target() -> CodegenTarget {
    CodegenTarget::decode_exact(TARGET_BYTES).expect("checked target must decode")
}

fn project(schema: &[u8]) -> Result<VisibleSchemaPlan, DiagnosticSet> {
    project_visible_schema(&target(), schema)
}

/// A dependency that carries every module-authoring directive at once.
fn dependency_with_every_directive(schema: &[u8]) -> Vec<u8> {
    let at = |site: Site| {
        module_authoring_directives()
            .into_iter()
            .filter(|(candidate, _)| *candidate == site)
            .map(|(_, directive)| directive)
            .collect::<Vec<_>>()
    };
    add_dependency_with_directives(
        schema,
        &at(Site::Type),
        &at(Site::Function),
        &at(Site::Argument),
    )
}

fn coordinate(site: Site) -> SchemaCoordinate {
    let owner = SchemaName::try_from("Greeter").expect("fixture type name is valid");
    let field = SchemaName::try_from("entries").expect("fixture field name is valid");
    let argument = SchemaName::try_from("source").expect("fixture argument name is valid");
    match site {
        Site::Type => SchemaCoordinate::named_type(&owner),
        Site::Function => SchemaCoordinate::field(&owner, &field),
        Site::Argument => SchemaCoordinate::argument(&owner, &field, &argument),
    }
}

#[test]
fn the_fixture_names_every_target_inactive_directive() {
    let plan = project(CORE_SCHEMA_BYTES).expect("checked schema must project");
    let inactive = plan
        .projection()
        .directives()
        .records()
        .values()
        .filter(|record| record.policy == DirectivePolicy::TargetInactive)
        .map(|record| record.name.as_str().to_owned())
        .collect::<Vec<_>>();
    let mut fixture = module_authoring_directives()
        .iter()
        .map(|(_, directive)| name(directive).to_owned())
        .collect::<Vec<_>>();
    fixture.sort();
    assert_eq!(fixture, inactive);
}

#[test]
fn each_module_authoring_directive_is_accepted_on_a_dependency_coordinate() {
    for (site, directive) in module_authoring_directives() {
        let one = std::slice::from_ref(&directive);
        let schema = match site {
            Site::Type => add_dependency_with_directives(CORE_SCHEMA_BYTES, one, &[], &[]),
            Site::Function => add_dependency_with_directives(CORE_SCHEMA_BYTES, &[], one, &[]),
            Site::Argument => add_dependency_with_directives(CORE_SCHEMA_BYTES, &[], &[], one),
        };
        let plan = project(&schema).unwrap_or_else(|error| {
            panic!(
                "@{} on a dependency's {site:?} must be accepted: {error:?}",
                name(&directive)
            )
        });
        let coordinate = coordinate(site);
        assert!(plan.extension_coordinates().contains(&coordinate));
        let expected = DirectiveApplicationPolicy::ModuleAuthoring {
            directive: SchemaName::try_from(name(&directive)).expect("directive name is valid"),
        };
        assert!(
            plan.projection()
                .directives()
                .for_coordinate(&coordinate)
                .contains(&expected),
            "@{} is recorded at {}",
            name(&directive),
            coordinate.as_str()
        );
    }
}

// The engine serves a dependency's constructor as a field of Core's `Query`. The field
// is the dependency's, not reviewed Core's, though its owner is a Core type.
#[test]
fn a_dependency_constructor_on_query_may_carry_a_function_directive() {
    let schema = apply_field_directive(
        &add_dependency_with_directives(CORE_SCHEMA_BYTES, &[], &[], &[]),
        "Query",
        "greeter",
        None,
        directive_application("cache", &[("policy", "NEVER")]),
    );
    let plan = project(&schema).expect("a dependency's constructor must be accepted");
    let query = SchemaName::try_from("Query").expect("fixture type name is valid");
    let greeter = SchemaName::try_from("greeter").expect("fixture field name is valid");
    let constructor = SchemaCoordinate::field(&query, &greeter);
    assert!(plan.extension_coordinates().contains(&constructor));
    assert_eq!(
        plan.projection().directives().for_coordinate(&constructor),
        [DirectiveApplicationPolicy::ModuleAuthoring {
            directive: SchemaName::try_from("cache").expect("directive name is valid"),
        }]
    );
}

// The schema `dagger generate` is served for a module that depends on a module with a
// function `entries(source: Directory! @defaultPath(path: "/"))`.
#[test]
fn a_module_generates_beside_a_dependency_that_carries_every_directive() {
    let schema = dependency_with_every_directive(&module_visible_schema(0));
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
    let plan = project_operation(OperationProjectionRequest {
        target: &target(),
        operation: OperationKind::GenerateModule,
        visible_schema_json: &schema,
        module: Some(&module),
        output: &output,
        sdk_dependency: &dependency,
        authoring: Some(&module_authoring_input()),
        engine_version: None,
    })
    .expect("a dependency's module-authoring directives must not stop generation");
    let binding = RelativeOperationPath::parse("candidate/src/dagger_generated/greeter.rs")
        .expect("expected path must parse");
    assert!(plan.artifacts().contains_key(&binding));
}

// The directives take no part in what a caller sends, so the dependency's binding is
// the same source with them and without them.
#[test]
fn the_directives_leave_the_dependency_binding_unchanged() {
    let plain = add_dependency_with_directives(CORE_SCHEMA_BYTES, &[], &[], &[]);
    let directed = dependency_with_every_directive(CORE_SCHEMA_BYTES);
    let output = RelativeOperationPath::parse("generated").expect("fixture path must parse");
    let dependency = PublishedSdkDependency::Registry {
        registry: "crates-io".to_owned(),
        exact_version: "1.0.0-beta.11.rust.5".to_owned(),
    };
    let render = |schema: &[u8]| {
        let plan = project_operation(OperationProjectionRequest {
            target: &target(),
            operation: OperationKind::GenerateLibrary,
            visible_schema_json: schema,
            module: None,
            output: &output,
            sdk_dependency: &dependency,
            authoring: None,
            engine_version: None,
        })
        .expect("the dependency must project");
        let path =
            RelativeOperationPath::parse("generated/greeter.rs").expect("expected path must parse");
        let source = String::from_utf8(plan.artifacts()[&path].content.clone())
            .expect("generated Rust is UTF-8");
        // The header names the consumed schema's identity, which the directives are
        // part of; the binding below it is what a caller compiles against.
        source
            .lines()
            .filter(|line| !line.starts_with("// @generated"))
            .collect::<Vec<_>>()
            .join("\n")
    };
    assert_eq!(render(&plain), render(&directed));
}

#[test]
fn each_module_authoring_directive_is_refused_on_a_core_coordinate() {
    for (site, directive) in module_authoring_directives() {
        let schema = match site {
            Site::Type => apply_type_directive(CORE_SCHEMA_BYTES, "Container", directive.clone()),
            Site::Function => apply_field_directive(
                CORE_SCHEMA_BYTES,
                "Container",
                "from",
                None,
                directive.clone(),
            ),
            Site::Argument => apply_field_directive(
                CORE_SCHEMA_BYTES,
                "Container",
                "from",
                Some("address"),
                directive.clone(),
            ),
        };
        let refused = project(&schema).expect_err("a Core coordinate must stay as reviewed");
        assert!(
            refused.contains(DiagnosticCode::SchemaCoreCoordinateIncompatible),
            "@{} on a Core {site:?}: {refused:?}",
            name(&directive)
        );
    }
}
