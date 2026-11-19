//! Engine-free descriptor, projection, rendering, and regeneration verification.

mod support;

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use dagger_codegen::engine::{
    ModuleAuthoringInput, ModuleProjectionInput, OperationKind, OperationProjectionRequest,
    PublishedSdkDependency, RelativeOperationPath, project_operation,
};
use dagger_codegen::module::{
    CfgEnvironment, FormatVersion, FunctionKind, GeneratedAssetPath, GeneratedTypeBinding,
    GeneratedTypeKind, GeneratedTypeRegistry, ModuleCompilation, ModuleCompilationRequest,
    ModuleCompiler, ModuleDiagnosticCode, ModulePackage, ModuleRenderRequest, ModuleRenderer,
    ModuleSourcePath, ModuleSourceSnapshot, ModuleTarget, PackageName, ProjectionCompiler,
    RegenerationPlanner, Sha256Digest, SourceDocument, TargetValue, WireName, canonical_bytes,
    manifest_digest, source_snapshot_digest,
};
use dagger_codegen::target::CodegenTarget;
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    #[test]
    fn property_11_descriptor_identity_canonical_change_sensitive(
        seed in any::<u16>(),
        reverse in any::<bool>(),
        mutation in 0_u8..5,
    ) {
        let left_source = source_snapshot(seed, false, false);
        let right_source = source_snapshot(seed, reverse, false);
        let target = target(digest(format!("schema-{seed}")));
        let generator = digest(format!("generator-{seed}"));
        let left = compile(&left_source, &target, &generator, BTreeSet::new()).unwrap();
        let right = compile(&right_source, &target, &generator, BTreeSet::new()).unwrap();

        prop_assert_eq!(canonical_bytes(&left.descriptor).unwrap(), canonical_bytes(&right.descriptor).unwrap());
        prop_assert_eq!(&left.descriptor.digest, &right.descriptor.digest);

        let mut changed_source = left_source.clone();
        let mut changed_target = target.clone();
        let mut changed_generator = generator.clone();
        match mutation {
            0 => changed_source = source_snapshot(seed, false, true),
            1 => {
                changed_source.cfg.features.insert("changed".to_owned());
                changed_source.digest = source_snapshot_digest(&changed_source).unwrap();
            }
            2 => changed_target.visible_schema_digest = digest(format!("schema-changed-{seed}")),
            3 => changed_generator = digest(format!("generator-changed-{seed}")),
            _ => changed_target.engine_version = value(format!("engine-changed-{seed}")),
        }
        let changed = compile(
            &changed_source,
            &changed_target,
            &changed_generator,
            BTreeSet::new(),
        ).unwrap();
        prop_assert_ne!(&left.descriptor.digest, &changed.descriptor.digest);
        match mutation {
            0 => prop_assert_ne!(&left.descriptor.provenance.source_files, &changed.descriptor.provenance.source_files),
            1 => prop_assert_ne!(&left.descriptor.provenance.cfg, &changed.descriptor.provenance.cfg),
            2 => prop_assert_ne!(&left.descriptor.provenance.visible_schema_digest, &changed.descriptor.provenance.visible_schema_digest),
            3 => prop_assert_ne!(&left.descriptor.provenance.generator_digest, &changed.descriptor.provenance.generator_digest),
            _ => prop_assert_ne!(&left.descriptor.target, &changed.descriptor.target),
        }
    }

    #[test]
    fn property_12_registration_introspection_equivalent_projections(seed in any::<u16>()) {
        let source = source_snapshot(seed, false, false);
        let target = target(digest(format!("schema-{seed}")));
        let compiled = compile(&source, &target, &digest("generator"), BTreeSet::new()).unwrap();

        prop_assert_eq!(&compiled.registration.types, &compiled.introspection.types);
        prop_assert_eq!(&compiled.registration.descriptor_digest, &compiled.descriptor.digest);
        let query = compiled.registration.types.get(&WireName::new("Query").unwrap()).unwrap();
        prop_assert_eq!(query.functions.len(), 1);
        prop_assert!(query.functions[0].constructor);
        prop_assert_eq!(compiled.registration.types.len(), compiled.descriptor.types.len() + 1);
        let context_path = GeneratedAssetPath::new("src/dagger_generated/module_context.rs").unwrap();
        let context = std::str::from_utf8(&compiled.assets.files[&context_path]).unwrap();
        prop_assert!(context.contains("root: renamed_sdk::Query"));
        prop_assert!(context.contains("generated_query_root"));
        prop_assert!(context.contains("impl ::core::ops::Deref for ModuleQuery"));
        prop_assert!(context.contains("pub fn current_function_call"));
        prop_assert!(context.contains("pub fn current_module"));
        prop_assert!(context.contains("pub fn current_node"));
        for forbidden in ["connect()", "static mut", "Serialize for ModuleContext"] {
            prop_assert!(!context.contains(forbidden));
        }

        let root_name = compiled
            .descriptor
            .types
            .iter()
            .find(|ty| ty.rust_symbol == compiled.descriptor.root)
            .unwrap()
            .wire_name
            .clone();
        let collision = ProjectionCompiler::project(
            &compiled.descriptor,
            &BTreeSet::from([root_name]),
        );
        prop_assert!(collision.is_err());
    }

    #[test]
    fn property_13_dispatch_registry_total_closed_mapping(seed in any::<u16>()) {
        let source = source_snapshot(seed, false, false);
        let target = target(digest(format!("schema-{seed}")));
        let compiled = compile(&source, &target, &digest("generator"), BTreeSet::new()).unwrap();
        let path = GeneratedAssetPath::new("src/dagger_generated/module_dispatch.rs").unwrap();
        let dispatch = String::from_utf8(compiled.assets.files[&path].clone()).unwrap();

        for function in &compiled.descriptor.functions {
            // The engine invokes the constructor with the empty function name.
            let selector = if function.compiled.kind == FunctionKind::Constructor {
                ""
            } else {
                function.wire_name.as_str()
            };
            let arm = format!("{selector:?} => {{");
            prop_assert_eq!(dispatch.matches(&arm).count(), 1);
            let fingerprint = function.fingerprint.as_u128().unwrap();
            let bridge = format!("__dagger_bridge_{}_{}", function.compiled.rust_name, fingerprint);
            prop_assert_eq!(dispatch.matches(&bridge).count(), 1);
        }
        prop_assert!(dispatch.contains("InvocationError::UnknownParent"));
        prop_assert!(dispatch.contains("InvocationError::UnknownFunction"));
        for forbidden in ["downcast", "reflection", "fallback", "inventory::"] {
            prop_assert!(!dispatch.contains(forbidden));
        }

        let duplicate_source = source_snapshot(seed, false, false).with_duplicate_function();
        let duplicate = compile(&duplicate_source, &target, &digest("generator"), BTreeSet::new());
        prop_assert!(duplicate.is_err());
    }

    #[test]
    fn property_27_compile_fixtures_fence_public_authoring_contract(
        seed in any::<u16>(),
        category in 0_u8..7,
    ) {
        let source = compile_contract_source(seed, category);
        let target = target(digest(format!("compile-contract-schema-{seed}")));
        let generator = digest(format!("compile-contract-generator-{seed}"));
        let first = compile(&source, &target, &generator, BTreeSet::new());
        let second = compile(&source, &target, &generator, BTreeSet::new());
        let expected_pass = category == 0;

        prop_assert_eq!(first.is_ok(), expected_pass);
        prop_assert_eq!(second.is_ok(), expected_pass);
        match (first, second) {
            (Ok(left), Ok(right)) => {
                prop_assert_eq!(left.descriptor.digest, right.descriptor.digest);
                prop_assert_eq!(left.assets, right.assets);
            }
            (Err(left), Err(right)) => {
                prop_assert_eq!(&left, &right);
                if category >= 2 {
                    prop_assert!(left.diagnostics().iter().any(|diagnostic| diagnostic.source_coordinate().is_some()));
                }
            }
            _ => prop_assert!(false, "classification changed between identical in-memory compiles"),
        }

        // Every in-memory family is paired with the one bounded trybuild batch rather
        // than spawning Cargo for randomized cases.
        let real_compile_fixture = match category {
            0 => "pass/foundations.rs",
            1 => "fail/duplicate_metadata.rs",
            2 => "fail/private_object.rs",
            3 => "fail/state_signature_mismatch.rs",
            4 => "fail/fingerprint_mismatch.rs",
            5 => "fail/payload_enum.rs",
            _ => "fail/duplicate_metadata.rs",
        };
        let fixture_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("codegen and SDK crates share a parent")
            .join("dagger-sdk/tests/fixtures/module_authoring")
            .join(real_compile_fixture);
        prop_assert!(fixture_root.is_file());
    }

    #[test]
    fn property_25_regeneration_scoped_deterministic_convergent(
        seed in any::<u16>(),
        missing_index in any::<usize>(),
        change in 0_u8..4,
    ) {
        let source = source_snapshot(seed, false, false);
        let baseline_target = target(digest(format!("schema-{seed}")));
        let baseline = compile(&source, &baseline_target, &digest("generator"), BTreeSet::new()).unwrap();
        let baseline_observed = observed(&baseline);
        let no_op = RegenerationPlanner::plan(
            Some(&baseline.assets.manifest),
            &baseline.assets.manifest,
            &baseline_observed,
        ).unwrap();
        prop_assert!(no_op.selected.is_empty());
        prop_assert!(no_op.removed.is_empty());
        prop_assert!(no_op.changed_domains.is_empty());

        let mut inconsistent = baseline.assets.manifest.clone();
        let record = inconsistent.assets.values_mut().next().unwrap();
        record.path = GeneratedAssetPath::new("src/dagger_generated/wrong-owner.rs").unwrap();
        inconsistent.digest = manifest_digest(&inconsistent).unwrap();
        prop_assert!(RegenerationPlanner::plan(
            Some(&inconsistent),
            &baseline.assets.manifest,
            &baseline_observed,
        ).is_err());

        let mut missing = baseline_observed.clone();
        let missing_path = baseline.assets.manifest.assets.keys().nth(
            missing_index % baseline.assets.manifest.assets.len()
        ).unwrap().clone();
        missing.remove(&missing_path);
        let repair = RegenerationPlanner::plan(
            Some(&baseline.assets.manifest),
            &baseline.assets.manifest,
            &missing,
        ).unwrap();
        prop_assert_eq!(repair.selected, BTreeSet::from([missing_path]));

        let obsolete_path = baseline.assets.manifest.assets.keys().next().unwrap().clone();
        let mut pruned = baseline.assets.manifest.clone();
        pruned.assets.remove(&obsolete_path);
        pruned.digest = manifest_digest(&pruned).unwrap();
        let unknown_path = GeneratedAssetPath::new("user-owned/notes.txt").unwrap();
        let mut observed_with_unknown = baseline_observed.clone();
        observed_with_unknown.insert(unknown_path.clone(), digest("unknown-user-bytes"));
        let removal = RegenerationPlanner::plan(
            Some(&baseline.assets.manifest),
            &pruned,
            &observed_with_unknown,
        ).unwrap();
        prop_assert_eq!(removal.removed, BTreeSet::from([obsolete_path]));
        prop_assert!(!removal.selected.contains(&unknown_path));

        let mut changed_source = source.clone();
        let mut changed_target = baseline_target.clone();
        let changed_generator;
        match change {
            0 => {
                changed_source = source_snapshot(seed, false, true);
                changed_generator = digest("generator");
            }
            1 => {
                changed_target.visible_schema_digest = digest(format!("schema-next-{seed}"));
                changed_generator = digest("generator");
            }
            2 => {
                changed_target.engine_version = value(format!("engine-next-{seed}"));
                changed_generator = digest("generator");
            }
            _ => changed_generator = digest(format!("generator-next-{seed}")),
        }
        let changed = compile(
            &changed_source,
            &changed_target,
            &changed_generator,
            BTreeSet::new(),
        ).unwrap();
        let plan = RegenerationPlanner::plan(
            Some(&baseline.assets.manifest),
            &changed.assets.manifest,
            &baseline_observed,
        ).unwrap();
        prop_assert!(!plan.selected.is_empty());
        prop_assert!(plan.selected.iter().all(|path| changed.assets.manifest.assets.contains_key(path)));
        if matches!(change, 2 | 3) {
            prop_assert_eq!(plan.selected.len(), changed.assets.manifest.assets.len());
        }
        let converged = RegenerationPlanner::plan(
            Some(&changed.assets.manifest),
            &changed.assets.manifest,
            &observed(&changed),
        ).unwrap();
        prop_assert_eq!(converged, Default::default());
    }

    #[test]
    fn property_24_pure_generation_rejection_returns_no_partial_compilation(
        seed in any::<u16>(),
        failure in 0_u8..4,
    ) {
        let source = source_snapshot(seed, false, false);
        let target = target(digest(format!("schema-{seed}")));
        let generator = digest(format!("generator-{seed}"));
        let baseline = compile(&source, &target, &generator, BTreeSet::new()).unwrap();

        let rejected = match failure {
            0 => {
                let stale = GeneratedTypeRegistry::empty(digest(format!("stale-schema-{seed}")));
                ModuleCompiler::compile(ModuleCompilationRequest {
                    target: &target,
                    source: &source,
                    generated_types: &stale,
                    visible_type_names: &BTreeSet::new(),
                    generator_digest: &generator,
                    sdk_dependency_alias: "renamed_sdk",
                    checked_bindings: &BTreeMap::new(),
                }).map(|compilation| compilation.assets)
            }
            1 => compile(
                &source.clone().with_duplicate_function(),
                &target,
                &generator,
                BTreeSet::new(),
            ).map(|compilation| compilation.assets),
            2 => {
                let root_name = baseline
                    .descriptor
                    .types
                    .iter()
                    .find(|ty| ty.rust_symbol == baseline.descriptor.root)
                    .unwrap()
                    .wire_name
                    .clone();
                compile(
                    &source,
                    &target,
                    &generator,
                    BTreeSet::from([root_name]),
                ).map(|compilation| compilation.assets)
            }
            _ => {
                let mut introspection = baseline.introspection.clone();
                introspection.descriptor_digest = digest(format!("wrong-descriptor-{seed}"));
                ModuleRenderer::render(ModuleRenderRequest {
                    descriptor: &baseline.descriptor,
                    registration: &baseline.registration,
                    introspection: &introspection,
                    sdk_dependency_alias: "renamed_sdk",
                    checked_bindings: &BTreeMap::new(),
                    generated_types: &BTreeMap::new(),
                })
            }
        };
        prop_assert!(rejected.is_err());
    }
}

