//! Production rendering from one pure module-authoring compilation.
//!
//! The visible schema contributes only checked generated-handle bindings: the SDK's
//! own for the reviewed Core API, typed ones rendered into the module for the types of
//! the modules it depends on, and untyped ones for anything else. Authored source,
//! descriptor construction, registration, dispatch, and entrypoint generation remain
//! inside the Rust compiler pipeline and are published as one candidate set.

use std::collections::{BTreeMap, BTreeSet};

use crate::diagnostic::{DiagnosticCode, DiagnosticSet};
use crate::module::{
    GENERATED_SUPPORT_MODULES, GENERATED_SUPPORT_NAMES, GeneratedAssetPath, GeneratedTypeBinding,
    GeneratedTypeKind, GeneratedTypeRegistry, MODULE_QUERY_METHODS, ModuleCompilation,
    ModuleCompilationRequest, ModuleCompiler, ModuleDiagnosticSet, ModuleTarget,
    Sha256Digest as ModuleDigest, TargetValue, WireName,
};
use crate::naming::NameContext;
use crate::projection::types::{ScalarKind, TypeProjection};

use super::dependency::{BindingSurface, constructor_fields, method_name};
use super::model::{CandidateArtifactKind, ContentDomain, PostWorkPlan, operation_diagnostic};
use super::render::{rust_source_paths, visible_binding_artifacts};
use super::renderers::{EntrypointRenderInput, ModuleRenderInput, RendererOutput};
use super::{CargoBinaryTarget, RelativeOperationPath};

pub(crate) fn render(input: ModuleRenderInput<'_>) -> Result<RendererOutput, DiagnosticSet> {
    let compilation = compile_authoring(input.target, input.schema, input.authoring)?;
    render_compilation(input.output, compilation, false)
}

pub(crate) fn render_entrypoint(
    input: EntrypointRenderInput<'_>,
) -> Result<RendererOutput, DiagnosticSet> {
    let compilation = compile_authoring(input.target, input.schema, input.authoring)?;
    render_compilation(input.output, compilation, true)
}

fn compile_authoring(
    target: &crate::target::CodegenTarget,
    schema: &super::visible::VisibleSchemaPlan,
    authoring: &super::ModuleAuthoringInput,
) -> Result<ModuleCompilation, DiagnosticSet> {
    let visible_digest = ModuleDigest::new(schema.digest().as_str()).map_err(|_| failure())?;
    let module_target = ModuleTarget {
        dagger_revision: target_value(target.dagger_revision().as_str())?,
        engine_version: target_value(&target.engine_version().to_string())?,
        rust_sdk_version: target_value(&target.rust_sdk_version().to_string())?,
        rust_toolchain: target_value(&target.rust_version().to_string())?,
        rust_edition: target_value(target.rust_edition().as_str())?,
        visible_schema_digest: visible_digest.clone(),
    };
    let generated_types = generated_type_registry(
        schema,
        &authoring.sdk_dependency_alias,
        visible_digest.clone(),
    )?;
    let visible_type_names = schema
        .projection()
        .named_types()
        .keys()
        // `Query` is the target registration root, not a local authored type.
        .filter(|name| name.as_str() != "Query")
        .map(|name| WireName::new(name.as_str()).map_err(|_| failure()))
        .collect::<Result<BTreeSet<_>, _>>()?;
    refuse_support_collisions(schema)?;
    let checked_bindings = checked_binding_assets(schema, &authoring.sdk_dependency_alias)?;

    ModuleCompiler::compile(ModuleCompilationRequest {
        target: &module_target,
        source: &authoring.source,
        generated_types: &generated_types,
        visible_type_names: &visible_type_names,
        generator_digest: &authoring.generator_digest,
        sdk_dependency_alias: &authoring.sdk_dependency_alias,
        checked_bindings: &checked_bindings,
    })
    .map_err(authoring_diagnostics)
}

// Module diagnostics already passed the compiler's safe-text policy and carry only
// package-relative source coordinates, so the operation boundary keeps their code,
// explanation, remediation, and authored location instead of one generic failure.
fn authoring_diagnostics(errors: ModuleDiagnosticSet) -> DiagnosticSet {
    let diagnostics = errors
        .diagnostics()
        .iter()
        .map(|diagnostic| {
            let coordinate = diagnostic.source_coordinate().map_or_else(
                || "operation.authoring".to_owned(),
                |source| format!("{}:{}:{}", source.path.as_str(), source.line, source.column),
            );
            operation_diagnostic(
                DiagnosticCode::GeneratedProvenanceInvalid,
                &coordinate,
                &format!(
                    "{}: {}; {}",
                    diagnostic.code().external(),
                    diagnostic.message(),
                    diagnostic.remediation()
                ),
            )
        })
        .collect();
    DiagnosticSet::new(diagnostics).unwrap_or_else(failure)
}

