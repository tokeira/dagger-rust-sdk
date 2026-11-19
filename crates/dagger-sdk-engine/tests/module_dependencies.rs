//! A module's dependency list is exactly the clients recorded for its scope.
//!
//! The rules are those of Dagger's manifest helper, `main.dang` of
//! github.com/dagger/sdk-helpers at v1.0.12. Each case states the manifest before,
//! the clients, and the manifest after, byte for byte.

use std::fs;

use dagger_sdk_engine::{
    EngineDiagnosticCode, EngineExecutionRequest, ExecutionResultKind, MODULE_MANIFEST,
    ModuleClient, ModuleDependenciesRequest, OperationRoot, RelativeOperationPath, canonical_bytes,
    decode_canonical, execute_module_dependencies, plan_module_dependencies,
};

/// A new module's manifest, as the adapter's `moduleManifest` writes it.
const NEW_MODULE: &str = "name = \"gate\"\n\
engineVersion = \"v1.0.0-beta.27\"\n\
\n\
[runtime]\n\
source = \"github.com/tokeira/dagger-rust-sdk/runtime@v0.9.0\"\n";

const GATE: &str = ".dagger/modules/gate";

fn local(name: &str, source_root_subpath: &str) -> ModuleClient {
    ModuleClient::Local {
        name: name.to_owned(),
        source_root_subpath: source_root_subpath.to_owned(),
    }
}

fn git(name: &str, source: &str) -> ModuleClient {
    ModuleClient::Git {
        name: name.to_owned(),
        source: source.to_owned(),
    }
}

struct Written {
    rule: &'static str,
    manifest: String,
    module_directory: &'static str,
    clients: Vec<ModuleClient>,
    expected: String,
}

