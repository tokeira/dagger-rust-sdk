//! Fixed operation-matrix and production-renderer regressions.

mod support;

use dagger_codegen::diagnostic::DiagnosticCode;
use dagger_codegen::engine::{
    ContentDomain, ModuleProjectionInput, OperationKind, OperationProjectionRequest,
    ProductionRenderers, PublishedSdkDependency, RelativeOperationPath, project_operation,
    project_operation_with,
};
use dagger_codegen::target::CodegenTarget;

use support::{
    CORE_SCHEMA_BYTES, MODULE_AUTHORING_SOURCE, TARGET_BYTES, VisibleSchemaCase,
    minimal_module_schema, module_authoring_input, module_authoring_input_from,
    module_visible_schema, visible_schema,
};

fn target() -> CodegenTarget {
    CodegenTarget::decode_exact(TARGET_BYTES).expect("checked target must decode")
}

fn module() -> ModuleProjectionInput {
    ModuleProjectionInput {
        name: "rust-probe".to_owned(),
        original_name: "RustProbe".to_owned(),
        source_subpath: RelativeOperationPath::parse("modules/rust-probe")
            .expect("fixture path must parse"),
        source_digest: "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
            .to_owned(),
    }
}

fn dependency() -> PublishedSdkDependency {
    PublishedSdkDependency::Registry {
        registry: "crates-io".to_owned(),
        exact_version: "1.0.0-beta.11.rust.5".to_owned(),
    }
}

#[test]
fn production_renderers_emit_bounded_operation_specific_artifacts() {
    let target = target();
    let module = module();
    let dependency = dependency();
    let authoring = module_authoring_input();
    let output = RelativeOperationPath::parse("candidate").expect("fixture path must parse");
    for operation in [
        OperationKind::GenerateLibrary,
        OperationKind::GenerateModule,
        OperationKind::GenerateEntrypoint,
    ] {
        let schema = visible_schema(VisibleSchemaCase::EngineModuleExtension, 0);
        let selected_module = match operation {
            OperationKind::GenerateLibrary => None,
            OperationKind::GenerateModule | OperationKind::GenerateEntrypoint => Some(&module),
        };
        let selected_authoring = matches!(
            operation,
            OperationKind::GenerateModule | OperationKind::GenerateEntrypoint
        )
        .then_some(&authoring);
        let plan = project_operation(OperationProjectionRequest {
            target: &target,
            operation,
            visible_schema_json: &schema,
            module: selected_module,
            output: &output,
            sdk_dependency: &dependency,
            authoring: selected_authoring,
            engine_version: None,
        })
        .expect("valid operation fixture must render");

        assert_eq!(plan.operation(), operation);
        assert_eq!(plan.projection_pass_limit(), 1);
        assert!(
            plan.artifacts()
                .keys()
                .all(|path| path.as_str().starts_with("candidate/"))
        );
        match operation {
            OperationKind::GenerateLibrary => {
                assert_eq!(plan.content_domain(), ContentDomain::VisibleSchemaBindings);
                assert!(
                    plan.artifacts().contains_key(
                        &RelativeOperationPath::parse("candidate/rust_mode.rs")
                            .expect("expected path must parse")
                    )
                );
            }
            OperationKind::GenerateModule => {
                assert_eq!(plan.content_domain(), ContentDomain::ModuleOperation);
                assert!(
                    plan.artifacts().contains_key(
                        &RelativeOperationPath::parse("candidate/src/bin/dagger-module.rs")
                            .expect("expected path must parse")
                    )
                );
                assert!(
                    !plan
                        .artifacts()
                        .keys()
                        .any(|path| { path.as_str().ends_with("operation-manifest.json") })
                );
                let registration = plan
                    .artifacts()
                    .get(
                        &RelativeOperationPath::parse(
                            "candidate/src/dagger_generated/module_registration.rs",
                        )
                        .expect("registration path must parse"),
                    )
                    .expect("module operation must contain active registration");
                let registration = std::str::from_utf8(&registration.content)
                    .expect("registration source must be UTF-8");
                for expected in [
                    "with_cache_policy(dagger_sdk::FunctionCachePolicy::Never)",
                    ".with_check()",
                    ".with_default_path(\".\")",
                    ".with_ignore(vec![\"target\"])",
                ] {
                    assert!(
                        registration.contains(expected),
                        "registration omitted {expected}"
                    );
                }
                assert!(
                    registration.contains(
                        "query.current_function_call().return_value(dagger_sdk::Json::new(encoded))"
                    ),
                    "registration must return the module through the active call"
                );
                assert!(!registration.contains(".serve()"));
                let dispatch = plan
                    .artifacts()
                    .get(
                        &RelativeOperationPath::parse(
                            "candidate/src/dagger_generated/module_dispatch.rs",
                        )
                        .expect("dispatch path must parse"),
                    )
                    .expect("module operation must contain dispatch");
                let dispatch =
                    std::str::from_utf8(&dispatch.content).expect("dispatch source must be UTF-8");
                assert!(
                    dispatch
                        .contains("<dagger_sdk::Directory as super::__private::ModuleInputCodec>"),
                    "dispatch must name Core handles through their SDK path"
                );
                assert!(!dispatch.contains("crate::dagger_generated::Directory"));
                let binary = plan
                    .cargo_binary()
                    .expect("module operation must declare its Cargo target");
                assert_eq!(binary.name, "dagger-module");
                assert_eq!(binary.path.as_str(), "src/bin/dagger-module.rs");
            }
            OperationKind::GenerateEntrypoint => {
                assert_eq!(plan.content_domain(), ContentDomain::ModuleEntrypoint);
                assert_eq!(plan.artifacts().len(), 1);
                let source = &plan
                    .artifacts()
                    .values()
                    .next()
                    .expect("entrypoint artifact must exist")
                    .content;
                let source = std::str::from_utf8(source).expect("entrypoint must be UTF-8");
                assert!(source.contains("Builder::new_current_thread"));
                assert!(source.contains("run_module_entrypoint"));
                assert!(source.contains("GeneratedDispatchRegistry"));
                assert_eq!(
                    plan.cargo_binary()
                        .expect("entrypoint operation must declare its Cargo target")
                        .name,
                    "dagger-module"
                );
            }
        }
    }
}