// The module is generated as `dagger generate` generates it, against a schema that
// holds module dependencies, so the bindings rendered for their types are built, tested
// and linted with the module's own generated code. The dependencies carry the names
// most likely to break generated Rust: arguments called `query` and `opts`, functions
// called `clone` and `new`, types called `Result`, `Option`, `Box` or what the module
// calls the SDK, and documentation that rustdoc would take for Rust.
#[test]
fn representative_generated_module_executes_directly_offline() {
    let source = representative_source();
    let generated = generate_representative(&source);
    for dependency in [
        "greeter.rs",
        "greeter_named.rs",
        "greeter_mode.rs",
        "vault.rs",
        "result.rs",
        "into_iterator.rs",
        "sync.rs",
        "renamed_sdk.rs",
    ] {
        assert!(
            generated.contains_key(&format!("src/dagger_generated/{dependency}")),
            "{dependency} must be generated"
        );
    }
    for (path, bytes) in &generated {
        if path.ends_with(".rs") {
            syn::parse_file(std::str::from_utf8(bytes).expect("generated Rust is UTF-8"))
                .unwrap_or_else(|error| panic!("{path} must parse: {error}"));
        }
    }

    let temporary = tempfile::tempdir().expect("fixture root must be available");
    let root = temporary.path();
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let sdk = manifest_dir
        .parent()
        .expect("codegen and SDK crates share a parent")
        .join("dagger-sdk");
    fs::create_dir_all(root.join("src")).expect("fixture source root must be created");
    fs::write(
        root.join("Cargo.toml"),
        format!(
            "[package]\nname = \"fixture\"\nversion = \"0.0.0\"\nedition = \"2024\"\n\n[dependencies]\nrenamed_sdk = {{ package = \"tokeira-dagger-sdk\", path = {:?} }}\ntokio = {{ version = \"1\", features = [\"rt\"] }}\nasync-trait = \"0.1\"\n",
            sdk
        ),
    )
    .expect("fixture manifest must write");
    // The workspace lockfile pins the SDK's dependency versions, so offline resolution does
    // not depend on what the registry has yanked since.
    let workspace = manifest_dir
        .parent()
        .and_then(Path::parent)
        .expect("codegen sits two levels below the workspace root");
    fs::copy(workspace.join("Cargo.lock"), root.join("Cargo.lock"))
        .expect("workspace lockfile must copy");
    for document in source.documents.values() {
        let destination = root.join(document.path.as_str());
        fs::create_dir_all(destination.parent().expect("document has a parent"))
            .expect("document parent must be created");
        fs::write(destination, &document.contents).expect("fixture source must write");
    }
    for (path, bytes) in &generated {
        if !path.ends_with(".rs") {
            continue;
        }
        let destination = root.join(path);
        fs::create_dir_all(destination.parent().expect("asset has a parent"))
            .expect("asset parent must be created");
        fs::write(destination, bytes).expect("generated source must write");
    }
    fs::create_dir_all(root.join("tests")).expect("fixture test root must be created");
    fs::write(
        root.join("tests/direct_harness.rs"),
        direct_harness_source(),
    )
    .expect("direct harness must write");
    let target_dir = temporary.path().join("target");
    let output = Command::new(env!("CARGO"))
        .args(["test", "--all-targets", "--offline", "--quiet"])
        .env("CARGO_TARGET_DIR", &target_dir)
        .current_dir(root)
        .output()
        .expect("offline fixture cargo must start");
    // A failed assertion of the harness is on its standard output.
    assert!(
        output.status.success(),
        "generated fixture failed:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    // `--all-targets` leaves doctests out, and a dependency's documentation must not
    // become one.
    let doctests = Command::new(env!("CARGO"))
        .args(["test", "--doc", "--offline", "--quiet"])
        .env("CARGO_TARGET_DIR", &target_dir)
        .current_dir(root)
        .output()
        .expect("offline fixture doctests must start");
    assert!(
        doctests.status.success(),
        "generated fixture has failing doctests:\n{}\n{}",
        String::from_utf8_lossy(&doctests.stdout),
        String::from_utf8_lossy(&doctests.stderr)
    );
    // A module that compiles generated code must not need lint exceptions of its own
    // for it.
    let lints = Command::new(env!("CARGO"))
        .args([
            "clippy",
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
        "generated fixture has lints:\n{}",
        String::from_utf8_lossy(&lints.stderr)
    );
}

#[test]
fn an_agent_function_registers_with_the_agent_role() {
    let registration = registration_source(&compile_representative(&representative_source()));
    assert!(registration.contains("function_model = function_model.with_agent();"));
}

#[test]
fn a_collection_registers_the_members_its_author_marked() {
    let registration = registration_source(&compile_representative(&representative_source()));
    for expected in [
        "type_fixtureitems = type_fixtureitems.with_collection();",
        "type_fixtureitems = type_fixtureitems.with_collection_keys(\"names\");",
        "type_fixtureitems = type_fixtureitems.with_collection_delta(\"selection\");",
        "type_fixtureitems = type_fixtureitems.with_collection_get(\"lookup\");",
    ] {
        assert!(
            registration.contains(expected),
            "registration lacks {expected}"
        );
    }
    // Only the collection is one, and its base is no part of its API.
    assert_eq!(registration.matches(".with_collection()").count(), 1);
    assert!(!registration.contains("\"base\""));
}

#[test]
fn collection_markers_out_of_place_are_refused_before_generation() {
    let target = target(digest("checked-schema"));
    let generator = digest("checked-generator");
    let object = |outer: &str, fields: &str| {
        format!(
            "#[renamed_sdk::object({outer}rename = \"FixtureBag\")]\npub(crate) struct Bag {{\n{fields}}}\n"
        )
    };
    let base = "    base: renamed_sdk::CollectionBase,\n";
    let keys = "    #[dagger(field, keys)]\n    names: Vec<String>,\n";
    let get = |marker: &str, name: &str| {
        format!(
            "#[renamed_sdk::methods]\nimpl Bag {{\n    #[dagger({marker})]\n    pub(crate) fn {name}(&self, name: String) -> String {{ name }}\n}}\n"
        )
    };
    for (declaration, code, message) in [
        (
            object("", &format!("{keys}{base}")),
            ModuleDiagnosticCode::StateShapeInvalid,
            "a `CollectionBase` field belongs only in a collection",
        ),
        (
            object("collection, ", keys),
            ModuleDiagnosticCode::StateShapeInvalid,
            "a collection declares one `CollectionBase` field, which carries the collection it was derived from",
        ),
        (
            object(
                "collection, ",
                &format!("{keys}{base}    other: renamed_sdk::CollectionBase,\n"),
            ),
            ModuleDiagnosticCode::StateShapeInvalid,
            "a collection declares exactly one `CollectionBase` field",
        ),
        (
            object(
                "collection, ",
                &format!("    #[dagger(keys)]\n    names: Vec<String>,\n{base}"),
            ),
            ModuleDiagnosticCode::MetadataConflict,
            "`keys` and `delta` mark a collection's `#[dagger(field)]`",
        ),
        (
            object("", keys),
            ModuleDiagnosticCode::MetadataConflict,
            "`keys`, `get` and `delta` mark members of a collection",
        ),
        (
            object(
                "collection, ",
                &format!("{keys}    #[dagger(field, keys)]\n    others: Vec<String>,\n{base}"),
            ),
            ModuleDiagnosticCode::MetadataConflict,
            "a collection marks one keys field and one delta field",
        ),
        (
            format!(
                "{}{}",
                object("", keys.replace(", keys", "").as_str()),
                get("function, get", "lookup")
            ),
            ModuleDiagnosticCode::MetadataConflict,
            "`keys`, `get` and `delta` mark members of a collection",
        ),
        (
            format!(
                "{}{}{}",
                object("collection, ", &format!("{keys}{base}")),
                get("function, get", "lookup"),
                get("function, get", "find"),
            ),
            ModuleDiagnosticCode::MetadataConflict,
            "a collection marks one get function",
        ),
        (
            format!(
                "{}{}",
                object("collection, ", &format!("{keys}{base}")),
                get("get", "lookup"),
            ),
            ModuleDiagnosticCode::MetadataConflict,
            "`get` marks a collection's `#[dagger(function)]`",
        ),
    ] {
        let source = with_contents(source_snapshot(3, false, false), "src/lib.rs", |contents| {
            format!("{contents}\n{declaration}")
        });
        let diagnostics = compile(&source, &target, &generator, BTreeSet::new())
            .err()
            .unwrap_or_else(|| panic!("generation must refuse:\n{declaration}"));
        // A refused declaration also hides the root declared beside it, so the refusal
        // can name more than this one diagnostic.
        let found = diagnostics
            .diagnostics()
            .iter()
            .map(|diagnostic| (diagnostic.code(), diagnostic.message()))
            .collect::<Vec<_>>();
        assert!(
            found.contains(&(code, message)),
            "{declaration}\nrefused with {found:?}"
        );
    }
}

#[test]
fn core_enums_and_scalars_register_with_their_kinds_and_defaults() {
    let compiled = compile_representative(&representative_source());
    let registration = registration_source(&compiled);
    for expected in [
        "query.type_def().with_enum(\"NetworkProtocol\")",
        "query.type_def().with_enum(\"ImageLayerCompression\")",
        "query.type_def().with_scalar(\"Platform\")",
        "query.type_def().with_scalar(\"JSON\")",
        "query.type_def().with_scalar(\"Bytes\")",
        // Only a field names `FileType`.
        "query.type_def().with_enum(\"FileType\")",
        // An enum default is its member's wire value; a scalar's is its exact text.
        ".with_default_value(renamed_sdk::Json::new(\"\\\"UDP\\\"\"))",
        ".with_default_value(renamed_sdk::Json::new(\"\\\"linux/arm64\\\"\"))",
        ".with_default_value(renamed_sdk::Json::new(\"\\\"{\\\\\\\"a\\\\\\\":1}\\\"\"))",
    ] {
        assert!(
            registration.contains(expected),
            "registration lacks {expected}"
        );
    }
    // Registration declares Core types by name and never defines them.
    assert!(!registration.contains("with_enum_opts(\"NetworkProtocol\""));
    assert!(!registration.contains("module = module.with_enum(type_networkprotocol)"));
}

#[test]
fn a_core_type_that_only_a_field_or_an_interface_method_names_registers() {
    // Registration declares a Core type with the kind of its binding, wherever the
    // module names it.
    for declaration in [
        "#[renamed_sdk::object(rename = \"FixtureTree\")]\npub(crate) struct Tree {\n    #[dagger(field)]\n    tree: renamed_sdk::Directory,\n}\n",
        "#[renamed_sdk::interface]\npub(crate) trait Probe {\n    fn tree(&self) -> renamed_sdk::Directory;\n}\n",
    ] {
        let source = with_contents(source_snapshot(3, false, false), "src/lib.rs", |contents| {
            format!("{contents}\n{declaration}")
        });
        let registration = registration_source(&compile_representative(&source));
        assert!(
            registration.contains("query.type_def().with_object(\"Directory\")"),
            "{declaration}"
        );
    }
}

#[test]
fn crate_root_documentation_is_the_module_description() {
    let target = target(digest("checked-schema"));
    let generator = digest("checked-generator");
    // The root object's documentation describes the object, not the module.
    let documented_root = |snapshot: ModuleSourceSnapshot| {
        with_contents(snapshot, "src/lib.rs", |contents| {
            contents.replace(
                "#[renamed_sdk::object(root",
                "/// The root object.\n#[renamed_sdk::object(root",
            )
        })
    };
    let undocumented = compile(
        &documented_root(source_snapshot(3, false, false)),
        &target,
        &generator,
        BTreeSet::new(),
    )
    .expect("a module without crate documentation compiles");
    assert_eq!(undocumented.descriptor.documentation, None);
    assert_eq!(undocumented.registration.documentation, None);
    assert!(!registration_source(&undocumented).contains("module = module.with_description("));

    // Only the crate root's inner documentation counts; a child module's describes
    // that module.
    let documented = with_contents(
        with_contents(
            documented_root(source_snapshot(3, false, false)),
            "src/lib.rs",
            |contents| format!("//! Greets a \"child\".\n//!\n//! Longer \\ text.\n{contents}"),
        ),
        "src/child.rs",
        |contents| format!("//! The child module.\n{contents}"),
    );
    let compiled = compile(&documented, &target, &generator, BTreeSet::new())
        .expect("a module with crate documentation compiles");
    let expected = "Greets a \"child\".\n\nLonger \\ text.";
    assert_eq!(compiled.descriptor.documentation.as_deref(), Some(expected));
    assert_eq!(
        compiled.registration.documentation.as_deref(),
        Some(expected)
    );
    assert_eq!(
        compiled.introspection.documentation.as_deref(),
        Some(expected)
    );
    let root = compiled
        .registration
        .types
        .values()
        .find(|ty| ty.wire_name.as_str() == "FixtureRoot")
        .expect("the root object is registered");
    assert_eq!(root.documentation.as_deref(), Some("The root object."));
    assert!(
        registration_source(&compiled)
            .contains(&format!("module = module.with_description({expected:?});"))
    );
    assert_ne!(compiled.descriptor.digest, undocumented.descriptor.digest);
}

fn registration_source(compilation: &ModuleCompilation) -> String {
    let path = GeneratedAssetPath::new("src/dagger_generated/module_registration.rs")
        .expect("the registration path is valid");
    String::from_utf8(compilation.assets.files[&path].clone()).expect("generated Rust is UTF-8")
}

fn with_contents(
    mut snapshot: ModuleSourceSnapshot,
    path: &str,
    change: impl FnOnce(&str) -> String,
) -> ModuleSourceSnapshot {
    let path = ModuleSourcePath::new(path).expect("fixture path is valid");
    let document = snapshot
        .documents
        .get_mut(&path)
        .expect("fixture document exists");
    document.contents = change(&document.contents);
    document.digest = Sha256Digest::hash_bytes(document.contents.as_bytes());
    snapshot.digest = source_snapshot_digest(&snapshot).expect("fixture snapshot must hash");
    snapshot
}

fn direct_harness_source() -> &'static str {
    r#"
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use fixture::dagger_generated::GeneratedDispatchRegistry;
use renamed_sdk::{
    Client, ClientConfig, CurrentCall, EngineConnection, EngineConnectionError, ModuleCancellation,
    RawRequest, RawResponse, ResponseData,
};
use renamed_sdk::__private::{
    CallEnvelope, CallIdentity, CallOutcome, CallSelector, ErrorOutcomeKind, ModuleBoxFuture,
    ModuleContextBase, ModuleFunctionName, ModuleJson, ModuleWireName, NamedModuleArgument,
    RegistrationError, RegistrationSink,
    RegistrationView, ResultPublishError, ResultSink, handle_call, serde_json,
};

// Stands in for the engine: it records every document it is sent and answers the
// ones a call to a module dependency sends, as the dependency would.
#[derive(Clone)]
struct NoEngine(Arc<Mutex<Vec<String>>>);

const DEPENDENCY_DOCUMENTS: &[(&str, &str)] = &[
    (
        "query{greeter(greeting:\"hi\", mode:LOUD){hello(name:\"friend\")}}",
        "{\"greeter\": {\"hello\": \"HI, FRIEND\"}}",
    ),
    (
        "query{node(id:\"passed-id\"){... on Greeter{id}}}",
        "{\"node\": {\"id\": \"greeter-id\"}}",
    ),
    (
        "query{node(id:\"passed-id\"){... on Greeter{hello(name:\"lazy\")}}}",
        "{\"node\": {\"hello\": \"hello, lazy\"}}",
    ),
    (
        "query{node(id:\"passed-id\"){... on Directory{id}}}",
        "{\"node\": {\"id\": \"directory-id\"}}",
    ),
    (
        "query{greeter{id}}",
        "{\"greeter\": {\"id\": \"state-id\"}}",
    ),
    (
        "query{node(id:\"state-id\"){... on Greeter{withMode(mode:QUIET){hello(name:\"again\")}}}}",
        "{\"node\": {\"withMode\": {\"hello\": \"hello, again\"}}}",
    ),
];

#[async_trait]
impl EngineConnection for NoEngine {
    async fn execute(&self, request: RawRequest) -> Result<RawResponse, EngineConnectionError> {
        self.0.lock().expect("event lock").push(request.query().to_owned());
        let data = DEPENDENCY_DOCUMENTS
            .iter()
            .find(|(document, _)| *document == request.query())
            .map_or(ResponseData::Null, |(_, data)| {
                ResponseData::Value(serde_json::from_str(data).expect("fixture answer is JSON"))
            });
        Ok(RawResponse::new(data))
    }

    async fn close(&self) -> Result<(), EngineConnectionError> { Ok(()) }

    fn abort(&self) {}
}

struct RegistrationRecorder(Arc<Mutex<Vec<String>>>);

impl RegistrationSink for RegistrationRecorder {
    fn serve<'a>(
        &'a self,
        registration: &'a RegistrationView,
    ) -> ModuleBoxFuture<'a, Result<(), RegistrationError>> {
        Box::pin(async move {
            self.0
                .lock()
                .expect("registration lock")
                .push(registration.descriptor_digest.to_owned());
            Ok(())
        })
    }
}