fn written_cases() -> Vec<Written> {
    vec![
        Written {
            rule: "a new local client is a path relative to the module, a new git client its reference, with no pin",
            manifest: NEW_MODULE.to_owned(),
            module_directory: GATE,
            clients: vec![
                local("greeter", ".dagger/modules/greeter"),
                git("fetcher", "github.com/acme/fetcher@v1.2.0"),
            ],
            expected: format!(
                "{NEW_MODULE}\n\
                 [[dependencies]]\n\
                 name = \"fetcher\"\n\
                 source = \"github.com/acme/fetcher@v1.2.0\"\n\
                 \n\
                 [[dependencies]]\n\
                 name = \"greeter\"\n\
                 source = \"../greeter\"\n"
            ),
        },
        Written {
            rule: "a dependency written by hand and not recorded as a client is removed",
            manifest: format!(
                "{NEW_MODULE}\n\
                 [[dependencies]]\n\
                 name = \"stale\"\n\
                 source = \"../stale\"\n"
            ),
            module_directory: GATE,
            clients: vec![local("greeter", ".dagger/modules/greeter")],
            expected: format!(
                "{NEW_MODULE}\n\
                 [[dependencies]]\n\
                 name = \"greeter\"\n\
                 source = \"../greeter\"\n"
            ),
        },
        Written {
            rule: "a dependency with the client's source keeps its name and settings and loses its pin and indentation",
            manifest: format!(
                "{NEW_MODULE}\n\
                 [[dependencies]]\n  \
                 source = \"./../greeter/\"\n  \
                 pin = \"0123456789abcdef0123456789abcdef01234567\"\n  \
                 ignoreChecks = [\"slow\"] # too slow here\n  \
                 name = \"hello\"\n"
            ),
            module_directory: GATE,
            clients: vec![local("greeter", ".dagger/modules/greeter")],
            expected: format!(
                "{NEW_MODULE}\n\
                 [[dependencies]]\n\
                 name = \"hello\"\n\
                 source = \"../greeter\"\n\
                 ignoreChecks = [\"slow\"] # too slow here\n"
            ),
        },
        Written {
            rule: "a git dependency with the client's reference is adopted and loses its pin",
            manifest: format!(
                "{NEW_MODULE}\n\
                 [[dependencies]]\n\
                 name = \"fetch\"\n\
                 source = \"github.com/acme/fetcher@v1.2.0\"\n\
                 pin = \"0123456789abcdef0123456789abcdef01234567\"\n"
            ),
            module_directory: GATE,
            clients: vec![git("fetcher", "github.com/acme/fetcher@v1.2.0")],
            expected: format!(
                "{NEW_MODULE}\n\
                 [[dependencies]]\n\
                 name = \"fetch\"\n\
                 source = \"github.com/acme/fetcher@v1.2.0\"\n"
            ),
        },
        Written {
            rule: "an adopted dependency without a name gets the target's original name",
            manifest: format!(
                "{NEW_MODULE}\n\
                 [[dependencies]]\n\
                 source = \"../greeter\"\n"
            ),
            module_directory: GATE,
            clients: vec![local("greeter", ".dagger/modules/greeter")],
            expected: format!(
                "{NEW_MODULE}\n\
                 [[dependencies]]\n\
                 name = \"greeter\"\n\
                 source = \"../greeter\"\n"
            ),
        },
        Written {
            rule: "dependencies are ordered by the targets' original names, whatever an adopted one is called",
            manifest: format!(
                "{NEW_MODULE}\n\
                 [[dependencies]]\n\
                 name = \"zz-first\"\n\
                 source = \"../alpha\"\n"
            ),
            module_directory: GATE,
            clients: vec![
                local("gamma", ".dagger/modules/gamma"),
                local("alpha", ".dagger/modules/alpha"),
                local("beta", ".dagger/modules/beta"),
            ],
            expected: format!(
                "{NEW_MODULE}\n\
                 [[dependencies]]\n\
                 name = \"zz-first\"\n\
                 source = \"../alpha\"\n\
                 \n\
                 [[dependencies]]\n\
                 name = \"beta\"\n\
                 source = \"../beta\"\n\
                 \n\
                 [[dependencies]]\n\
                 name = \"gamma\"\n\
                 source = \"../gamma\"\n"
            ),
        },
        Written {
            rule: "a local source is relative to the module directory, wherever both are",
            manifest: NEW_MODULE.to_owned(),
            module_directory: "services/api/module",
            clients: vec![
                local("below", "services/api/module/deps/below"),
                local("cousin", "/services/shared/cousin/"),
                local("root", "."),
                local("sibling", "./services/api/sibling"),
            ],
            expected: format!(
                "{NEW_MODULE}\n\
                 [[dependencies]]\n\
                 name = \"below\"\n\
                 source = \"deps/below\"\n\
                 \n\
                 [[dependencies]]\n\
                 name = \"cousin\"\n\
                 source = \"../../shared/cousin\"\n\
                 \n\
                 [[dependencies]]\n\
                 name = \"root\"\n\
                 source = \"../../..\"\n\
                 \n\
                 [[dependencies]]\n\
                 name = \"sibling\"\n\
                 source = \"../sibling\"\n"
            ),
        },
        Written {
            rule: "a module at the workspace root reaches a local client by its path from the root",
            manifest: NEW_MODULE.to_owned(),
            module_directory: ".",
            clients: vec![local("lib", "deps/lib")],
            expected: format!(
                "{NEW_MODULE}\n\
                 [[dependencies]]\n\
                 name = \"lib\"\n\
                 source = \"deps/lib\"\n"
            ),
        },
        Written {
            rule: "comments and everything outside the dependency tables are kept byte for byte",
            manifest: "# The gate module.\n\
                 name = \"gate\"   # keep this spacing\n\
                 engineVersion = 'v1.0.0-beta.27'\n\
                 include = [\n  \"../shared/**\", # shared sources\n]\n\
                 \n\
                 # The runtime, pinned by hand.\n\
                 [runtime]\n  \
                 source = \"github.com/tokeira/dagger-rust-sdk/runtime@v0.9.0\"\n\
                 \n\
                 # Greets people.\n\
                 [[dependencies]]\n\
                 name = \"greeter\"\n\
                 source = \"../greeter\"\n\
                 \n\
                 # Gone after generation.\n\
                 [[dependencies]]\n\
                 name = \"stale\"\n\
                 source = \"../stale\"\n\
                 \n\
                 # Generation settings.\n\
                 [codegen]\n\
                 automaticGitignore = false\n\
                 \n\
                 [future.table]\n\
                 unknown = { kept = true }\n\
                 # The end.\n"
                .to_owned(),
            module_directory: GATE,
            clients: vec![
                local("greeter", ".dagger/modules/greeter"),
                git("fetcher", "github.com/acme/fetcher@v1.2.0"),
            ],
            expected: "# The gate module.\n\
                 name = \"gate\"   # keep this spacing\n\
                 engineVersion = 'v1.0.0-beta.27'\n\
                 include = [\n  \"../shared/**\", # shared sources\n]\n\
                 \n\
                 # The runtime, pinned by hand.\n\
                 [runtime]\n  \
                 source = \"github.com/tokeira/dagger-rust-sdk/runtime@v0.9.0\"\n\
                 \n\
                 [[dependencies]]\n\
                 name = \"fetcher\"\n\
                 source = \"github.com/acme/fetcher@v1.2.0\"\n\
                 \n\
                 # Greets people.\n\
                 [[dependencies]]\n\
                 name = \"greeter\"\n\
                 source = \"../greeter\"\n\
                 \n\
                 # Generation settings.\n\
                 [codegen]\n\
                 automaticGitignore = false\n\
                 \n\
                 [future.table]\n\
                 unknown = { kept = true }\n\
                 # The end.\n"
                .to_owned(),
        },
        Written {
            rule: "new dependency tables follow [runtime] and precede the tables after it",
            manifest: format!("{NEW_MODULE}\n[codegen]\nautomaticGitignore = false\n"),
            module_directory: GATE,
            clients: vec![local("greeter", ".dagger/modules/greeter")],
            expected: format!(
                "{NEW_MODULE}\n\
                 [[dependencies]]\n\
                 name = \"greeter\"\n\
                 source = \"../greeter\"\n\
                 \n\
                 [codegen]\n\
                 automaticGitignore = false\n"
            ),
        },
        Written {
            rule: "an empty client list removes every dependency",
            manifest: format!(
                "{NEW_MODULE}\n\
                 [[dependencies]]\n\
                 name = \"greeter\"\n\
                 source = \"../greeter\"\n\
                 \n\
                 [[dependencies]]\n\
                 name = \"stale\"\n\
                 source = \"../stale\"\n\
                 \n\
                 [codegen]\n\
                 automaticGitignore = false\n"
            ),
            module_directory: GATE,
            clients: Vec::new(),
            expected: format!("{NEW_MODULE}\n[codegen]\nautomaticGitignore = false\n"),
        },
        Written {
            rule: "a module without dependencies or clients is left as it is",
            manifest: "name = \"gate\"\n\n[runtime]\n  source = \"rust\"\n".to_owned(),
            module_directory: GATE,
            clients: Vec::new(),
            expected: "name = \"gate\"\n\n[runtime]\n  source = \"rust\"\n".to_owned(),
        },
        Written {
            rule: "an inline dependencies array is written back as tables",
            manifest: "name = \"gate\"\n\
                 dependencies = [{ source = \"../greeter\", name = \"hello\", pin = \"abc\" }, \"../stale\"]\n\
                 \n\
                 [runtime]\n\
                 source = \"rust\"\n"
                .to_owned(),
            module_directory: GATE,
            clients: vec![local("greeter", ".dagger/modules/greeter")],
            expected: "name = \"gate\"\n\
                 \n\
                 [runtime]\n\
                 source = \"rust\"\n\
                 \n\
                 [[dependencies]]\n\
                 name = \"hello\"\n\
                 source = \"../greeter\"\n"
                .to_owned(),
        },
        Written {
            rule: "tables nested in an adopted dependency stay beneath it",
            manifest: format!(
                "{NEW_MODULE}\n\
                 [[dependencies]]\n\
                 name = \"zeta\"\n\
                 source = \"../zeta\"\n\
                 \n\
                 [[dependencies.customizations]]\n\
                 argument = \"src\"\n\
                 \n\
                 [codegen]\n\
                 automaticGitignore = false\n"
            ),
            module_directory: GATE,
            clients: vec![
                local("zeta", ".dagger/modules/zeta"),
                local("alpha", ".dagger/modules/alpha"),
            ],
            expected: format!(
                "{NEW_MODULE}\n\
                 [[dependencies]]\n\
                 name = \"alpha\"\n\
                 source = \"../alpha\"\n\
                 \n\
                 [[dependencies]]\n\
                 name = \"zeta\"\n\
                 source = \"../zeta\"\n\
                 \n\
                 [[dependencies.customizations]]\n\
                 argument = \"src\"\n\
                 \n\
                 [codegen]\n\
                 automaticGitignore = false\n"
            ),
        },
        Written {
            rule: "a manifest without [runtime] gets its dependencies ahead of its tables",
            manifest: "name = \"gate\"\n\n[codegen]\nautomaticGitignore = false\n".to_owned(),
            module_directory: GATE,
            clients: vec![local("greeter", ".dagger/modules/greeter")],
            expected: "name = \"gate\"\n\
                 \n\
                 [[dependencies]]\n\
                 name = \"greeter\"\n\
                 source = \"../greeter\"\n\
                 \n\
                 [codegen]\n\
                 automaticGitignore = false\n"
                .to_owned(),
        },
    ]
}