// A type outside the reviewed Core API is bound in the module's own crate, in a file
// named after it and re-exported from `dagger_generated` beside the module's generated
// support. A type that projects to a name or a file of that support would shadow it or
// be shadowed by it, so it is refused here with its own coordinate rather than as a
// compiler error in generated code. The engine names a dependency's types after the
// name the dependency is declared under (`Namer.NamespaceObject` in core/gqlformat.go
// at the target), which is why declaring it under another name is the remedy.
fn refuse_support_collisions(
    schema: &super::visible::VisibleSchemaPlan,
) -> Result<(), DiagnosticSet> {
    let mut diagnostics = Vec::new();
    for projection in schema.projection().named_types().values() {
        let (wire_name, coordinate, rust_names, module_name) = match projection {
            TypeProjection::Object(object) => (
                &object.wire_name,
                &object.coordinate,
                vec![object.rust_name.as_str()],
                Some(object.module_name.clone()),
            ),
            TypeProjection::Interface(interface) => (
                &interface.wire_name,
                &interface.coordinate,
                vec![
                    interface.trait_name.as_str(),
                    interface.client_name.as_str(),
                ],
                Some(interface.module_name.clone()),
            ),
            TypeProjection::Enum(enumeration) => (
                &enumeration.wire_name,
                &enumeration.coordinate,
                vec![enumeration.rust_name.as_str()],
                schema
                    .projection()
                    .names()
                    .get(&enumeration.coordinate, NameContext::Module)
                    .map(|name| name.identifier.clone()),
            ),
            TypeProjection::InputObject(input) => (
                &input.wire_name,
                &input.coordinate,
                vec![input.rust_name.as_str()],
                Some(input.module_name.clone()),
            ),
            TypeProjection::Scalar(_) | TypeProjection::TargetPrivate(_) => continue,
        };
        if !schema.is_extension_type(wire_name) {
            continue;
        }
        // Only a dependency's type has a name the module's author chose.
        let (owner, remedy) = schema.dependency_module(wire_name).map_or_else(
            || {
                (
                    "the visible schema".to_owned(),
                    "the module cannot be generated against this schema",
                )
            },
            |module| {
                (
                    format!("module dependency `{module}`"),
                    "declare the dependency under another name",
                )
            },
        );
        for rust_name in rust_names {
            if GENERATED_SUPPORT_NAMES.contains(&rust_name) {
                diagnostics.push(operation_diagnostic(
                    DiagnosticCode::RustNameCollision,
                    coordinate.as_str(),
                    &format!(
                        "type `{wire_name}` of {owner} would be bound as `{rust_name}`, which the module's generated support defines; {remedy}"
                    ),
                ));
            }
        }
        if let Some(module_name) = module_name {
            let stem = module_name.strip_prefix("r#").unwrap_or(&module_name);
            if GENERATED_SUPPORT_MODULES.contains(&stem) {
                diagnostics.push(operation_diagnostic(
                    DiagnosticCode::OperationArtifactCollision,
                    coordinate.as_str(),
                    &format!(
                        "type `{wire_name}` of {owner} would be bound in `{stem}.rs`, which holds the module's generated support; {remedy}"
                    ),
                ));
            }
        }
    }
    // A dependency's constructor becomes a method of the module's query root, named
    // after the dependency. A dependency named like a method written there by hand
    // would define that method a second time, and so would two constructors that the
    // renderer's escape of prelude method names brings to one name.
    let mut constructors = BTreeMap::new();
    for (field, target) in constructor_fields(schema) {
        let owner = schema.dependency_module(target).unwrap_or_default();
        for method in [Some(method_name(field)), field.options_method_name.clone()]
            .into_iter()
            .flatten()
        {
            let stated = method.strip_prefix("r#").unwrap_or(&method);
            if MODULE_QUERY_METHODS.contains(&method.as_str()) {
                diagnostics.push(operation_diagnostic(
                    DiagnosticCode::RustNameCollision,
                    field.coordinate.as_str(),
                    &format!(
                        "the constructor of module dependency `{owner}` would be bound as `ModuleQuery::{stated}`, which the module's query root defines; declare the dependency under another name"
                    ),
                ));
            } else if let Some(previous) = constructors.insert(method.clone(), &field.coordinate)
                && previous != &field.coordinate
            {
                diagnostics.push(operation_diagnostic(
                    DiagnosticCode::RustNameCollision,
                    field.coordinate.as_str(),
                    &format!(
                        "the constructor of module dependency `{owner}` would be bound as `ModuleQuery::{stated}`, as `{}` already is; declare the dependency under another name",
                        previous.as_str()
                    ),
                ));
            }
        }
    }
    DiagnosticSet::new(diagnostics).map_or(Ok(()), Err)
}

