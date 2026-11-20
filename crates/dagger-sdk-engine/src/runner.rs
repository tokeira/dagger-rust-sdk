//! End-to-end private operation runner built from pure projection and confined I/O.
//!
//! The runner validates every semantic identity before invoking the pure compiler,
//! performs closed post-work in an isolated tree, recomputes final digests, and exposes
//! output only through manifest-authorized failure-atomic publication.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;

use dagger_codegen::diagnostic::DiagnosticSet as CompilerDiagnosticSet;
use dagger_codegen::engine as compiler;
use dagger_codegen::target::CodegenTarget;
use sha2::{Digest as _, Sha256};

use crate::canonical::{DigestDomain, canonical_digest};
use crate::diagnostic::{EngineDiagnostic, EngineDiagnosticCode};
use crate::post_work::{
    Cancellation, ProcessOutcome, current_allowlisted_environment, require_convergence,
};
use crate::project::source_snapshot::{SourceSnapshotBuilder, SourceSnapshotRequest};
use crate::publication::{OperationCandidate, publish, verify_ownership};
use crate::root::validate_path_sets;
use crate::{
    ArtifactKind, ArtifactOwnership, ArtifactRecord, CandidateArtifact, EngineRelease,
    EngineSourceDescriptor, ExactRustToolchain, ExactVersion, ExecutionResult, ExecutionResultKind,
    FormatVersion, GenerationMode, GeneratorIdentity, ModuleConfigFormat, OperationManifest,
    OperationRequest, OperationRoot, PostWorkPlan, PostWorkRecord, PublishedSdkDependency,
    RelativeOperationPath, Sha256Digest, TargetIdentity,
};

const TARGET_DESCRIPTOR: &[u8] = include_bytes!("../../../codegen/target.json");

/// Executes the closed post-work plans selected by the production operation runner.
///
/// Engine packaging uses [`PackagedPostWork`]. Engine-free fixtures inject an executor
/// for the same typed plans so the production compiler/reconciler/publisher can run on
/// a development host without pretending its paths are the packaged Linux image.
// The runner awaits each operation in place and never exposes or spawns its future,
// so requiring a public `Send` future would add ceremony without a usable guarantee.
#[allow(async_fn_in_trait)]
pub trait OperationPostWork: Send + Sync {
    /// Executes one runner-authored plan in its confined staging root.
    async fn execute(
        &self,
        root: &OperationRoot,
        plan: &PostWorkPlan,
        environment: &BTreeMap<String, String>,
        cancel: &Cancellation,
    ) -> Result<ProcessOutcome, EngineDiagnostic>;
}

/// Production executor backed by immutable paths in the packaged engine content.
#[derive(Clone, Copy, Debug, Default)]
pub struct PackagedPostWork;

impl OperationPostWork for PackagedPostWork {
    async fn execute(
        &self,
        root: &OperationRoot,
        plan: &PostWorkPlan,
        environment: &BTreeMap<String, String>,
        cancel: &Cancellation,
    ) -> Result<ProcessOutcome, EngineDiagnostic> {
        crate::post_work::execute(root, plan, environment, cancel).await
    }
}

/// Executes one validated operation and publishes its ownership manifest last.
pub async fn execute_operation(
    root: &OperationRoot,
    request: &OperationRequest,
    schema: &[u8],
    descriptor: &EngineSourceDescriptor,
    cancel: &Cancellation,
) -> Result<ExecutionResult, EngineDiagnostic> {
    execute_operation_with_post_work(root, request, schema, descriptor, cancel, &PackagedPostWork)
        .await
}