#[test]
fn each_manifest_rule_writes_the_expected_bytes_and_is_idempotent() {
    for case in written_cases() {
        let written =
            plan_module_dependencies(&case.manifest, case.module_directory, &case.clients)
                .unwrap_or_else(|diagnostic| panic!("{}: refused: {diagnostic}", case.rule));
        assert_eq!(written, case.expected, "{}", case.rule);
        let again = plan_module_dependencies(&written, case.module_directory, &case.clients)
            .unwrap_or_else(|diagnostic| panic!("{}: second run refused: {diagnostic}", case.rule));
        assert_eq!(
            again, written,
            "{}: a second run changed the manifest",
            case.rule
        );
    }
}

/// The helper's own tests state its output in fragments rather than whole files.
/// These manifests are the two it gives in full, both in
/// `.dagger/modules/sdk-helpers-dev` of github.com/dagger/sdk-helpers at v1.0.12:
/// the TOML that `loadsManifestFormats` (manifest.dang) loads, which is in the form
/// `tomlContents` writes, and the manifest `reusesDependencyWithSameSource`
/// (main.dang) starts from, with the output `tomlContents` gives for its
/// `withLegacyRuntimeDependencies(modules: [module])` call.
#[test]
fn output_equals_what_sdk_helpers_v1_0_12_writes() {
    let without_dependency = "name = \"from-toml\"\nengineVersion = \"v0.20.3\"\ninclude = [\"toml/**\"]\nsource = \"toml-src\"\n\n\
        [runtime]\nsource = \"go\"\n\n\
        [codegen]\nautomaticGitignore = false\n\n\
        [entrypoint]\nkind = \"dang\"\nsource = \"main.dang\"\n";
    let loads_manifest_formats = "name = \"from-toml\"\nengineVersion = \"v0.20.3\"\ninclude = [\"toml/**\"]\nsource = \"toml-src\"\n\n\
        [runtime]\nsource = \"go\"\n\n\
        [[dependencies]]\nname = \"toml-dep\"\nsource = \"example.invalid/toml-dep\"\n\n\
        [codegen]\nautomaticGitignore = false\n\n\
        [entrypoint]\nkind = \"dang\"\nsource = \"main.dang\"\n";
    let clients = [git("toml-dep", "example.invalid/toml-dep")];
    for manifest in [without_dependency, loads_manifest_formats] {
        assert_eq!(
            plan_module_dependencies(manifest, "app", &clients).unwrap(),
            loads_manifest_formats
        );
    }

    // The helper also removes the indentation of [runtime], which it renders again
    // from its model. The manifest here starts from the unindented [runtime] that
    // v1.0.12 writes, so the two outputs are the same bytes.
    let reuses_dependency = "name = \"app\"\nengineVersion = \"v0.21.9\"\n\n[runtime]\nsource = \"go\"\n\n\
        [[dependencies]]\n  name = \"alias\"\n  source = \"../deps/lib\"\n\n\
        [[dependencies]]\n  name = \"stale\"\n  source = \"../stale\"\n";
    assert_eq!(
        plan_module_dependencies(reuses_dependency, "app", &[local("lib", "deps/lib")]).unwrap(),
        "name = \"app\"\nengineVersion = \"v0.21.9\"\n\n[runtime]\nsource = \"go\"\n\n\
         [[dependencies]]\nname = \"alias\"\nsource = \"../deps/lib\"\n"
    );
}