#[test]
fn source_map_required_argument_compatibility_is_engine_specific() {
    let target = target();
    let dependency = dependency();
    let output = RelativeOperationPath::parse("candidate").expect("fixture path must parse");
    let schema = visible_schema(VisibleSchemaCase::EngineModuleExtension, 0);

    project_operation(OperationProjectionRequest {
        target: &target,
        operation: OperationKind::GenerateLibrary,
        visible_schema_json: &schema,
        module: None,
        output: &output,
        sdk_dependency: &dependency,
        authoring: None,
        engine_version: None,
    })
    .expect("engine-authored module-only source maps must project");

    let mut invalid = Vec::new();
    for mutation in 0..3 {
        let mut document: serde_json::Value =
            serde_json::from_slice(&schema).expect("source-map fixture must decode");
        let extension = document["__schema"]["types"]
            .as_array_mut()
            .expect("fixture types must be an array")
            .iter_mut()
            .find(|definition| definition["name"] == "RustMode")
            .expect("fixture must contain the module extension");
        match mutation {
            0 => extension["directives"][0]["args"] = serde_json::json!([]),
            1 => {
                extension["directives"][0]["args"][0]["value"] = serde_json::Value::Null;
            }
            _ => {
                extension["directives"] = serde_json::json!([{
                    "name": "expectedType",
                    "args": []
                }])
            }
        }
        invalid.push(serde_json::to_vec(&document).expect("source-map mutation must encode"));
    }
    for schema in invalid {
        let diagnostics = project_operation(OperationProjectionRequest {
            target: &target,
            operation: OperationKind::GenerateLibrary,
            visible_schema_json: &schema,
            module: None,
            output: &output,
            sdk_dependency: &dependency,
            authoring: None,
            engine_version: None,
        })
        .expect_err("required directive arguments outside the engine exception must fail");
        assert!(diagnostics.contains(DiagnosticCode::SchemaDirectiveArgumentInvalid));
    }
}