/// Executes the production operation stack with an explicit typed post-work boundary.
pub async fn execute_operation_with_post_work<P: OperationPostWork>(
    root: &OperationRoot,
    request: &OperationRequest,
    schema: &[u8],
    descriptor: &EngineSourceDescriptor,
    cancel: &Cancellation,
    post_work: &P,
) -> Result<ExecutionResult, EngineDiagnostic> {
    validate_request(request, descriptor)?;
    let target = compiler_target(&request.target)?;
    validate_schema_identity(&request.visible_schema, schema)?;

    let module = request
        .module
        .as_ref()
        .map(|module| {
            Ok(compiler::ModuleProjectionInput {
                name: module.name.to_string(),
                original_name: module.original_name.to_string(),
                source_subpath: compiler_path(&module.source_subpath)?,
                source_digest: module.source_digest.to_string(),
            })
        })
        .transpose()?;
    let output = compiler_path(&request.output_root)?;
    let dependency = compiler_dependency(&request.sdk_dependency);
    let authoring = if matches!(
        request.operation,
        compiler::OperationKind::GenerateModule | compiler::OperationKind::GenerateEntrypoint
    ) {
        Some(module_authoring_input(root, request, descriptor)?)
    } else {
        None
    };
    let projection_request = compiler::OperationProjectionRequest {
        target: &target,
        operation: request.operation,
        visible_schema_json: schema,
        module: module.as_ref(),
        output: &output,
        sdk_dependency: &dependency,
        authoring: authoring.as_ref(),
        engine_version: request.engine_version.as_ref().map(EngineRelease::as_str),
    };
    let projected = compiler::project_operation(projection_request).map_err(compiler_diagnostic)?;
    let consumed_schema_digest: Sha256Digest = projected
        .schema()
        .digest()
        .as_str()
        .parse()
        .map_err(|_| generation_failed())?;
    let mut vcs_generated = projected
        .vcs_generated()
        .iter()
        .map(engine_path)
        .collect::<Result<BTreeSet<_>, _>>()?;
    let vcs_ignored = projected
        .vcs_ignored()
        .iter()
        .map(engine_path)
        .collect::<Result<BTreeSet<_>, _>>()?;

    let temporary = tempfile::tempdir().map_err(|_| generation_failed())?;
    for (path, artifact) in projected.artifacts() {
        let path = engine_path(path)?;
        let absolute = path.join_lexically(temporary.path());
        if let Some(parent) = absolute.parent() {
            fs::create_dir_all(parent).map_err(|_| generation_failed())?;
        }
        fs::write(absolute, &artifact.content).map_err(|_| generation_failed())?;
    }
    let staging_root = OperationRoot::open(temporary.path())?;
    if projected.projection_pass_limit() > 2 {
        return Err(diagnostic(
            EngineDiagnosticCode::PostWorkRejected,
            "operation.projection",
            "pure compiler requested more than two projection passes",
        ));
    }
    let mut post_work_records = Vec::new();
    for work in projected.post_work() {
        let compiler::PostWorkPlan::FormatRust { files } = work;
        let files = files
            .iter()
            .map(engine_path)
            .collect::<Result<BTreeSet<_>, _>>()?;
        post_work_records.push(
            format_rust(
                post_work,
                &staging_root,
                &request.target.rust_toolchain,
                files,
                cancel,
            )
            .await?,
        );
    }

    let artifacts = projected
        .artifacts()
        .iter()
        .map(|(path, artifact)| {
            let path = engine_path(path)?;
            let content = staging_root.read(&path)?;
            let kind = match artifact.kind {
                compiler::CandidateArtifactKind::RustSource => ArtifactKind::RustSource,
                compiler::CandidateArtifactKind::CargoManifest => ArtifactKind::CargoManifest,
                compiler::CandidateArtifactKind::ControlManifest => ArtifactKind::ControlManifest,
            };
            Ok((
                path,
                CandidateArtifact {
                    kind,
                    content,
                    ownership: ArtifactOwnership::Generator,
                },
            ))
        })
        .collect::<Result<BTreeMap<_, _>, EngineDiagnostic>>()?;
    let artifact_records = artifacts
        .iter()
        .map(|(path, artifact)| {
            (
                path.clone(),
                ArtifactRecord {
                    kind: artifact.kind,
                    digest: digest(&artifact.content),
                    ownership: artifact.ownership,
                },
            )
        })
        .collect();
    let manifest_path = RelativeOperationPath::parse(&format!(
        "{}/.dagger/rust/operation-manifest.json",
        request.output_root.as_str()
    ))
    .map_err(|_| generation_failed())?;
    vcs_generated.insert(manifest_path.clone());
    validate_path_sets(&vcs_generated, &vcs_ignored)?;
    let previous_bytes = root
        .exists(&manifest_path)
        .then(|| root.read(&manifest_path))
        .transpose()?;
    let previous = previous_bytes
        .as_deref()
        .map(crate::decode_canonical::<OperationManifest>)
        .transpose()
        .map_err(|_| {
            diagnostic(
                EngineDiagnosticCode::OperationManifestStale,
                manifest_path.as_str(),
                "prior operation manifest is not canonical or is incompatible",
            )
        })?;
    let manifest = OperationManifest {
        format_version: crate::FormatVersion,
        operation: request.operation,
        mode: if request
            .module
            .as_ref()
            .is_some_and(|module| module.config_format == ModuleConfigFormat::Legacy)
        {
            GenerationMode::LegacyRuntimeCodegen
        } else {
            GenerationMode::CheckedGenerated
        },
        target: request.target.clone(),
        input_digest: manifest_input_digest(request, &consumed_schema_digest)?,
        visible_schema_digest: consumed_schema_digest,
        module_source_digest: request
            .module
            .as_ref()
            .map(|module| module.source_digest.clone()),
        sdk_dependency: request.sdk_dependency.clone(),
        output_root: request.output_root.clone(),
        artifacts: artifact_records,
        post_work: post_work_records,
        generator: GeneratorIdentity {
            version: ExactVersion::new(env!("CARGO_PKG_VERSION").to_owned())
                .expect("package version must be exact semantic version"),
            engine_source_digest: descriptor
                .generator_digest()
                .map_err(|_| generation_failed())?,
        },
    };
    let candidate = OperationCandidate {
        artifacts,
        manifest,
        manifest_path: manifest_path.clone(),
        previous_manifest_digest: previous_bytes.as_deref().map(digest),
    };
    let publication = verify_ownership(root, previous.as_ref(), &candidate)?;
    let published = publish(root, publication)?;
    Ok(ExecutionResult {
        format_version: FormatVersion,
        kind: ExecutionResultKind::Generation,
        output_root: request.output_root.clone(),
        touched_paths: published
            .changes
            .iter()
            .map(|change| change.path.clone())
            .chain(published.manifest_changed.then(|| manifest_path.clone()))
            .collect(),
        removed_paths: Default::default(),
        operation_manifest: Some(manifest_path),
        vcs_generated,
        vcs_ignored,
    })
}