#[test]
fn refusals_name_what_is_wrong() {
    let dependency =
        format!("{NEW_MODULE}\n[[dependencies]]\nname = \"beta\"\nsource = \"../alpha\"\n");
    let cases: Vec<(&str, &str, Vec<ModuleClient>, &str)> = vec![
        (
            "a local source that is still absolute after normalization",
            NEW_MODULE,
            vec![local("greeter", "//srv/greeter")],
            "local dependency source must be relative to the manifest file: /srv/greeter",
        ),
        (
            "a local source on a drive",
            NEW_MODULE,
            vec![local("greeter", "C:/modules/greeter")],
            "local dependency source must be relative to the manifest file: C:/modules/greeter",
        ),
        (
            "a local source on a network share",
            NEW_MODULE,
            vec![local("greeter", "\\\\host\\greeter")],
            "local dependency source must be relative to the manifest file",
        ),
        (
            "a local source outside the workspace",
            NEW_MODULE,
            vec![local("greeter", "../outside/greeter")],
            "local dependency source must be inside the workspace: ../outside/greeter",
        ),
        (
            "two clients whose modules share a name",
            NEW_MODULE,
            vec![
                local("greeter", ".dagger/modules/greeter"),
                git("greeter", "github.com/acme/greeter@v2"),
            ],
            "clients `../greeter` and `github.com/acme/greeter@v2` are both modules named `greeter`",
        ),
        (
            "an adopted dependency named as another client's module",
            &dependency,
            vec![
                local("alpha", ".dagger/modules/alpha"),
                local("beta", ".dagger/modules/beta"),
            ],
            "dependencies `../alpha` and `../beta` would both be named `beta`",
        ),
        (
            "a client without a name",
            NEW_MODULE,
            vec![local("", ".dagger/modules/greeter")],
            "a client's module has no name",
        ),
        (
            "a manifest that is not TOML",
            "name = \n",
            vec![local("greeter", ".dagger/modules/greeter")],
            "dagger-module.toml is not valid TOML",
        ),
        (
            "dependencies that are not a list of tables",
            "name = \"gate\"\ndependencies = \"../greeter\"\n",
            vec![local("greeter", ".dagger/modules/greeter")],
            "`dependencies` in dagger-module.toml is not a list of tables",
        ),
        (
            "an inline dependencies array holding something other than a dependency",
            "name = \"gate\"\ndependencies = [1]\n",
            Vec::new(),
            "`dependencies` in dagger-module.toml is not a list of tables",
        ),
    ];
    for (rule, manifest, clients, message) in cases {
        let diagnostic = plan_module_dependencies(manifest, GATE, &clients)
            .expect_err(&format!("{rule}: unexpectedly written"));
        assert_eq!(
            diagnostic.code,
            EngineDiagnosticCode::OperationInputInvalid,
            "{rule}"
        );
        assert!(
            diagnostic.message.contains(message),
            "{rule}: message was {:?}",
            diagnostic.message
        );
    }
}