struct ResultRecorder(Arc<Mutex<Vec<CallOutcome>>>);

impl ResultSink for ResultRecorder {
    fn publish<'a>(
        &'a self,
        outcome: CallOutcome,
    ) -> ModuleBoxFuture<'a, Result<(), ResultPublishError>> {
        Box::pin(async move {
            self.0.lock().expect("result lock").push(outcome);
            Ok(())
        })
    }
}

fn envelope(
    call_id: &str,
    function: &str,
    parent: Option<serde_json::Value>,
    arguments: &[(&str, serde_json::Value)],
) -> CallEnvelope {
    envelope_on("FixtureRoot", call_id, function, parent, arguments)
}

fn envelope_on(
    parent_type: &str,
    call_id: &str,
    function: &str,
    parent: Option<serde_json::Value>,
    arguments: &[(&str, serde_json::Value)],
) -> CallEnvelope {
    let selector = CallSelector::Invocation {
        parent_wire_name: ModuleWireName::new(parent_type).expect("parent name"),
        function_wire_name: ModuleFunctionName::new(function).expect("function name"),
    };
    CallEnvelope::new(
        CallIdentity::new(call_id, selector).expect("call identity"),
        parent.map(ModuleJson::new),
        arguments
            .iter()
            .map(|(name, value)| NamedModuleArgument {
                name: ModuleWireName::new(*name).expect("argument name"),
                value: ModuleJson::new(value.clone()),
            })
            .collect(),
    )
}

