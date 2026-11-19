//! Typed module clients for a Cargo package that is not a module.
//!
//! `dagger module client add` records a module as a client of the nearest Cargo
//! package, and generation then gives the package one typed client for each:
//!
//! ```text
//! Cargo.toml                                  the SDK dependency, at exactly the release
//! src/dagger_clients/mod.rs                   one `pub mod` for each client
//! src/dagger_clients/.dagger-generated.json   the record that owns mod.rs
//! src/dagger_clients/<name>/…                 the client: mod.rs and a file for each type
//! src/dagger_clients/<name>/.dagger-generated.json
//! ```
//!
//! The package's author writes `mod dagger_clients;` once. Generation never edits
//! authored source, and replaces or removes only what its records own (the
//! `ownership` module). A client that is no longer recorded is removed the same
//! way, and `mod.rs` with the last of them.
//!
//! Every check runs before the first write: each schema against its digest, the
//! names, the manifest, the rendering, and every directory's ownership. A request
//! that is refused therefore leaves the package as it was.
//!
//! This is what Dagger's Go SDK does for a Go module that is not a Dagger module
//! (`generateStandaloneClients` in go-sdk.dang and `runPruneClients` in
//! cmd/dagger-go-sdk-codegen/prune.go of github.com/dagger/go-sdk at bb23e8d), with
//! `src/dagger_clients` where Go has `internal/dagger/clients`.

mod dependency;
mod ownership;

use std::collections::{BTreeMap, BTreeSet};
use std::fs;

use dagger_codegen::engine as compiler;
use dagger_codegen::naming::rust_module_name;

use crate::diagnostic::{EngineDiagnostic, EngineDiagnosticCode};
use crate::post_work::Cancellation;
use crate::runner::{OperationPostWork, PackagedPostWork, compiler_target, digest, format_rust};
use crate::{
    EngineSourceDescriptor, ExecutionResult, ExecutionResultKind, FormatVersion, OperationRoot,
    PackageClientsRequest, RelativeOperationPath,
};

use dependency::{AncestorManifest, plan_package_dependency};
use ownership::{GENERATED_MARKER, OwnedChanges, has_record, plan_owned_overlay};

/// The directory of a package's module clients, beneath the package.
const CLIENTS_DIRECTORY: &str = "src/dagger_clients";

/// Generates the module clients of one Cargo package with the packaged formatter.
///
/// `schemas` holds each client's schema at the path its request names.
///
/// # Errors
///
/// Returns a diagnostic, with nothing written, for whatever
/// [`execute_package_clients_with_post_work`] refuses.
pub async fn execute_package_clients(
    root: &OperationRoot,
    schemas: &OperationRoot,
    request: &PackageClientsRequest,
    descriptor: &EngineSourceDescriptor,
    cancel: &Cancellation,
) -> Result<ExecutionResult, EngineDiagnostic> {
    execute_package_clients_with_post_work(
        root,
        schemas,
        request,
        descriptor,
        cancel,
        &PackagedPostWork,
    )
    .await
}