/// Renders a module beside dependency types of every kind that gets a binding (objects,
/// an interface, an enum and an input object), with the module's SDK
/// dependency keyed under `alias`, and returns the dependency bindings by file name.
fn dependency_bindings(alias: &str) -> std::collections::BTreeMap<String, String> {
    let output = RelativeOperationPath::parse("candidate").expect("fixture path must parse");
    let mut authoring =
        module_authoring_input_from(&MODULE_AUTHORING_SOURCE.replace("dagger_sdk", alias));
    authoring.sdk_dependency_alias = alias.to_owned();
    let plan = project_operation(OperationProjectionRequest {
        target: &target(),
        operation: OperationKind::GenerateModule,
        visible_schema_json: &minimal_module_schema(),
        module: Some(&module()),
        output: &output,
        sdk_dependency: &dependency(),
        authoring: Some(&authoring),
        engine_version: None,
    })
    .expect("a module beside dependency types must generate");
    plan.artifacts()
        .iter()
        .filter_map(|(path, artifact)| {
            let name = path
                .as_str()
                .strip_prefix("candidate/src/dagger_generated/minimal")?;
            let source = String::from_utf8(artifact.content.clone()).expect("Rust is UTF-8");
            Some((format!("minimal{name}"), source))
        })
        .collect()
}

// A module may key its SDK dependency under any name, and the rest of its generated
// code reaches the SDK under that name. Dependency bindings that named `dagger_sdk`
// would not compile in such a module.
#[test]
fn dependency_bindings_reach_the_sdk_under_the_module_alias() {
    let default = dependency_bindings("dagger_sdk");
    let renamed = dependency_bindings("renamed_sdk");
    assert_eq!(
        default.keys().collect::<Vec<_>>(),
        [
            "minimal.rs",
            "minimal_client.rs",
            "minimal_config.rs",
            "minimal_item.rs",
            "minimal_node.rs",
            "minimal_state.rs",
        ]
    );
    for expected in [
        "pub struct Minimal { query: dagger_sdk::QueryBuilder }",
        "pub fn from_query(query: dagger_sdk::QueryBuilder) -> Self { Self { query } }",
        "pub fn selection(&self) -> &dagger_sdk::QueryBuilder { &self.query }",
        "pub fn use_item(&self, item: dagger_sdk::Json) -> dagger_sdk::QueryBuilder {",
        "pub config: Option<dagger_sdk::Json>,",
        "opts: &MinimalItemOpts) -> dagger_sdk::QueryBuilder {",
    ] {
        assert!(
            default["minimal.rs"].contains(expected),
            "the default binding lacks `{expected}`"
        );
    }
    assert!(
        default["minimal_node.rs"].contains("fn selection(&self) -> &dagger_sdk::QueryBuilder;")
    );
    assert!(default["minimal_config.rs"].contains("pub enabled: Option<dagger_sdk::Json>,"));

    assert_eq!(default.len(), renamed.len());
    for (name, source) in &renamed {
        assert!(
            !source.contains("dagger_sdk"),
            "{name} names the SDK other than by the module's alias"
        );
        // The alias is the only difference, so the default alias renders what it did
        // before the bindings took one.
        assert_eq!(source.replace("renamed_sdk", "dagger_sdk"), default[name]);
    }
}

#[test]
fn module_visibility_accepts_only_the_target_introspection_scrub() {
    let target = target();
    let module = module();
    let dependency = dependency();
    let output = RelativeOperationPath::parse("candidate").expect("fixture path must parse");
    let schema = module_visible_schema(0);

    let plan = project_operation(OperationProjectionRequest {
        target: &target,
        operation: OperationKind::GenerateModule,
        visible_schema_json: &schema,
        module: Some(&module),
        output: &output,
        sdk_dependency: &dependency,
        authoring: Some(&module_authoring_input()),
        engine_version: None,
    })
    .expect("the exact target module scrub must remain compatible");
    for (path, artifact) in plan.artifacts() {
        if artifact.kind == dagger_codegen::engine::CandidateArtifactKind::RustSource {
            let source = std::str::from_utf8(&artifact.content)
                .unwrap_or_else(|_| panic!("{} must be UTF-8", path.as_str()));
            syn::parse_file(source)
                .unwrap_or_else(|error| panic!("{} must parse: {error}", path.as_str()));
        }
    }
    let directory = tempfile::tempdir().expect("formatter fixture root must be created");
    let mut rust_paths = Vec::new();
    for (path, artifact) in plan.artifacts() {
        if artifact.kind != dagger_codegen::engine::CandidateArtifactKind::RustSource {
            continue;
        }
        let destination = directory.path().join(path.as_str());
        std::fs::create_dir_all(
            destination
                .parent()
                .expect("generated artifact must have a parent"),
        )
        .expect("generated artifact parent must be created");
        std::fs::write(&destination, &artifact.content)
            .expect("generated artifact must be written");
        rust_paths.push(path.as_str().to_owned());
    }
    let formatted = std::process::Command::new("rustfmt")
        .arg("+1.99.1")
        .args(["--edition", "2024"])
        .args(&rust_paths)
        .current_dir(directory.path())
        .output()
        .expect("pinned rustfmt must start");
    assert!(
        formatted.status.success(),
        "module-visible artifacts must format: {}",
        String::from_utf8_lossy(&formatted.stderr)
    );

    let mut unexpected: serde_json::Value =
        serde_json::from_slice(&schema).expect("module-visible fixture must decode");
    let query = unexpected["__schema"]["types"]
        .as_array_mut()
        .expect("fixture types must be an array")
        .iter_mut()
        .find(|definition| definition["name"] == "Query")
        .expect("fixture must contain Query");
    query["fields"]
        .as_array_mut()
        .expect("Query fields must be an array")
        .retain(|field| field["name"] != "address");
    let unexpected = serde_json::to_vec(&unexpected).expect("fixture must encode");
    let diagnostics = project_operation(OperationProjectionRequest {
        target: &target,
        operation: OperationKind::GenerateModule,
        visible_schema_json: &unexpected,
        module: Some(&module),
        output: &output,
        sdk_dependency: &dependency,
        authoring: Some(&module_authoring_input()),
        engine_version: None,
    })
    .expect_err("an unrelated module-schema omission must remain incompatible");
    assert!(diagnostics.contains(DiagnosticCode::SchemaCoreCoordinateMissing));
}