fn context(client: &Client, envelope: &CallEnvelope) -> ModuleContextBase {
    ModuleContextBase::new(
        client.query_builder(),
        ModuleCancellation::default(),
        Default::default(),
        CurrentCall::new(
            envelope.identity.call_id(),
            envelope.identity.selector().clone(),
        )
        .expect("current call"),
    )
}

#[test]
fn generated_registry_runs_through_the_production_wrapper_without_an_engine() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("runtime");
    runtime.block_on(async {
        let engine_events = Arc::new(Mutex::new(Vec::new()));
        let client = renamed_sdk::connect_with(
            ClientConfig::builder()
                .connection(Box::new(NoEngine(Arc::clone(&engine_events))))
                .build()
                .expect("client config"),
        )
        .await
        .expect("injected client");
        let registrations = Arc::new(Mutex::new(Vec::new()));
        let outcomes = Arc::new(Mutex::new(Vec::new()));
        let registration_sink = RegistrationRecorder(Arc::clone(&registrations));
        let result_sink = ResultRecorder(Arc::clone(&outcomes));

        let registration = CallEnvelope::new(
            CallIdentity::new("registration", CallSelector::Registration).expect("identity"),
            None,
            Vec::new(),
        );
        handle_call(
            &GeneratedDispatchRegistry,
            registration.clone(),
            context(&client, &registration),
            &registration_sink,
            &result_sink,
        )
        .await
        .expect("registration");

        let constructor = envelope(
            "constructor",
            "",
            None,
            &[("child", serde_json::json!({"value": "hello"}))],
        );
        handle_call(
            &GeneratedDispatchRegistry,
            constructor.clone(),
            context(&client, &constructor),
            &registration_sink,
            &result_sink,
        )
        .await
        .expect("constructor");
        let parent = match outcomes.lock().expect("result lock").pop().expect("state") {
            CallOutcome::Value(value) => value.into_json(),
            outcome => panic!("unexpected constructor outcome: {outcome:?}"),
        };

        for (call_id, arguments, expected) in [
            ("default", Vec::new(), "hello-world-7-stable"),
            (
                "explicit",
                vec![("name", serde_json::json!("friend"))],
                "hello-friend-7-stable",
            ),
        ] {
            let invocation = envelope(call_id, "greet", Some(parent.clone()), &arguments);
            handle_call(
                &GeneratedDispatchRegistry,
                invocation.clone(),
                context(&client, &invocation),
                &registration_sink,
                &result_sink,
            )
            .await
            .expect("invocation");
            let value = match outcomes.lock().expect("result lock").pop().expect("value") {
                CallOutcome::Value(value) => value.into_json(),
                outcome => panic!("unexpected invocation outcome: {outcome:?}"),
            };
            assert_eq!(value, serde_json::json!(expected));
        }

        // A Core enum or scalar crosses as one string, and an absent argument takes its
        // default.
        for (call_id, function, arguments, expected) in [
            ("protocol-default", "protocol", Vec::new(), serde_json::json!("UDP")),
            (
                "protocol",
                "protocol",
                vec![("protocol", serde_json::json!("TCP"))],
                serde_json::json!("TCP"),
            ),
            (
                "compression",
                "compression",
                vec![("compression", serde_json::json!("GZIP"))],
                serde_json::json!("GZIP"),
            ),
            ("platform-default", "platform", Vec::new(), serde_json::json!("linux/arm64")),
            ("labels-default", "labels", Vec::new(), serde_json::json!("{\"a\":1}")),
            (
                "labels",
                "labels",
                vec![("labels", serde_json::json!("[true]"))],
                serde_json::json!("[true]"),
            ),
            (
                "data",
                "data",
                vec![("data", serde_json::json!("aGk="))],
                serde_json::json!(["aGk="]),
            ),
            ("entry", "entry", Vec::new(), serde_json::json!({"kind": "REGULAR"})),
            // A fresh collection is its own base, so its JSON names none.
            (
                "collection",
                "items",
                Vec::new(),
                serde_json::json!({"names": ["a", "b"], "selection": null, "prefix": ""}),
            ),
        ] {
            let invocation = envelope(call_id, function, Some(parent.clone()), &arguments);
            handle_call(
                &GeneratedDispatchRegistry,
                invocation.clone(),
                context(&client, &invocation),
                &registration_sink,
                &result_sink,
            )
            .await
            .expect("Core value invocation");
            let value = match outcomes.lock().expect("result lock").pop().expect("value") {
                CallOutcome::Value(value) => value.into_json(),
                outcome => panic!("unexpected {call_id} outcome: {outcome:?}"),
            };
            assert_eq!(value, expected, "{call_id}");
        }

        // The engine hands a collection the collection it was derived from. A value
        // built from the receiver returns that base, and a fresh value returns none.
        let items = serde_json::json!({
            "names": ["a", "b"],
            "selection": null,
            "prefix": "p-",
            "__daggerCollectionBase": "base-id",
        });
        for (call_id, function, arguments, expected) in [
            (
                "collection-get",
                "lookup",
                vec![("name", serde_json::json!("a"))],
                serde_json::json!({"name": "p-a"}),
            ),
            (
                "collection-derived",
                "withPrefix",
                vec![("prefix", serde_json::json!("q-"))],
                serde_json::json!({
                    "names": ["a", "b"],
                    "selection": null,
                    "prefix": "q-",
                    "__daggerCollectionBase": "base-id",
                }),
            ),
            (
                "collection-fresh",
                "fresh",
                Vec::new(),
                serde_json::json!({"names": ["z"], "selection": null, "prefix": "p-"}),
            ),
        ] {
            let invocation =
                envelope_on("FixtureItems", call_id, function, Some(items.clone()), &arguments);
            handle_call(
                &GeneratedDispatchRegistry,
                invocation.clone(),
                context(&client, &invocation),
                &registration_sink,
                &result_sink,
            )
            .await
            .expect("collection invocation");
            let value = match outcomes.lock().expect("result lock").pop().expect("value") {
                CallOutcome::Value(value) => value.into_json(),
                outcome => panic!("unexpected {call_id} outcome: {outcome:?}"),
            };
            assert_eq!(value, expected, "{call_id}");
        }

        // A call to a module dependency sends the dependency's own fields on the call's
        // session: the constructor with its options, then the function. A handle made
        // from an ID the caller passes loads through `node(id:)`, and only a checked
        // load, which the engine answers with the object's ID, keeps one.
        for (call_id, function, arguments, expected) in [
            (
                "dependency-call",
                "hail",
                vec![("name", serde_json::json!("friend"))],
                serde_json::json!("HI, FRIEND"),
            ),
            (
                "dependency-load",
                "reload",
                vec![("id", serde_json::json!("passed-id"))],
                serde_json::json!(["hello, lazy", "greeter-id", "directory-id"]),
            ),
        ] {
            let invocation = envelope(call_id, function, Some(parent.clone()), &arguments);
            handle_call(
                &GeneratedDispatchRegistry,
                invocation.clone(),
                context(&client, &invocation),
                &registration_sink,
                &result_sink,
            )
            .await
            .expect("dependency invocation");
            let value = match outcomes.lock().expect("result lock").pop().expect("value") {
                CallOutcome::Value(value) => value.into_json(),
                outcome => panic!("unexpected {call_id} outcome: {outcome:?}"),
            };
            assert_eq!(value, expected, "{call_id}");
        }

        // An object keeps a dependency's object and enum in private state. Encoding the
        // state resolves the handle's ID with one request, and the next call loads the
        // handle from that ID without keeping it, then calls the dependency through it.
        let welcome = envelope("dependency-state", "welcome", Some(parent.clone()), &[]);
        handle_call(
            &GeneratedDispatchRegistry,
            welcome.clone(),
            context(&client, &welcome),
            &registration_sink,
            &result_sink,
        )
        .await
        .expect("state invocation");
        let state = match outcomes.lock().expect("result lock").pop().expect("state") {
            CallOutcome::Value(value) => value.into_json(),
            outcome => panic!("unexpected state outcome: {outcome:?}"),
        };
        assert_eq!(
            state,
            serde_json::json!({"greeter": "state-id", "mode": "QUIET", "previous": null, "outcome": null})
        );
        let greet = envelope_on(
            "FixtureWelcome",
            "dependency-state-call",
            "greet",
            Some(state),
            &[("name", serde_json::json!("again"))],
        );
        handle_call(
            &GeneratedDispatchRegistry,
            greet.clone(),
            context(&client, &greet),
            &registration_sink,
            &result_sink,
        )
        .await
        .expect("state call");
        match outcomes.lock().expect("result lock").pop().expect("value") {
            CallOutcome::Value(value) => {
                assert_eq!(value.into_json(), serde_json::json!("hello, again"));
            }
            outcome => panic!("unexpected state call outcome: {outcome:?}"),
        }

        // A value outside a dependency's enum is refused as one outside Core's is, and
        // a handle's place in state accepts nothing but an ID.
        for state in [
            serde_json::json!({"greeter": "state-id", "mode": "SHOUTED", "previous": null, "outcome": null}),
            serde_json::json!({"greeter": {"hello": "state-id"}, "mode": "QUIET", "previous": null, "outcome": null}),
            serde_json::json!({"greeter": "", "mode": "QUIET", "previous": null, "outcome": null}),
        ] {
            let invalid = envelope_on(
                "FixtureWelcome",
                "dependency-state-invalid",
                "greet",
                Some(state),
                &[("name", serde_json::json!("nobody"))],
            );
            let refused = handle_call(
                &GeneratedDispatchRegistry,
                invalid.clone(),
                context(&client, &invalid),
                &registration_sink,
                &result_sink,
            )
            .await;
            let published = outcomes.lock().expect("result lock").pop();
            assert!(
                refused.is_err() || matches!(published, Some(CallOutcome::Error { .. })),
                "invalid dependency state must not reach user code: {published:?}"
            );
        }

        // A value outside the enum is refused before user code runs.
        let outside = envelope(
            "protocol-outside",
            "protocol",
            Some(parent.clone()),
            &[("protocol", serde_json::json!("SCTP"))],
        );
        let refused = handle_call(
            &GeneratedDispatchRegistry,
            outside.clone(),
            context(&client, &outside),
            &registration_sink,
            &result_sink,
        )
        .await;
        let published = outcomes.lock().expect("result lock").pop();
        assert!(
            refused.is_err() || matches!(published, Some(CallOutcome::Error { .. })),
            "an unknown enum member must not reach user code: {published:?}"
        );

        let refusal = envelope("refusal", "refuse", Some(parent.clone()), &[]);
        handle_call(
            &GeneratedDispatchRegistry,
            refusal.clone(),
            context(&client, &refusal),
            &registration_sink,
            &result_sink,
        )
        .await
        .expect("fallible invocation");
        match outcomes.lock().expect("result lock").pop().expect("error") {
            CallOutcome::Error { error, kind: ErrorOutcomeKind::Application } => {
                assert_eq!(error.message(), "refused");
            }
            outcome => panic!("unexpected fallible outcome: {outcome:?}"),
        }

        let unknown = envelope("unknown", "absent", Some(parent), &[]);
        assert!(
            handle_call(
                &GeneratedDispatchRegistry,
                unknown.clone(),
                context(&client, &unknown),
                &registration_sink,
                &result_sink,
            )
            .await
            .is_err()
        );
        assert_eq!(registrations.lock().expect("registration lock").len(), 1);
        assert!(outcomes.lock().expect("result lock").is_empty());
        // Nothing but the dependency calls reached the engine, each as one request.
        assert_eq!(
            engine_events.lock().expect("event lock").as_slice(),
            DEPENDENCY_DOCUMENTS
                .iter()
                .map(|(document, _)| (*document).to_owned())
                .collect::<Vec<_>>()
        );
        client.close().await.expect("fixture client close");
    });
}
"#
}