/// The request digest an operation manifest records, taken over what generation
/// consumed rather than over what the engine sent.
///
/// The schema is identified by the consumed schema's semantic identity, not by the
/// engine's schema bytes: a later engine that serves the same reviewed core contract
/// with reworded documentation or unconsumed additions yields the same digest, so
/// regenerating on it leaves the manifest unchanged. The engine's stated release only
/// selects diagnostic advice and is left out.
///
/// A module's resolved pin names the commit the engine loaded its source from, so it
/// is set for a workspace loaded from a Git reference and absent for a checkout of the
/// same commit, while the module's source digest already binds what generation read.
/// The pin is therefore left out.
fn manifest_input_digest(
    request: &OperationRequest,
    consumed_schema_digest: &Sha256Digest,
) -> Result<Sha256Digest, EngineDiagnostic> {
    let mut consumed = request.clone();
    consumed.visible_schema.digest = consumed_schema_digest.clone();
    consumed.engine_version = None;
    if let Some(module) = consumed.module.as_mut() {
        module.resolved_pin = None;
    }
    canonical_digest(DigestDomain::OperationRequest, &consumed).map_err(|_| generation_failed())
}

async fn first_failing_format_path<P: OperationPostWork>(
    post_work: &P,
    root: &OperationRoot,
    plan: &PostWorkPlan,
    environment: &BTreeMap<String, String>,
    cancel: &Cancellation,
) -> Result<Option<String>, EngineDiagnostic> {
    let PostWorkPlan::FormatRust { toolchain, files } = plan else {
        return Ok(None);
    };
    // Formatter stderr may contain caller-authored source. A failure-only replay of
    // one normalized generated path at a time identifies the repair coordinate
    // without forwarding process output across the private adapter boundary.
    for file in files {
        let single = PostWorkPlan::FormatRust {
            toolchain: toolchain.clone(),
            files: BTreeSet::from([file.clone()]),
        };
        if !post_work
            .execute(root, &single, environment, cancel)
            .await?
            .success
        {
            return Ok(Some(file.to_string()));
        }
    }
    Ok(None)
}