/// Generates the module clients of one Cargo package, formatting through `post_work`.
///
/// The result lists every path written and every path removed. A second run with the
/// same request writes and removes nothing.
///
/// # Errors
///
/// Returns a diagnostic, with nothing written, when the request differs from the
/// packaged target, when a schema differs from its digest, when two clients would
/// take one Rust module name or a name is no Rust identifier, when the package's
/// manifest cannot take the SDK dependency, when a client cannot be rendered, and
/// when a file to replace holds its author's changes.
pub async fn execute_package_clients_with_post_work<P: OperationPostWork>(
    root: &OperationRoot,
    schemas: &OperationRoot,
    request: &PackageClientsRequest,
    descriptor: &EngineSourceDescriptor,
    cancel: &Cancellation,
    post_work: &P,
) -> Result<ExecutionResult, EngineDiagnostic> {
    validate_target(request, descriptor)?;
    let target = compiler_target(&request.target)?;

    let mut schema_bytes = Vec::with_capacity(request.clients.len());
    for (index, client) in request.clients.iter().enumerate() {
        let bytes = schemas.read(&client.schema.path)?;
        if digest(&bytes) != client.schema.digest {
            return Err(diagnostic(
                EngineDiagnosticCode::OperationInputInvalid,
                &format!("request.clients.{index}.schema.digest"),
                "schema bytes differ from the operation request digest",
            ));
        }
        schema_bytes.push(bytes);
    }
    let modules = client_modules(request)?;

    let package = &request.client_root;
    let clients_root = child(package, CLIENTS_DIRECTORY)?;
    let mut changes = OwnedChanges::default();
    let mut aggregated = BTreeMap::new();

    if !request.clients.is_empty() {
        let manifest_path = child(package, "Cargo.toml")?;
        if !root.exists(&manifest_path) {
            return Err(diagnostic(
                EngineDiagnosticCode::CargoManifestMissing,
                manifest_path.as_str(),
                "the package has no Cargo.toml; module clients are generated into a Cargo package",
            ));
        }
        let manifest = root.read(&manifest_path)?;
        let ancestors = ancestor_manifests(root, package)?;
        let dependency = plan_package_dependency(
            &manifest_path,
            &manifest,
            &ancestors
                .iter()
                .map(|(display, path, bytes)| AncestorManifest {
                    display: display.clone(),
                    path,
                    bytes,
                })
                .collect::<Vec<_>>(),
            &request.sdk_dependency,
        )?;
        if let Some(rendered) = dependency.rendered {
            changes.writes.insert(manifest_path, rendered);
        }

        // Render every client into a private tree and format it there, so that the
        // records hold the hashes of the bytes the package receives.
        let staging = tempfile::tempdir().map_err(|_| generation_failed())?;
        let mut rendered = Vec::with_capacity(request.clients.len());
        let mut rust_files = BTreeSet::new();
        for ((client, module), schema) in request.clients.iter().zip(&modules).zip(&schema_bytes) {
            let directory = child(&clients_root, module.directory())?;
            let plan = compiler::project_visible_schema(&target, schema)
                .map_err(|diagnostics| compiler_diagnostic(&diagnostics, &directory))?;
            let output = compiler::render_module_client(compiler::ModuleClientRenderInput {
                schema: &plan,
                output: &compiler::RelativeOperationPath::parse(directory.as_str())
                    .map_err(|_| generation_failed())?,
                sdk_dependency_alias: &dependency.alias,
                address: client.address.as_str(),
                pin: client.pin.as_ref().map_or("", |pin| pin.as_str()),
            })
            .map_err(|diagnostics| compiler_diagnostic(&diagnostics, &directory))?;
            let mut files = BTreeSet::new();
            for (path, artifact) in &output.artifacts {
                let path =
                    RelativeOperationPath::parse(path.as_str()).map_err(|_| generation_failed())?;
                let destination = path.join_lexically(staging.path());
                if let Some(parent) = destination.parent() {
                    fs::create_dir_all(parent).map_err(|_| generation_failed())?;
                }
                fs::write(destination, &artifact.content).map_err(|_| generation_failed())?;
                files.insert(path);
            }
            for work in &output.post_work {
                let compiler::PostWorkPlan::FormatRust { files } = work;
                for file in files {
                    rust_files.insert(
                        RelativeOperationPath::parse(file.as_str())
                            .map_err(|_| generation_failed())?,
                    );
                }
            }
            rendered.push((directory, files));
        }
        let staging_root = OperationRoot::open(staging.path())?;
        format_rust(
            post_work,
            &staging_root,
            &request.target.rust_toolchain,
            rust_files,
            cancel,
        )
        .await?;

        for (directory, files) in rendered {
            let mut overlay = BTreeMap::new();
            for path in files {
                let name = path
                    .as_str()
                    .strip_prefix(directory.as_str())
                    .and_then(|rest| rest.strip_prefix('/'))
                    .ok_or_else(generation_failed)?
                    .to_owned();
                overlay.insert(name, staging_root.read(&path)?);
            }
            merge(
                &mut changes,
                plan_owned_overlay(root, package, &directory, &overlay)?,
            );
        }
        aggregated.insert("mod.rs".to_owned(), aggregator(&modules).into_bytes());
    }

    // The directory itself: `mod.rs`, or nothing once the last client is gone. A
    // package that never had a client has no record there, and gets none.
    if !aggregated.is_empty() || has_record(root, &clients_root) {
        merge(
            &mut changes,
            plan_owned_overlay(root, package, &clients_root, &aggregated)?,
        );
    }
    // A client generated before and no longer recorded: a directory with a record
    // that is not one of this request's.
    let wanted = modules
        .iter()
        .map(ClientModule::directory)
        .collect::<BTreeSet<_>>();
    for name in removed_client_directories(root, &clients_root, &wanted)? {
        merge(
            &mut changes,
            plan_owned_overlay(
                root,
                package,
                &child(&clients_root, &name)?,
                &BTreeMap::new(),
            )?,
        );
    }

    for (path, bytes) in &changes.writes {
        let parent = root.ensure_parent(path)?;
        let name = path.as_str().rsplit('/').next().unwrap_or(path.as_str());
        fs::write(parent.join(name), bytes).map_err(|_| {
            diagnostic(
                EngineDiagnosticCode::PublicationFailed,
                path.as_str(),
                "generated file could not be written",
            )
        })?;
    }
    for path in &changes.removed {
        fs::remove_file(path.join_lexically(root.absolute())).map_err(|_| {
            diagnostic(
                EngineDiagnosticCode::PublicationFailed,
                path.as_str(),
                "generated file could not be removed",
            )
        })?;
    }
    Ok(ExecutionResult {
        format_version: FormatVersion,
        kind: ExecutionResultKind::PackageClients,
        output_root: package.clone(),
        touched_paths: changes.writes.into_keys().collect(),
        removed_paths: changes.removed,
        operation_manifest: None,
        vcs_generated: BTreeSet::new(),
        vcs_ignored: BTreeSet::new(),
        client_plan: None,
    })
}