// The representative module names Core objects, enums, and scalars. `LLM` is `Llm` in
// Rust, so the generated dispatch must name each Core type through its binding path
// rather than its target name. Its private state also names an object and an enum of a
// module it depends on, which are bound in the module's own crate.
fn compile_representative(source: &ModuleSourceSnapshot) -> ModuleCompilation {
    let target = target(digest("checked-schema"));
    let binding =
        |name: &str, target_name: &str, kind, members: &[(&str, &str)]| GeneratedTypeBinding {
            rust_path: format!("renamed_sdk::{name}"),
            target_name: target_name.to_owned(),
            visible_schema_digest: target.visible_schema_digest.clone(),
            kind,
            enum_members: members
                .iter()
                .map(|(variant, member)| ((*variant).to_owned(), (*member).to_owned()))
                .collect(),
        };
    let generated = GeneratedTypeRegistry::new(
        target.visible_schema_digest.clone(),
        [
            binding("Directory", "Directory", GeneratedTypeKind::Object, &[]),
            binding("Llm", "LLM", GeneratedTypeKind::Object, &[]),
            binding(
                "CollectionDelta",
                "CollectionDelta",
                GeneratedTypeKind::Object,
                &[],
            ),
            binding(
                "NetworkProtocol",
                "NetworkProtocol",
                GeneratedTypeKind::Enum,
                &[("Tcp", "TCP"), ("Udp", "UDP")],
            ),
            binding(
                "ImageLayerCompression",
                "ImageLayerCompression",
                GeneratedTypeKind::Enum,
                &[
                    ("Estargz", "ESTARGZ"),
                    ("Gzip", "GZIP"),
                    ("Uncompressed", "UNCOMPRESSED"),
                    ("Zstd", "ZSTD"),
                ],
            ),
            binding(
                "FileType",
                "FileType",
                GeneratedTypeKind::Enum,
                &[
                    ("Directory", "DIRECTORY"),
                    ("Regular", "REGULAR"),
                    ("Symlink", "SYMLINK"),
                ],
            ),
            binding("Platform", "Platform", GeneratedTypeKind::Scalar, &[]),
            binding("Json", "JSON", GeneratedTypeKind::Scalar, &[]),
            binding("Bytes", "Bytes", GeneratedTypeKind::Scalar, &[]),
            GeneratedTypeBinding {
                rust_path: "crate::dagger_generated::Greeter".to_owned(),
                ..binding("Greeter", "Greeter", GeneratedTypeKind::Object, &[])
            },
            GeneratedTypeBinding {
                rust_path: "crate::dagger_generated::Result".to_owned(),
                ..binding("Result", "Result", GeneratedTypeKind::Object, &[])
            },
            GeneratedTypeBinding {
                rust_path: "crate::dagger_generated::GreeterMode".to_owned(),
                ..binding(
                    "GreeterMode",
                    "GreeterMode",
                    GeneratedTypeKind::Enum,
                    &[("Loud", "LOUD"), ("Quiet", "QUIET")],
                )
            },
        ],
    )
    .expect("fixture bindings are unique");
    ModuleCompiler::compile(ModuleCompilationRequest {
        target: &target,
        source,
        generated_types: &generated,
        visible_type_names: &BTreeSet::new(),
        generator_digest: &digest("checked-generator"),
        sdk_dependency_alias: "renamed_sdk",
        checked_bindings: &BTreeMap::new(),
    })
    .expect("representative module must compile purely")
}