fn request(clients: Vec<ModuleClient>) -> ModuleDependenciesRequest {
    ModuleDependenciesRequest {
        format_version: dagger_sdk_engine::FormatVersion,
        workspace_root: RelativeOperationPath::parse("workspace").unwrap(),
        module_root: RelativeOperationPath::parse("workspace/.dagger/modules/gate").unwrap(),
        clients,
    }
}

fn module_tree() -> (tempfile::TempDir, std::path::PathBuf) {
    let temporary = tempfile::tempdir().unwrap();
    let module = temporary.path().join("workspace/.dagger/modules/gate");
    fs::create_dir_all(&module).unwrap();
    (temporary, module)
}

#[test]
fn execution_writes_the_manifest_once_and_reports_only_a_change() {
    let (temporary, module) = module_tree();
    fs::write(module.join(MODULE_MANIFEST), NEW_MODULE).unwrap();
    let root = OperationRoot::open(temporary.path()).unwrap();
    let request = request(vec![local("greeter", ".dagger/modules/greeter")]);

    let first = execute_module_dependencies(&root, &request).unwrap();
    assert_eq!(first.kind, ExecutionResultKind::ModuleDependencies);
    assert_eq!(first.output_root.as_str(), "workspace/.dagger/modules/gate");
    assert_eq!(
        first
            .touched_paths
            .iter()
            .map(RelativeOperationPath::as_str)
            .collect::<Vec<_>>(),
        ["workspace/.dagger/modules/gate/dagger-module.toml"]
    );
    assert!(first.operation_manifest.is_none());
    let written = fs::read_to_string(module.join(MODULE_MANIFEST)).unwrap();
    assert_eq!(
        written,
        format!("{NEW_MODULE}\n[[dependencies]]\nname = \"greeter\"\nsource = \"../greeter\"\n")
    );

    let second = execute_module_dependencies(&root, &request).unwrap();
    assert!(second.touched_paths.is_empty());
    assert_eq!(
        fs::read_to_string(module.join(MODULE_MANIFEST)).unwrap(),
        written
    );
}