/// The Rust module of one client: `r#gen` in `mod.rs`, `gen` on disk.
struct ClientModule {
    identifier: String,
}

impl ClientModule {
    fn directory(&self) -> &str {
        self.identifier
            .strip_prefix("r#")
            .unwrap_or(&self.identifier)
    }
}

// Each client is a Rust module named after its Dagger module, so two modules whose
// names make one Rust name cannot both be clients of a package: `engine-dev` and
// `engine_dev`, say. Directories are compared as a file system that ignores case
// compares them. No module can be the client `mod`: `mod.rs` declares the clients,
// and Rust would look for the file of `pub mod r#mod;` there as well as in
// `mod/mod.rs`, and refuse both.
fn client_modules(request: &PackageClientsRequest) -> Result<Vec<ClientModule>, EngineDiagnostic> {
    let mut modules = Vec::with_capacity(request.clients.len());
    let mut taken = BTreeMap::<String, &str>::new();
    for (index, client) in request.clients.iter().enumerate() {
        let identifier = rust_module_name(client.name.as_str());
        if syn::parse_str::<syn::Ident>(&identifier).is_err() {
            return Err(diagnostic(
                EngineDiagnosticCode::OperationInputInvalid,
                &format!("request.clients.{index}.name"),
                &format!(
                    "module `{}` has no name a Rust module can take; no client can be generated for it",
                    client.name
                ),
            ));
        }
        let module = ClientModule { identifier };
        if module.directory() == "mod" {
            return Err(diagnostic(
                EngineDiagnosticCode::OperationInputInvalid,
                &format!("request.clients.{index}.name"),
                &format!(
                    "module `{}` would be the Rust module `mod`, which Rust would look for in both src/dagger_clients/mod.rs and src/dagger_clients/mod/mod.rs; no client can be generated for it",
                    client.name
                ),
            ));
        }
        if let Some(other) = taken.insert(module.directory().to_lowercase(), client.name.as_str()) {
            return Err(diagnostic(
                EngineDiagnosticCode::OperationInputInvalid,
                &format!("request.clients.{index}.name"),
                &format!(
                    "modules `{other}` and `{}` would both be the Rust module `{}`; a package can have a client for only one of them",
                    client.name,
                    module.directory()
                ),
            ));
        }
        modules.push(module);
    }
    Ok(modules)
}

// `src/dagger_clients/mod.rs`. The author's own `cargo fmt` reaches this file, with
// the author's configuration and edition, so it holds nothing rustfmt would lay out
// differently: every line is short and unindented. `#[rustfmt::skip]` on a
// declaration keeps rustfmt out of that client's files, which are formatted once,
// here, and hashed; it also keeps rustfmt from reordering the declarations, which
// are in the order of their names.
fn aggregator(modules: &[ClientModule]) -> String {
    let mut modules = modules.iter().collect::<Vec<_>>();
    modules.sort_unstable_by_key(|module| module.directory());
    let mut source = format!(
        "{GENERATED_MARKER} by `dagger generate`. Do not edit.\n//! Typed clients for Dagger modules.\n"
    );
    for module in modules {
        source.push_str("\n#[rustfmt::skip]\npub mod ");
        source.push_str(&module.identifier);
        source.push_str(";\n");
    }
    source
}