// The whole operation `dagger generate` runs for a module: the visible schema is
// checked and projected, the bindings of the dependencies' types are rendered, and the
// authored source is compiled against the registry of both. The files are keyed by
// their path in the module.
fn generate_representative(source: &ModuleSourceSnapshot) -> BTreeMap<String, Vec<u8>> {
    let output = RelativeOperationPath::parse("candidate").expect("fixture path must parse");
    let module = ModuleProjectionInput {
        name: "fixture".to_owned(),
        original_name: "Fixture".to_owned(),
        source_subpath: RelativeOperationPath::parse("modules/fixture")
            .expect("fixture path must parse"),
        source_digest: "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
            .to_owned(),
    };
    let dependency = PublishedSdkDependency::Registry {
        registry: "crates-io".to_owned(),
        exact_version: "1.0.0-beta.11.rust.5".to_owned(),
    };
    let authoring = ModuleAuthoringInput {
        source: source.clone(),
        generator_digest: digest("checked-generator"),
        sdk_dependency_alias: "renamed_sdk".to_owned(),
    };
    let target = CodegenTarget::decode_exact(support::TARGET_BYTES).expect("target must decode");
    let plan = project_operation(OperationProjectionRequest {
        target: &target,
        operation: OperationKind::GenerateModule,
        visible_schema_json: &support::adversarial_dependency_module_schema("renamed_sdk"),
        module: Some(&module),
        output: &output,
        sdk_dependency: &dependency,
        authoring: Some(&authoring),
        engine_version: None,
    })
    .expect("the representative module must generate beside its dependencies");
    plan.artifacts()
        .iter()
        .map(|(path, artifact)| {
            let path = path
                .as_str()
                .strip_prefix("candidate/")
                .expect("every artifact lies under the output root");
            (path.to_owned(), artifact.content.clone())
        })
        .collect()
}