// Engine operations receive the core schema plus module-scoped extensions. The pure
// compiler validates every reviewed core coordinate before rendering; this boundary
// separately proves that the mounted bytes are exactly those named by the request.
fn validate_schema_identity(
    input: &crate::SchemaInput,
    schema: &[u8],
) -> Result<(), EngineDiagnostic> {
    if digest(schema) != input.digest {
        return Err(diagnostic(
            EngineDiagnosticCode::OperationInputInvalid,
            "request.visible_schema.digest",
            "schema bytes differ from the operation request digest",
        ));
    }
    Ok(())
}

fn compiler_diagnostic(diagnostics: CompilerDiagnosticSet) -> EngineDiagnostic {
    let primary = &diagnostics.diagnostics()[0];
    EngineDiagnostic::new(
        EngineDiagnosticCode::GenerationFailed,
        Some(
            primary
                .coordinate
                .as_ref()
                .map_or("visible-schema", |coordinate| coordinate.as_str()),
        ),
        format!("{}: {}", primary.code, primary.message),
    )
}

/// Returns the checked compiler target, which a request must name exactly.
pub(crate) fn compiler_target(target: &TargetIdentity) -> Result<CodegenTarget, EngineDiagnostic> {
    let compiler_target = CodegenTarget::decode_exact(TARGET_DESCRIPTOR).map_err(|_| {
        diagnostic(
            EngineDiagnosticCode::OperationInputInvalid,
            "request.target",
            "private compiler target descriptor is invalid",
        )
    })?;
    if target.repository.as_str() != format!("https://{}", compiler_target.dagger_repository())
        || target.dagger_revision.as_str() != compiler_target.dagger_revision().as_str()
        || target.engine_version.as_str() != compiler_target.engine_version().to_string()
        || target.rust_sdk_version.as_str() != compiler_target.rust_sdk_version().to_string()
        || target.rust_toolchain.as_str() != compiler_target.rust_version().to_string()
        || target.core_schema_digest.as_str() != compiler_target.schema_digest().to_string()
    {
        return Err(diagnostic(
            EngineDiagnosticCode::OperationInputInvalid,
            "request.target",
            "operation target differs from the checked private compiler target",
        ));
    }
    Ok(compiler_target)
}

/// Formats staged generated Rust with the pinned formatter, twice, and requires the
/// second pass to change nothing.
pub(crate) async fn format_rust<P: OperationPostWork>(
    post_work: &P,
    staging_root: &OperationRoot,
    toolchain: &ExactRustToolchain,
    files: BTreeSet<RelativeOperationPath>,
    cancel: &Cancellation,
) -> Result<PostWorkRecord, EngineDiagnostic> {
    let plan = PostWorkPlan::FormatRust {
        toolchain: toolchain.clone(),
        files: files.clone(),
    };
    let environment = current_allowlisted_environment();
    let outcome = post_work
        .execute(staging_root, &plan, &environment, cancel)
        .await?;
    if !outcome.success {
        let coordinate =
            first_failing_format_path(post_work, staging_root, &plan, &environment, cancel)
                .await?
                .unwrap_or_else(|| "post-work.rustfmt".to_owned());
        return Err(diagnostic(
            EngineDiagnosticCode::FormatFailed,
            &coordinate,
            "pinned rustfmt rejected generated Rust source",
        ));
    }
    let first_digest = digest_path_set(staging_root, &files)?;
    let convergence = post_work
        .execute(staging_root, &plan, &environment, cancel)
        .await?;
    if !convergence.success {
        return Err(diagnostic(
            EngineDiagnosticCode::FormatFailed,
            "post-work.rustfmt",
            "pinned rustfmt rejected its convergence pass",
        ));
    }
    let second_digest = digest_path_set(staging_root, &files)?;
    require_convergence(&[first_digest, second_digest.clone()])?;
    Ok(PostWorkRecord {
        plan,
        result_digest: second_digest,
    })
}