fn removed_client_directories(
    root: &OperationRoot,
    clients_root: &RelativeOperationPath,
    wanted: &BTreeSet<&str>,
) -> Result<Vec<String>, EngineDiagnostic> {
    let absolute = clients_root.join_lexically(root.absolute());
    let unreadable = || {
        diagnostic(
            EngineDiagnosticCode::GenerationFailed,
            clients_root.as_str(),
            "the directory of module clients could not be read",
        )
    };
    let entries = match fs::read_dir(&absolute) {
        Ok(entries) => entries,
        // Absent, or not a directory: ownership of the directory itself says which.
        Err(_) => return Ok(Vec::new()),
    };
    let mut removed = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|_| unreadable())?;
        // A symlink is not a directory here, as in Go's `entry.IsDir()`.
        if !entry.file_type().map_err(|_| unreadable())?.is_dir() {
            continue;
        }
        let Ok(name) = entry.file_name().into_string() else {
            continue;
        };
        if wanted.contains(name.as_str()) {
            continue;
        }
        let Ok(directory) = child(clients_root, &name) else {
            continue;
        };
        if has_record(root, &directory) {
            removed.push(name);
        }
    }
    removed.sort();
    Ok(removed)
}

// The Cargo manifests above the package, nearest first, as far as the operation
// root's first directory: the adapter mounts the workspace there.
fn ancestor_manifests(
    root: &OperationRoot,
    package: &RelativeOperationPath,
) -> Result<Vec<(String, RelativeOperationPath, Vec<u8>)>, EngineDiagnostic> {
    let components = package.as_str().split('/').collect::<Vec<_>>();
    let mut manifests = Vec::new();
    for depth in (1..components.len()).rev() {
        let path =
            RelativeOperationPath::parse(&format!("{}/Cargo.toml", components[..depth].join("/")))
                .map_err(|_| generation_failed())?;
        if root.exists(&path) {
            let display = format!("{}Cargo.toml", "../".repeat(components.len() - depth));
            manifests.push((display, path.clone(), root.read(&path)?));
        }
    }
    Ok(manifests)
}

fn merge(changes: &mut OwnedChanges, more: OwnedChanges) {
    changes.writes.extend(more.writes);
    changes.removed.extend(more.removed);
}

fn validate_target(
    request: &PackageClientsRequest,
    descriptor: &EngineSourceDescriptor,
) -> Result<(), EngineDiagnostic> {
    descriptor.validate()?;
    let target = &request.target;
    if target.repository != descriptor.repository
        || target.dagger_revision != descriptor.dagger_revision
        || target.engine_version != descriptor.engine_version
        || target.rust_sdk_version != descriptor.rust_sdk_version
        || target.rust_toolchain != descriptor.rust_toolchain
        || target.core_schema_digest != descriptor.core_schema_digest
        || request.sdk_dependency != descriptor.sdk_dependency
    {
        return Err(diagnostic(
            EngineDiagnosticCode::OperationInputInvalid,
            "request.target",
            "operation request differs from the packaged immutable engine descriptor",
        ));
    }
    Ok(())
}

fn compiler_diagnostic(
    diagnostics: &dagger_codegen::diagnostic::DiagnosticSet,
    directory: &RelativeOperationPath,
) -> EngineDiagnostic {
    let primary = &diagnostics.diagnostics()[0];
    EngineDiagnostic::new(
        EngineDiagnosticCode::GenerationFailed,
        Some(
            primary
                .coordinate
                .as_ref()
                .map_or(directory.as_str(), |coordinate| coordinate.as_str()),
        ),
        format!("{}: {}", primary.code, primary.message),
    )
}

fn child(
    parent: &RelativeOperationPath,
    suffix: &str,
) -> Result<RelativeOperationPath, EngineDiagnostic> {
    RelativeOperationPath::parse(&format!("{parent}/{suffix}")).map_err(|_| {
        diagnostic(
            EngineDiagnosticCode::OutputPathEscape,
            parent.as_str(),
            "module client path is not confined",
        )
    })
}

fn generation_failed() -> EngineDiagnostic {
    diagnostic(
        EngineDiagnosticCode::GenerationFailed,
        "operation",
        "module clients could not be staged",
    )
}

fn diagnostic(code: EngineDiagnosticCode, coordinate: &str, message: &str) -> EngineDiagnostic {
    EngineDiagnostic::new(code, Some(coordinate), message)
}