fn generated_type_registry(
    schema: &super::visible::VisibleSchemaPlan,
    alias: &str,
    digest: ModuleDigest,
) -> Result<GeneratedTypeRegistry, DiagnosticSet> {
    let mut bindings = Vec::new();
    for projection in schema.projection().named_types().values() {
        let (wire_name, rust_name, kind, extension_name, enum_members) = match projection {
            TypeProjection::Object(object) => (
                object.wire_name.as_str(),
                object.rust_name.as_str(),
                GeneratedTypeKind::Object,
                &object.wire_name,
                BTreeMap::new(),
            ),
            TypeProjection::Interface(interface) => (
                interface.wire_name.as_str(),
                interface.client_name.as_str(),
                GeneratedTypeKind::Interface,
                &interface.wire_name,
                BTreeMap::new(),
            ),
            // An enum needs module codecs to appear in a module's signatures or state.
            // The SDK crate implements them for Core's enums, and the binding rendered
            // for a dependency's enum carries its own. An enum outside the reviewed Core
            // API that no dependency owns is bound without them.
            TypeProjection::Enum(enumeration)
                if !schema.is_extension_type(&enumeration.wire_name)
                    || schema.is_dependency_type(&enumeration.wire_name) =>
            {
                (
                    enumeration.wire_name.as_str(),
                    enumeration.rust_name.as_str(),
                    GeneratedTypeKind::Enum,
                    &enumeration.wire_name,
                    enumeration
                        .variants
                        .values()
                        .map(|variant| (variant.rust_name.clone(), variant.wire_name.to_string()))
                        .collect(),
                )
            }
            // A module declares no scalars of its own (the scalar case of
            // `userMod.modTypeFor` in core/module.go at the target finds none), so a
            // dependency contributes none and only the SDK crate implements module
            // codecs for one.
            TypeProjection::Scalar(scalar) if !schema.is_extension_type(&scalar.wire_name) => {
                let rust_name = match scalar.scalar {
                    ScalarKind::Json => "Json",
                    ScalarKind::Platform => "Platform",
                    ScalarKind::Custom => schema
                        .projection()
                        .names()
                        .get(&scalar.coordinate, NameContext::Type)
                        .map(|name| name.identifier.as_str())
                        .ok_or_else(failure)?,
                    // Builtin scalars are Rust primitives, an `ID` is a handle's identity,
                    // and `Void` is a unit result.
                    ScalarKind::Boolean
                    | ScalarKind::Float
                    | ScalarKind::Int
                    | ScalarKind::String
                    | ScalarKind::Id
                    | ScalarKind::Void => continue,
                };
                (
                    scalar.wire_name.as_str(),
                    rust_name,
                    GeneratedTypeKind::Scalar,
                    &scalar.wire_name,
                    BTreeMap::new(),
                )
            }
            _ => continue,
        };
        let rust_path = if schema.is_extension_type(extension_name) {
            format!("crate::dagger_generated::{rust_name}")
        } else {
            format!("{alias}::{rust_name}")
        };
        bindings.push(GeneratedTypeBinding {
            rust_path,
            target_name: wire_name.to_owned(),
            visible_schema_digest: digest.clone(),
            kind,
            enum_members,
        });
    }
    GeneratedTypeRegistry::new(digest, bindings).map_err(|_| failure())
}

fn checked_binding_assets(
    schema: &super::visible::VisibleSchemaPlan,
    alias: &str,
) -> Result<BTreeMap<GeneratedAssetPath, Vec<u8>>, DiagnosticSet> {
    let root = RelativeOperationPath::parse("src/dagger_generated")?;
    visible_binding_artifacts(schema, &root, alias, BindingSurface::Module)?
        .into_iter()
        .filter(|(path, artifact)| {
            artifact.kind == CandidateArtifactKind::RustSource
                && path.as_str() != "src/dagger_generated/mod.rs"
        })
        .map(|(path, artifact)| {
            GeneratedAssetPath::new(path.as_str())
                .map(|path| (path, artifact.content))
                .map_err(|_| failure())
        })
        .collect()
}

fn render_compilation(
    output_root: &RelativeOperationPath,
    compilation: ModuleCompilation,
    entrypoint_only: bool,
) -> Result<RendererOutput, DiagnosticSet> {
    let mut output = RendererOutput::new(if entrypoint_only {
        ContentDomain::ModuleEntrypoint
    } else {
        ContentDomain::ModuleOperation
    });
    for (path, content) in compilation.assets.files {
        let is_entrypoint = path.as_str() == "src/bin/dagger-module.rs";
        if entrypoint_only && !is_entrypoint {
            continue;
        }
        let destination = output_root.join(path.as_str())?;
        let kind = if path.as_str().ends_with(".rs") {
            CandidateArtifactKind::RustSource
        } else {
            CandidateArtifactKind::ControlManifest
        };
        output.insert_artifact(destination, kind, content)?;
    }
    let rust_files = rust_source_paths(&output.artifacts);
    output
        .vcs_generated
        .extend(output.artifacts.keys().cloned());
    output
        .post_work
        .push(PostWorkPlan::FormatRust { files: rust_files });
    output.cargo_binary = Some(CargoBinaryTarget {
        name: "dagger-module".to_owned(),
        path: RelativeOperationPath::parse("src/bin/dagger-module.rs")?,
    });
    Ok(output)
}

fn target_value(value: &str) -> Result<TargetValue, DiagnosticSet> {
    TargetValue::new(value).map_err(|_| failure())
}

fn failure() -> DiagnosticSet {
    DiagnosticSet::one(operation_diagnostic(
        DiagnosticCode::GeneratedProvenanceInvalid,
        "operation.authoring",
        "Rust module authoring compilation failed",
    ))
}