/// Validates descriptor/request agreement and operation-specific input shape.
pub fn validate_request(
    request: &OperationRequest,
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
    let module_required = matches!(
        request.operation,
        compiler::OperationKind::GenerateModule | compiler::OperationKind::GenerateEntrypoint
    );
    if request.module.is_some() != module_required {
        return Err(diagnostic(
            EngineDiagnosticCode::OperationInputInvalid,
            "request.module",
            "operation has a missing or forbidden module input",
        ));
    }
    Ok(())
}

fn compiler_path(
    path: &RelativeOperationPath,
) -> Result<compiler::RelativeOperationPath, EngineDiagnostic> {
    compiler::RelativeOperationPath::parse(path.as_str()).map_err(|_| generation_failed())
}

fn engine_path(
    path: &compiler::RelativeOperationPath,
) -> Result<RelativeOperationPath, EngineDiagnostic> {
    RelativeOperationPath::parse(path.as_str()).map_err(|_| generation_failed())
}

fn compiler_dependency(dependency: &PublishedSdkDependency) -> compiler::PublishedSdkDependency {
    match dependency {
        PublishedSdkDependency::Registry {
            registry,
            exact_version,
            ..
        } => compiler::PublishedSdkDependency::Registry {
            registry: registry.to_string(),
            exact_version: exact_version.to_string(),
        },
        PublishedSdkDependency::Git { url, revision, .. } => {
            compiler::PublishedSdkDependency::Git {
                url: url.to_string(),
                revision: revision.to_string(),
            }
        }
    }
}

fn module_authoring_input(
    root: &OperationRoot,
    request: &OperationRequest,
    descriptor: &EngineSourceDescriptor,
) -> Result<compiler::ModuleAuthoringInput, EngineDiagnostic> {
    let module = request.module.as_ref().ok_or_else(|| {
        diagnostic(
            EngineDiagnosticCode::OperationInputInvalid,
            "request.module",
            "module authoring requires one selected module source",
        )
    })?;
    let manifest_path = child_path(&module.source_subpath, "Cargo.toml")?;
    let manifest_bytes = root.read(&manifest_path)?;
    let manifest_text = std::str::from_utf8(&manifest_bytes).map_err(|_| {
        diagnostic(
            EngineDiagnosticCode::CargoManifestInvalid,
            manifest_path.as_str(),
            "selected Cargo manifest is not UTF-8",
        )
    })?;
    let manifest = manifest_text
        .parse::<toml_edit::DocumentMut>()
        .map_err(|_| {
            diagnostic(
                EngineDiagnosticCode::CargoManifestInvalid,
                manifest_path.as_str(),
                "selected Cargo manifest is invalid TOML",
            )
        })?;
    let package = manifest
        .get("package")
        .and_then(toml_edit::Item::as_table_like)
        .ok_or_else(|| {
            diagnostic(
                EngineDiagnosticCode::CargoManifestInvalid,
                manifest_path.as_str(),
                "selected Cargo manifest has no package table",
            )
        })?;
    let package_name = package
        .get("name")
        .and_then(toml_edit::Item::as_str)
        .ok_or_else(|| {
            diagnostic(
                EngineDiagnosticCode::CargoManifestInvalid,
                manifest_path.as_str(),
                "selected Cargo package has no name",
            )
        })?;
    let edition = package
        .get("edition")
        .and_then(toml_edit::Item::as_str)
        .ok_or_else(|| {
            diagnostic(
                EngineDiagnosticCode::CargoManifestInvalid,
                manifest_path.as_str(),
                "selected Cargo package has no explicit edition",
            )
        })?;
    let crate_root_suffix = manifest
        .get("lib")
        .and_then(toml_edit::Item::as_table_like)
        .and_then(|library| library.get("path"))
        .and_then(toml_edit::Item::as_str)
        .unwrap_or("src/lib.rs");
    let crate_root = child_path(&module.source_subpath, crate_root_suffix)?;
    let sdk_dependency_alias = sdk_dependency_alias(&manifest).ok_or_else(|| {
        diagnostic(
            EngineDiagnosticCode::CargoManifestInvalid,
            manifest_path.as_str(),
            "selected Cargo package has no tokeira-dagger-sdk dependency",
        )
    })?;
    let cargo_package = crate::CargoPackage {
        package_id: crate::StableCoordinate::new(format!(
            "{package_name}@{}",
            module.source_subpath.as_str()
        ))
        .map_err(|_| generation_failed())?,
        name: crate::StableCoordinate::new(package_name).map_err(|_| generation_failed())?,
        manifest_path,
        package_root: module.source_subpath.clone(),
    };
    let source = SourceSnapshotBuilder::new().build(
        root,
        SourceSnapshotRequest {
            package: &cargo_package,
            crate_root: &crate_root,
            edition,
            // The absence of selected cfg values is itself explicit; build-script and
            // ambient host cfgs cannot silently enter authoring semantics.
            cfg: dagger_codegen::module::CfgEnvironment {
                values: BTreeMap::new(),
                features: BTreeSet::new(),
            },
        },
    )?;
    let generator_digest = dagger_codegen::module::Sha256Digest::new(
        descriptor
            .generator_digest()
            .map_err(|_| generation_failed())?
            .as_str(),
    )
    .map_err(|_| generation_failed())?;
    Ok(compiler::ModuleAuthoringInput {
        source,
        generator_digest,
        sdk_dependency_alias,
    })
}