fn compile(
    source: &ModuleSourceSnapshot,
    target: &ModuleTarget,
    generator: &Sha256Digest,
    visible_type_names: BTreeSet<WireName>,
) -> Result<ModuleCompilation, dagger_codegen::module::ModuleDiagnosticSet> {
    let generated = GeneratedTypeRegistry::empty(target.visible_schema_digest.clone());
    ModuleCompiler::compile(ModuleCompilationRequest {
        target,
        source,
        generated_types: &generated,
        visible_type_names: &visible_type_names,
        generator_digest: generator,
        sdk_dependency_alias: "renamed_sdk",
        checked_bindings: &BTreeMap::new(),
    })
}

fn source_snapshot(seed: u16, reverse: bool, changed: bool) -> ModuleSourceSnapshot {
    let suffix = if changed { "changed" } else { "stable" };
    let root = format!(
        r#"
mod child;

#[renamed_sdk::enum_type]
pub(crate) enum Mood {{
    /// Ready for execution.
    Ready,
    /// Retained only for compatibility.
    #[deprecated(note = "use Ready")]
    Waiting,
}}

#[renamed_sdk::object(root, rename = "FixtureRoot")]
pub(crate) struct Root {{
    #[dagger(field)]
    child: child::Child,
}}

#[renamed_sdk::methods]
impl Root {{
    #[dagger(constructor)]
    pub(crate) fn new(child: child::Child) -> Root {{ Root {{ child }} }}

    #[dagger(function, cache = "never", role = "check")]
    pub(crate) fn greet(&self, #[dagger(default = "world")] name: String) -> String {{
        format!("{{}}-{{name}}-{seed}-{suffix}", self.child.value)
    }}
}}
"#
    );
    let child = r#"
#[renamed_sdk::object(rename = "FixtureChild")]
pub(crate) struct Child {
    #[dagger(field)]
    pub(crate) value: String,
}
"#
    .to_owned();
    let mut entries = vec![("src/lib.rs", root), ("src/child.rs", child)];
    if reverse {
        entries.reverse();
    }
    snapshot(entries)
}