#[test]
fn a_refusal_writes_nothing() {
    let (temporary, module) = module_tree();
    let manifest =
        format!("{NEW_MODULE}\n[[dependencies]]\nname = \"stale\"\nsource = \"../stale\"\n");
    fs::write(module.join(MODULE_MANIFEST), &manifest).unwrap();
    let root = OperationRoot::open(temporary.path()).unwrap();

    let diagnostic = execute_module_dependencies(
        &root,
        &request(vec![
            local("greeter", ".dagger/modules/greeter"),
            local("greeter", ".dagger/modules/other-greeter"),
        ]),
    )
    .unwrap_err();
    assert_eq!(diagnostic.code, EngineDiagnosticCode::OperationInputInvalid);
    assert_eq!(
        fs::read_to_string(module.join(MODULE_MANIFEST)).unwrap(),
        manifest
    );
    assert_eq!(fs::read_dir(&module).unwrap().count(), 1);
}

#[test]
fn a_module_configured_by_dagger_json_is_kept_or_refused() {
    let (temporary, module) = module_tree();
    let legacy = module.join("dagger.json");
    let root = OperationRoot::open(temporary.path()).unwrap();

    // Nothing to write: the module has no dependencies and no clients.
    fs::write(&legacy, "{\"name\":\"gate\",\"sdk\":{\"source\":\"rust\"}}").unwrap();
    let unchanged = execute_module_dependencies(&root, &request(Vec::new())).unwrap();
    assert!(unchanged.touched_paths.is_empty());

    // A client would have to be written to dagger.json.
    let diagnostic = execute_module_dependencies(
        &root,
        &request(vec![local("greeter", ".dagger/modules/greeter")]),
    )
    .unwrap_err();
    assert_eq!(diagnostic.code, EngineDiagnosticCode::OperationInputInvalid);
    assert_eq!(
        diagnostic.coordinate.as_deref(),
        Some("workspace/.dagger/modules/gate/dagger.json")
    );
    assert_eq!(diagnostic.message, migrate_advice("/.dagger/modules/gate"));

    // A dependency would have to be removed from dagger.json.
    let with_dependency =
        "{\"name\":\"gate\",\"dependencies\":[{\"name\":\"stale\",\"source\":\"../stale\"}]}";
    fs::write(&legacy, with_dependency).unwrap();
    execute_module_dependencies(&root, &request(Vec::new())).unwrap_err();
    assert_eq!(fs::read_to_string(&legacy).unwrap(), with_dependency);
    assert!(!module.join(MODULE_MANIFEST).exists());

    // No configuration at all is not a module.
    fs::remove_file(&legacy).unwrap();
    let diagnostic = execute_module_dependencies(&root, &request(Vec::new())).unwrap_err();
    assert!(
        diagnostic
            .message
            .contains("the module has no dagger-module.toml")
    );
}