#[test]
fn operation_matrix_rejects_missing_and_forbidden_inputs_before_rendering() {
    let target = target();
    let module = module();
    let dependency = dependency();
    let authoring = module_authoring_input();
    let output = RelativeOperationPath::parse("candidate").expect("fixture path must parse");
    let renderer = ProductionRenderers;

    for (operation, module, authoring, expected) in [
        (
            OperationKind::GenerateModule,
            None,
            None,
            DiagnosticCode::OperationInputMissing,
        ),
        (
            OperationKind::GenerateEntrypoint,
            Some(&module),
            None,
            DiagnosticCode::OperationInputMissing,
        ),
        (
            OperationKind::GenerateLibrary,
            None,
            Some(&authoring),
            DiagnosticCode::OperationInputForbidden,
        ),
    ] {
        let diagnostics = project_operation_with(
            &renderer,
            OperationProjectionRequest {
                target: &target,
                operation,
                visible_schema_json: CORE_SCHEMA_BYTES,
                module,
                output: &output,
                sdk_dependency: &dependency,
                authoring,
                engine_version: None,
            },
        )
        .expect_err("invalid operation matrix entry must fail");
        assert!(diagnostics.contains(expected));
    }
}

#[test]
fn module_authoring_failure_reports_the_authored_diagnostic() {
    let target = target();
    let module = module();
    let dependency = dependency();
    let output = RelativeOperationPath::parse("candidate").expect("fixture path must parse");
    let schema = visible_schema(VisibleSchemaCase::EngineModuleExtension, 0);
    let authoring = module_authoring_input_from(
        r#"
#[dagger_sdk::object(root)]
pub struct FixtureRoot {}

#[dagger_sdk::methods]
impl FixtureRoot {
    #[dagger(constructor)]
    pub fn new() -> FixtureRoot { FixtureRoot {} }

    #[dagger(function)]
    pub fn address(&self) -> std::net::Ipv4Addr { std::net::Ipv4Addr::LOCALHOST }
}
"#,
    );
    let diagnostics = project_operation(OperationProjectionRequest {
        target: &target,
        operation: OperationKind::GenerateModule,
        visible_schema_json: &schema,
        module: Some(&module),
        output: &output,
        sdk_dependency: &dependency,
        authoring: Some(&authoring),
        engine_version: None,
    })
    .expect_err("an unsupported exported type must fail authoring");
    let primary = &diagnostics.diagnostics()[0];
    assert!(
        primary
            .coordinate
            .as_ref()
            .is_some_and(|coordinate| coordinate.as_str().starts_with("src/lib.rs:")),
        "authoring failure must locate the authored source: {diagnostics}"
    );
    assert!(
        primary.message.starts_with("module.foreign-type: "),
        "authoring failure must carry the compiler diagnostic: {diagnostics}"
    );
}

#[test]
fn unknown_operation_selector_is_rejected_as_typed_input() {
    let diagnostics = OperationKind::decode("generate-everything")
        .expect_err("unknown operation selector must fail");
    assert!(diagnostics.contains(DiagnosticCode::OperationUnknown));
}