// The representative crate is authored the way an initialized module is: its root
// describes the module and declares the generated module, and its exports include a
// direct `ModuleError` result, Core handles whose Rust and target names differ, an
// agent, Core enums and scalars, with defaults and as a field, a collection, and an
// object that keeps no state, whose codec reads and writes an empty JSON object.
fn representative_source() -> ModuleSourceSnapshot {
    let mut snapshot = source_snapshot(7, false, false);
    let root_path = ModuleSourcePath::new("src/lib.rs").expect("fixture root path is valid");
    let document = snapshot
        .documents
        .get_mut(&root_path)
        .expect("fixture root exists");
    document.contents = format!(
        "//! A representative module.\n\npub mod dagger_generated;\n\nuse dagger_generated::{{Greeter, GreeterMode, ModuleContext, QueryGreeterOpts}};\n{}{}",
        document.contents,
        r#"
#[renamed_sdk::methods]
impl Root {
    #[dagger(function)]
    pub(crate) fn refuse(&self) -> Result<String, renamed_sdk::ModuleError> {
        Err(renamed_sdk::ModuleError::new("refused"))
    }

    #[dagger(function)]
    pub(crate) fn tree(&self, source: renamed_sdk::Directory) -> renamed_sdk::Directory {
        source
    }

    #[dagger(function, role = "agent")]
    pub(crate) fn model(&self, model: renamed_sdk::Llm) -> renamed_sdk::Llm {
        model
    }

    #[dagger(function)]
    pub(crate) fn mood(&self, mood: Mood) -> Mood {
        mood
    }

    #[dagger(function)]
    pub(crate) fn protocol(
        &self,
        #[dagger(default = renamed_sdk::NetworkProtocol::Udp)] protocol: renamed_sdk::NetworkProtocol,
    ) -> renamed_sdk::NetworkProtocol {
        protocol
    }

    #[dagger(function)]
    pub(crate) fn compression(
        &self,
        compression: renamed_sdk::ImageLayerCompression,
    ) -> renamed_sdk::ImageLayerCompression {
        compression
    }

    #[dagger(function)]
    pub(crate) fn platform(
        &self,
        #[dagger(default = "linux/arm64")] platform: renamed_sdk::Platform,
    ) -> renamed_sdk::Platform {
        platform
    }

    #[dagger(function)]
    pub(crate) fn labels(
        &self,
        #[dagger(default = "{\"a\":1}")] labels: renamed_sdk::Json,
    ) -> renamed_sdk::Json {
        labels
    }

    #[dagger(function)]
    pub(crate) fn data(&self, data: Option<renamed_sdk::Bytes>) -> Vec<renamed_sdk::Bytes> {
        data.into_iter().collect()
    }

    #[dagger(function)]
    pub(crate) fn entry(&self) -> Entry {
        Entry {
            kind: renamed_sdk::FileType::Regular,
        }
    }

    #[dagger(function)]
    pub(crate) fn marker(&self) -> Marker {
        Marker {}
    }

    // A dependency is constructed from the query root and called as Core is.
    #[dagger(function)]
    pub(crate) async fn hail(
        &self,
        name: String,
        #[dagger(context)] context: ModuleContext,
    ) -> Result<String, renamed_sdk::ModuleError> {
        let options = QueryGreeterOpts::default()
            .with_greeting("hi")
            .with_mode(GreeterMode::Loud);
        // The dependency has a function called `clone`, and another dependency is
        // itself called `clone`. Cloning the handle or the query root still clones.
        let query = context.query().clone();
        let greeter = query.greeter_opts(&options).clone();
        Ok(greeter.hello(name).await?)
    }

    // The dependency's object and enum leave in an object's private state.
    #[dagger(function)]
    pub(crate) fn welcome(&self, #[dagger(context)] context: ModuleContext) -> Welcome {
        Welcome {
            greeter: context.query().greeter(),
            mode: GreeterMode::Quiet,
            previous: None,
            outcome: None,
        }
    }

    // One pair of methods loads a dependency's object and a Core object from an ID.
    #[dagger(function)]
    pub(crate) async fn reload(
        &self,
        id: String,
        #[dagger(context)] context: ModuleContext,
    ) -> Result<Vec<String>, renamed_sdk::ModuleError> {
        let id = renamed_sdk::Id::new(id);
        let lazy: Greeter = context.query().r#ref(id.clone());
        let loaded: Greeter = context.query().load(id.clone()).await?;
        let directory: renamed_sdk::Directory = context.query().r#ref(id);
        Ok(vec![
            lazy.hello("lazy").await?,
            loaded.id().await?.as_str().to_owned(),
            directory.id().await?.as_str().to_owned(),
        ])
    }

    #[dagger(function)]
    pub(crate) fn items(&self) -> Items {
        Items {
            names: vec!["a".to_owned(), "b".to_owned()],
            selection: None,
            prefix: String::new(),
            base: renamed_sdk::CollectionBase::default(),
        }
    }
}

#[renamed_sdk::object(rename = "FixtureEntry")]
pub(crate) struct Entry {
    #[dagger(field)]
    kind: renamed_sdk::FileType,
}

#[renamed_sdk::object(rename = "FixtureMarker")]
pub(crate) struct Marker {}

// A module's own API may not name a dependency's type, but private state may hold its
// objects and enums, bare or wrapped, between calls.
#[renamed_sdk::object(rename = "FixtureWelcome")]
pub(crate) struct Welcome {
    #[dagger(state)]
    greeter: Greeter,
    #[dagger(state)]
    mode: GreeterMode,
    #[dagger(state)]
    previous: Option<Greeter>,
    // A dependency called `result` has the main object `Result`. Named by its path,
    // it is that handle and not the standard type.
    #[dagger(state)]
    outcome: Option<dagger_generated::Result>,
}

#[renamed_sdk::methods]
impl Welcome {
    #[dagger(function)]
    pub(crate) async fn greet(&self, name: String) -> Result<String, renamed_sdk::ModuleError> {
        let _ = (&self.previous, &self.outcome);
        Ok(self.greeter.with_mode(self.mode).hello(name).await?)
    }
}

#[renamed_sdk::object(collection, rename = "FixtureItems")]
#[derive(Clone)]
pub(crate) struct Items {
    #[dagger(field, keys)]
    names: Vec<String>,
    #[dagger(field, delta)]
    selection: Option<renamed_sdk::CollectionDelta>,
    #[dagger(field)]
    prefix: String,
    base: renamed_sdk::CollectionBase,
}

#[renamed_sdk::methods]
impl Items {
    #[dagger(function, get)]
    pub(crate) fn lookup(&self, name: String) -> Item {
        Item {
            name: format!("{}{name}", self.prefix),
        }
    }

    #[dagger(function)]
    pub(crate) fn with_prefix(&self, prefix: String) -> Items {
        Items {
            prefix,
            ..self.clone()
        }
    }

    #[dagger(function)]
    pub(crate) fn fresh(&self) -> Items {
        Items {
            names: vec!["z".to_owned()],
            selection: None,
            prefix: self.prefix.clone(),
            base: renamed_sdk::CollectionBase::default(),
        }
    }
}

#[renamed_sdk::object(rename = "FixtureItem")]
pub(crate) struct Item {
    #[dagger(field)]
    name: String,
}
"#
    );
    document.digest = Sha256Digest::hash_bytes(document.contents.as_bytes());
    snapshot.digest = source_snapshot_digest(&snapshot).expect("fixture snapshot must hash");
    snapshot
}

fn compile_contract_source(seed: u16, category: u8) -> ModuleSourceSnapshot {
    let mut snapshot = source_snapshot(seed, false, false);
    let root_path = ModuleSourcePath::new("src/lib.rs").expect("fixture root path is valid");
    let document = snapshot
        .documents
        .get_mut(&root_path)
        .expect("fixture root exists");
    match category {
        0 => {}
        1 => {
            document.contents.push_str(
                r#"
#[renamed_sdk::methods]
impl Root {
    #[dagger(function, rename = "greet")]
    pub(crate) fn duplicate(&self) -> String { String::new() }
}
"#,
            );
        }
        2 => {
            document.contents = document
                .contents
                .replace("pub(crate) struct Root", "struct Root")
        }
        3 => {
            document.contents = document
                .contents
                .replace("child: child::Child", "#[dagger(state)]\n    child: usize")
        }
        4 => document.contents = document.contents.replace("name: String", "name: u64"),
        5 => {
            document.contents = document
                .contents
                .replace("#[dagger(default = \"world\")]", "#[dagger(default = 7)]")
        }
        _ => {
            document.contents.push_str(
                r#"
#[renamed_sdk::object(root)]
pub(crate) struct OtherRoot;
"#,
            );
        }
    }
    document.digest = Sha256Digest::hash_bytes(document.contents.as_bytes());
    snapshot.digest = source_snapshot_digest(&snapshot).expect("fixture snapshot must hash");
    snapshot
}

fn snapshot(entries: Vec<(&str, String)>) -> ModuleSourceSnapshot {
    let documents = entries
        .into_iter()
        .map(|(path, contents)| {
            let path = ModuleSourcePath::new(path).expect("fixture path is valid");
            (path.clone(), SourceDocument::new(path, contents))
        })
        .collect();
    let mut snapshot = ModuleSourceSnapshot {
        format_version: FormatVersion::current(),
        package: ModulePackage {
            name: PackageName::new("fixture").expect("fixture package is valid"),
            crate_root: ModuleSourcePath::new("src/lib.rs").expect("fixture root is valid"),
            edition: value("2024"),
        },
        cfg: CfgEnvironment {
            values: BTreeMap::from([("unix".to_owned(), BTreeSet::new())]),
            features: BTreeSet::new(),
        },
        documents,
        digest: digest("pending"),
    };
    snapshot.digest = source_snapshot_digest(&snapshot).expect("fixture snapshot must hash");
    snapshot
}

trait SnapshotMutation {
    fn with_duplicate_function(self) -> Self;
}

impl SnapshotMutation for ModuleSourceSnapshot {
    fn with_duplicate_function(mut self) -> Self {
        let path = ModuleSourcePath::new("src/lib.rs").expect("fixture root is valid");
        let document = self.documents.get_mut(&path).expect("fixture root exists");
        document.contents.push_str(
            r#"
#[renamed_sdk::methods]
impl Root {
    #[dagger(function, rename = "greet")]
    pub(crate) fn duplicate(&self) -> String { String::new() }
}
"#,
        );
        document.digest = Sha256Digest::hash_bytes(document.contents.as_bytes());
        self.digest = source_snapshot_digest(&self).expect("mutated snapshot must hash");
        self
    }
}

fn target(visible_schema_digest: Sha256Digest) -> ModuleTarget {
    ModuleTarget {
        dagger_revision: value("0123456789abcdef0123456789abcdef01234567"),
        engine_version: value("v1.0.0"),
        rust_sdk_version: value("1.0.0-beta.11.rust.5"),
        rust_toolchain: value("1.89.0"),
        rust_edition: value("2024"),
        visible_schema_digest,
    }
}

fn observed(compilation: &ModuleCompilation) -> BTreeMap<GeneratedAssetPath, Sha256Digest> {
    compilation
        .assets
        .manifest
        .assets
        .iter()
        .map(|(path, asset)| (path.clone(), asset.digest.clone()))
        .collect()
}

fn digest(value: impl AsRef<[u8]>) -> Sha256Digest {
    Sha256Digest::hash_bytes(value.as_ref())
}

fn value(value: impl Into<String>) -> TargetValue {
    TargetValue::new(value).expect("fixture target value is valid")
}