fn child_path(
    root: &RelativeOperationPath,
    suffix: &str,
) -> Result<RelativeOperationPath, EngineDiagnostic> {
    RelativeOperationPath::parse(&format!(
        "{}/{}",
        root.as_str().trim_end_matches('/'),
        suffix.trim_start_matches('/')
    ))
    .map_err(|_| {
        diagnostic(
            EngineDiagnosticCode::OutputPathEscape,
            root.as_str(),
            "module authoring path is not confined",
        )
    })
}

fn sdk_dependency_alias(manifest: &toml_edit::DocumentMut) -> Option<String> {
    let dependencies = manifest.get("dependencies")?.as_table_like()?;
    dependencies.iter().find_map(|(key, dependency)| {
        match dependency.get("package").and_then(toml_edit::Item::as_str) {
            // A renamed dependency reaches code under its key.
            Some(package) => (package == crate::SDK_PACKAGE).then(|| key.replace('-', "_")),
            // An unrenamed dependency reaches code under the package's library name.
            None => (key == crate::SDK_PACKAGE).then(|| crate::SDK_LIBRARY.to_owned()),
        }
    })
}

fn digest_path_set(
    root: &OperationRoot,
    paths: &BTreeSet<RelativeOperationPath>,
) -> Result<Sha256Digest, EngineDiagnostic> {
    let mut hasher = Sha256::new();
    for path in paths {
        hasher.update(path.as_str().as_bytes());
        hasher.update([0]);
        hasher.update(root.read(path)?);
        hasher.update([0]);
    }
    format!("sha256:{:x}", hasher.finalize())
        .parse()
        .map_err(|_| generation_failed())
}

pub(crate) fn digest(bytes: &[u8]) -> Sha256Digest {
    format!("sha256:{:x}", Sha256::digest(bytes))
        .parse()
        .expect("SHA-256 formatting must satisfy the digest scalar")
}

fn generation_failed() -> EngineDiagnostic {
    diagnostic(
        EngineDiagnosticCode::GenerationFailed,
        "operation",
        "operation candidate could not be constructed",
    )
}

fn diagnostic(code: EngineDiagnosticCode, coordinate: &str, message: &str) -> EngineDiagnostic {
    EngineDiagnostic::new(code, Some(coordinate), message)
}

#[cfg(test)]
mod tests {
    use super::{digest, validate_schema_identity};
    use crate::{RelativeOperationPath, SchemaInput};

    #[test]
    fn mounted_visible_schema_is_bound_by_digest_without_requiring_core_only_bytes() {
        let schema = br#"{"__schema":{"types":[{"name":"ModuleExtension"}]}}"#;
        let input = SchemaInput {
            path: RelativeOperationPath::parse("schema.json").expect("fixture path must parse"),
            digest: digest(schema),
        };

        validate_schema_identity(&input, schema)
            .expect("named bytes must pass identity validation");
        assert!(validate_schema_identity(&input, b"{}").is_err());
    }
}