// The advice names the module by its path from the workspace root, which
// `dagger module migrate` reads wherever it runs; the root module is `/`.
#[test]
fn the_dagger_json_refusal_names_the_module_to_migrate() {
    let temporary = tempfile::tempdir().unwrap();
    let workspace = temporary.path().join("workspace");
    fs::create_dir_all(&workspace).unwrap();
    fs::write(
        workspace.join("dagger.json"),
        "{\"name\":\"gate\",\"sdk\":{\"source\":\"rust\"}}",
    )
    .unwrap();
    let root = OperationRoot::open(temporary.path()).unwrap();
    let request = ModuleDependenciesRequest {
        format_version: dagger_sdk_engine::FormatVersion,
        workspace_root: RelativeOperationPath::parse("workspace").unwrap(),
        module_root: RelativeOperationPath::parse("workspace").unwrap(),
        clients: vec![local("greeter", "greeter")],
    };
    let diagnostic = execute_module_dependencies(&root, &request).unwrap_err();
    assert_eq!(diagnostic.message, migrate_advice("/"));
}

fn migrate_advice(module: &str) -> String {
    format!(
        "the module is configured by dagger.json, where the Rust SDK does not write \
         dependencies; run `dagger module migrate {module}`, which moves its configuration \
         to dagger-module.toml and records its dependencies as clients, then \
         `dagger generate`; a coding agent passes `-y` to apply each, or `--no-apply` to \
         preview the migration"
    )
}

#[test]
fn a_module_root_outside_the_workspace_root_is_refused() {
    let (temporary, _) = module_tree();
    let root = OperationRoot::open(temporary.path()).unwrap();
    let mut request = request(Vec::new());
    request.workspace_root = RelativeOperationPath::parse("work").unwrap();
    let diagnostic = execute_module_dependencies(&root, &request).unwrap_err();
    assert!(
        diagnostic
            .message
            .contains("the module root is outside the workspace root")
    );
}

#[test]
fn the_request_document_is_closed_and_canonical() {
    let document = "{\n  \"request\": {\n    \"clients\": [\n      {\n        \"kind\": \"local\",\n        \"name\": \"greeter\",\n        \"source_root_subpath\": \".dagger/modules/greeter\"\n      },\n      {\n        \"kind\": \"git\",\n        \"name\": \"fetcher\",\n        \"source\": \"github.com/acme/fetcher@v1.2.0\"\n      }\n    ],\n    \"format_version\": 1,\n    \"module_root\": \"workspace/.dagger/modules/gate\",\n    \"workspace_root\": \"workspace\"\n  },\n  \"request_kind\": \"write-module-dependencies\"\n}\n";
    let decoded: EngineExecutionRequest = decode_canonical(document.as_bytes()).unwrap();
    assert_eq!(
        decoded,
        EngineExecutionRequest::WriteModuleDependencies(request(vec![
            local("greeter", ".dagger/modules/greeter"),
            git("fetcher", "github.com/acme/fetcher@v1.2.0"),
        ]))
    );
    assert_eq!(canonical_bytes(&decoded).unwrap(), document.as_bytes());

    // A git client carries no path, a local client no reference, and neither a pin.
    for unknown in [
        document.replace(
            "\"source\": \"github.com/acme/fetcher@v1.2.0\"",
            "\"source\": \"github.com/acme/fetcher@v1.2.0\",\n        \"source_root_subpath\": \"fetcher\"",
        ),
        document.replace(
            "\"name\": \"greeter\",",
            "\"name\": \"greeter\",\n        \"pin\": \"0123456789abcdef0123456789abcdef01234567\",",
        ),
        document.replace("\"kind\": \"git\"", "\"kind\": \"directory\""),
    ] {
        assert!(decode_canonical::<EngineExecutionRequest>(unknown.as_bytes()).is_err());
    }
}
