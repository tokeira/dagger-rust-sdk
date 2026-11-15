//! Closed projection policies for checked schema directives.
//!
//! A directive is never ignored because it is unfamiliar to the renderer. Active
//! applications become typed policy data, while inactive definitions remain pinned by
//! fingerprints and fail as soon as the target gives them client-visible semantics.
//!
//! The engine also serves coordinates outside the reviewed Core API: those of the
//! modules a module depends on. It applies the module-authoring directives to them, as
//! it does to any module's functions, arguments and collections. There they say how the
//! engine fills in or schedules another module's function, which changes nothing a
//! caller sends, so they are recorded and accepted. On a reviewed Core coordinate they
//! stay refused.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::diagnostic::{Diagnostic, DiagnosticCode, DiagnosticCoordinate, DiagnosticSet};
use crate::projection::catalog::SemanticDigest;
use crate::schema::CoreCoordinateManifest;
use crate::schema::canonical::{
    CanonicalSchema, DirectiveApplication, EnumDefinition, SchemaCoordinate, SchemaName,
    TypeDefinition,
};

/// The registered policy for one target directive definition.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub enum DirectivePolicy {
    /// Typed ID inputs and self-return IDs.
    ExpectedType,
    /// Compiler deprecation plus caller-visible documentation.
    Deprecated,
    /// Caller-visible stability documentation without a feature gate.
    Experimental,
    /// A schema enum spelling that aliases a canonical sibling value.
    EnumValueAlias,
    /// Engine-authored module ownership metadata with no public client behaviour.
    SourceMap,
    /// Fields whose every evaluation may give a new result or repeat a side effect.
    /// The target's contract forbids reusing an ID fetched for an object built through
    /// such a field; generated handles fetch their ID on every use and never memoize
    /// one, so every application is satisfied by construction.
    Reevaluate,
    /// Engine workflow classification of core functions (`@check`, `@generate`): it
    /// tells the engine's artifact discovery which fields `dagger check` and
    /// `dagger generate` run, and gives a client call no different semantics.
    WorkflowRole,
    /// Module-authoring metadata that no reviewed Core coordinate applies. The engine
    /// applies it only to a module's own coordinates, which a schema carries for the
    /// modules a module depends on.
    TargetInactive,
}

impl DirectivePolicy {
    /// Returns whether a reviewed Core coordinate may apply a directive of this policy.
    #[must_use]
    pub const fn accepts_applications(&self) -> bool {
        !matches!(self, Self::TargetInactive)
    }
}

/// Typed meaning of one active directive application.
#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub enum DirectiveApplicationPolicy {
    /// Expected object or interface target.
    ExpectedType { target: SchemaName },
    /// Engine-authored deprecation reason.
    Deprecated { reason: String },
    /// Engine-authored experimental stability note.
    Experimental { reason: String },
    /// Canonical enum value selected by an alias wire name.
    EnumValueAlias { canonical: SchemaName },
    /// Module identity attached by the engine's dynamic schema merger.
    SourceMap { module: String },
    /// Fresh evaluation per use, independent of the `when` argument because generated
    /// handles never memoize IDs.
    Reevaluate,
    /// The workflow role (`check` or `generate`) the engine assigns the field.
    WorkflowRole { role: SchemaName },
    /// A module-authoring directive the engine applies to a coordinate outside the
    /// reviewed Core API, where it tells the engine how to fill in or schedule another
    /// module's function and gives a client call no different semantics.
    ModuleAuthoring { directive: SchemaName },
}

/// One definition and all of its validated target applications.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct DirectivePolicyRecord {
    /// Exact directive wire name.
    pub name: SchemaName,
    /// Exact definition coordinate.
    pub coordinate: SchemaCoordinate,
    /// Semantic definition fingerprint retained for drift review.
    pub definition_fingerprint: SemanticDigest,
    /// Closed projection policy.
    pub policy: DirectivePolicy,
    /// Typed applications in exact coordinate order.
    pub applications: BTreeMap<SchemaCoordinate, DirectiveApplicationPolicy>,
}

/// Complete checked-target directive policies.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct DirectiveProjection {
    records: BTreeMap<SchemaName, DirectivePolicyRecord>,
    applications: BTreeMap<SchemaCoordinate, Vec<DirectiveApplicationPolicy>>,
}

impl DirectiveProjection {
    /// Borrows all directive records in wire name order.
    #[must_use]
    pub const fn records(&self) -> &BTreeMap<SchemaName, DirectivePolicyRecord> {
        &self.records
    }

    /// Borrows all active policies attached to one schema coordinate.
    #[must_use]
    pub fn for_coordinate(&self, coordinate: &SchemaCoordinate) -> &[DirectiveApplicationPolicy] {
        self.applications
            .get(coordinate)
            .map(Vec::as_slice)
            .unwrap_or_default()
    }

    /// Returns the expected type attached to a field or argument.
    #[must_use]
    pub fn expected_type(&self, coordinate: &SchemaCoordinate) -> Option<&SchemaName> {
        self.for_coordinate(coordinate).iter().find_map(|policy| {
            if let DirectiveApplicationPolicy::ExpectedType { target } = policy {
                Some(target)
            } else {
                None
            }
        })
    }

    /// Returns the canonical value named by an enum alias coordinate.
    #[must_use]
    pub fn enum_alias(&self, coordinate: &SchemaCoordinate) -> Option<&SchemaName> {
        self.for_coordinate(coordinate).iter().find_map(|policy| {
            if let DirectiveApplicationPolicy::EnumValueAlias { canonical } = policy {
                Some(canonical)
            } else {
                None
            }
        })
    }

    /// Returns deprecation metadata attached to a public coordinate.
    #[must_use]
    pub fn deprecation_reason(&self, coordinate: &SchemaCoordinate) -> Option<&str> {
        self.for_coordinate(coordinate).iter().find_map(|policy| {
            if let DirectiveApplicationPolicy::Deprecated { reason } = policy {
                Some(reason.as_str())
            } else {
                None
            }
        })
    }

    /// Returns the experimental stability note attached to a public coordinate.
    #[must_use]
    pub fn experimental_reason(&self, coordinate: &SchemaCoordinate) -> Option<&str> {
        self.for_coordinate(coordinate).iter().find_map(|policy| {
            if let DirectiveApplicationPolicy::Experimental { reason } = policy {
                Some(reason.as_str())
            } else {
                None
            }
        })
    }
}

/// Projects every directive definition and application of `schema`. `core` is the
/// reviewed Core API the schema was verified against, whose coordinates may not apply a
/// module-authoring directive; without one, the whole schema is reviewed Core.
pub(crate) fn project_directives(
    schema: &CanonicalSchema,
    core: Option<&CoreCoordinateManifest>,
) -> Result<DirectiveProjection, DiagnosticSet> {
    let is_reviewed_core =
        |coordinate: &SchemaCoordinate| core.is_none_or(|core| core.contains(coordinate));
    let sites = application_sites(schema);
    let mut records = BTreeMap::new();
    let mut applications = BTreeMap::<SchemaCoordinate, Vec<DirectiveApplicationPolicy>>::new();
    let mut diagnostics = Vec::new();

    for (name, definition) in schema.directives() {
        let policy = match name.as_str() {
            "expectedType" => DirectivePolicy::ExpectedType,
            "deprecated" => DirectivePolicy::Deprecated,
            "experimental" => DirectivePolicy::Experimental,
            "enumValue" => DirectivePolicy::EnumValueAlias,
            "sourceMap" => DirectivePolicy::SourceMap,
            "reevaluate" => DirectivePolicy::Reevaluate,
            // v1.0.0-beta.18 applies these to core fields (`Generator.stale`,
            // `WorkspaceSDK.generate`) for the engine's artifact discovery.
            "check" | "generate" => DirectivePolicy::WorkflowRole,
            // "agent" (v1.0.0-beta.11): marks module functions as agent middleware.
            // "collection", "delta", "get", "keys" (v1.0.0-beta.18): module collection
            // authoring markers. All are definition-only at the exact target — no
            // core-schema coordinate applies them — so an application appearing on a
            // core coordinate in a future target fails closed via
            // TargetInactiveDirectiveChanged until its client semantics are decided.
            // The engine applies them to the coordinates of a module's dependencies
            // (`Function.Directives` and `FunctionArg.Directives` in core/typedef.go,
            // `ModuleObject.TypeDefinition` in core/object.go), where they are accepted.
            "agent" | "cache" | "collection" | "defaultAddress" | "defaultPath" | "delta"
            | "get" | "ignorePatterns" | "keys" | "up" => DirectivePolicy::TargetInactive,
            _ => {
                diagnostics.push(diagnostic(
                    DiagnosticCode::SchemaDirectiveUnmapped,
                    &definition.coordinate,
                    "directive has no registered core-client projection policy",
                ));
                continue;
            }
        };
        let definition_fingerprint = match SemanticDigest::for_value(definition) {
            Ok(fingerprint) => fingerprint,
            Err(_) => {
                diagnostics.push(diagnostic(
                    DiagnosticCode::SchemaDirectiveUnmapped,
                    &definition.coordinate,
                    "directive definition could not be fingerprinted",
                ));
                continue;
            }
        };
        let mut typed = BTreeMap::new();
        for site in sites.iter().filter(|site| site.application.name == *name) {
            if !policy.accepts_applications() && is_reviewed_core(&site.coordinate) {
                diagnostics.push(diagnostic(
                    DiagnosticCode::TargetInactiveDirectiveChanged,
                    &site.coordinate,
                    "target-inactive directive gained a core-client application",
                ));
                continue;
            }
            match project_application(schema, site, &policy) {
                Ok(application) => {
                    typed.insert(site.coordinate.clone(), application.clone());
                    applications
                        .entry(site.coordinate.clone())
                        .or_default()
                        .push(application);
                }
                Err(error) => diagnostics.push(error),
            }
        }
        records.insert(
            name.clone(),
            DirectivePolicyRecord {
                name: name.clone(),
                coordinate: definition.coordinate.clone(),
                definition_fingerprint,
                policy,
                applications: typed,
            },
        );
    }

    for policies in applications.values_mut() {
        policies.sort();
    }
    if let Some(diagnostics) = DiagnosticSet::new(diagnostics) {
        return Err(diagnostics);
    }
    Ok(DirectiveProjection {
        records,
        applications,
    })
}

struct ApplicationSite<'a> {
    coordinate: SchemaCoordinate,
    application: &'a DirectiveApplication,
    enum_definition: Option<&'a EnumDefinition>,
}

fn application_sites(schema: &CanonicalSchema) -> Vec<ApplicationSite<'_>> {
    let mut sites = Vec::new();
    for definition in schema.types().values() {
        match definition {
            TypeDefinition::Scalar(scalar) => {
                push_sites(&mut sites, &scalar.coordinate, &scalar.directives, None);
            }
            TypeDefinition::Object(object) => {
                push_sites(&mut sites, &object.coordinate, &object.directives, None);
                for field in object.fields.values() {
                    push_sites(&mut sites, &field.coordinate, &field.directives, None);
                    for argument in field.arguments.values() {
                        push_sites(&mut sites, &argument.coordinate, &argument.directives, None);
                    }
                }
            }
            TypeDefinition::Interface(interface) => {
                push_sites(
                    &mut sites,
                    &interface.coordinate,
                    &interface.directives,
                    None,
                );
                for field in interface.fields.values() {
                    push_sites(&mut sites, &field.coordinate, &field.directives, None);
                    for argument in field.arguments.values() {
                        push_sites(&mut sites, &argument.coordinate, &argument.directives, None);
                    }
                }
            }
            TypeDefinition::Enum(enumeration) => {
                push_sites(
                    &mut sites,
                    &enumeration.coordinate,
                    &enumeration.directives,
                    Some(enumeration),
                );
                for value in enumeration.values.values() {
                    push_sites(
                        &mut sites,
                        &value.coordinate,
                        &value.directives,
                        Some(enumeration),
                    );
                }
            }
            TypeDefinition::InputObject(input) => {
                push_sites(&mut sites, &input.coordinate, &input.directives, None);
                for field in input.fields.values() {
                    push_sites(&mut sites, &field.coordinate, &field.directives, None);
                }
            }
        }
    }
    sites
}

fn push_sites<'a>(
    sites: &mut Vec<ApplicationSite<'a>>,
    coordinate: &SchemaCoordinate,
    directives: &'a [DirectiveApplication],
    enum_definition: Option<&'a EnumDefinition>,
) {
    sites.extend(directives.iter().map(|application| ApplicationSite {
        coordinate: coordinate.clone(),
        application,
        enum_definition,
    }));
}

fn project_application(
    schema: &CanonicalSchema,
    site: &ApplicationSite<'_>,
    policy: &DirectivePolicy,
) -> Result<DirectiveApplicationPolicy, Diagnostic> {
    match policy {
        DirectivePolicy::ExpectedType => {
            let target = string_argument(site, "name", None, DiagnosticCode::ExpectedTypeInvalid)?;
            let target = SchemaName::try_from(target.as_str()).map_err(|()| {
                diagnostic(
                    DiagnosticCode::ExpectedTypeInvalid,
                    &site.coordinate,
                    "expectedType target is not a valid wire name",
                )
            })?;
            if !matches!(
                schema.types().get(&target),
                Some(TypeDefinition::Object(_) | TypeDefinition::Interface(_))
            ) {
                return Err(diagnostic(
                    DiagnosticCode::ExpectedTypeInvalid,
                    &site.coordinate,
                    "expectedType target is not a public object or interface",
                ));
            }
            Ok(DirectiveApplicationPolicy::ExpectedType { target })
        }
        DirectivePolicy::Deprecated => Ok(DirectiveApplicationPolicy::Deprecated {
            reason: string_argument(
                site,
                "reason",
                Some("No longer supported"),
                DiagnosticCode::DeprecationDirectiveInvalid,
            )?,
        }),
        DirectivePolicy::Experimental => Ok(DirectiveApplicationPolicy::Experimental {
            reason: string_argument(
                site,
                "reason",
                Some("Not stabilized"),
                DiagnosticCode::ExperimentalDirectiveInvalid,
            )?,
        }),
        DirectivePolicy::EnumValueAlias => {
            let canonical = string_argument(
                site,
                "value",
                None,
                DiagnosticCode::SchemaDirectiveArgumentInvalid,
            )?;
            let canonical = SchemaName::try_from(canonical.as_str()).map_err(|()| {
                diagnostic(
                    DiagnosticCode::SchemaDirectiveArgumentInvalid,
                    &site.coordinate,
                    "enumValue target is not a valid wire name",
                )
            })?;
            let Some(enumeration) = site.enum_definition else {
                return Err(diagnostic(
                    DiagnosticCode::SchemaDirectiveArgumentInvalid,
                    &site.coordinate,
                    "enumValue is not attached to an enum value",
                ));
            };
            if !enumeration.values.contains_key(&canonical) {
                return Err(diagnostic(
                    DiagnosticCode::SchemaDirectiveArgumentInvalid,
                    &site.coordinate,
                    "enumValue target is not a sibling enum wire name",
                ));
            }
            let target = &enumeration.values[&canonical];
            if target
                .directives
                .iter()
                .any(|directive| directive.name.as_str() == "enumValue")
            {
                return Err(diagnostic(
                    DiagnosticCode::SchemaDirectiveArgumentInvalid,
                    &site.coordinate,
                    "enumValue aliases may not form a chain",
                ));
            }
            Ok(DirectiveApplicationPolicy::EnumValueAlias { canonical })
        }
        DirectivePolicy::SourceMap => Ok(DirectiveApplicationPolicy::SourceMap {
            module: string_argument(
                site,
                "module",
                None,
                DiagnosticCode::SchemaDirectiveArgumentInvalid,
            )?,
        }),
        DirectivePolicy::Reevaluate => Ok(DirectiveApplicationPolicy::Reevaluate),
        DirectivePolicy::WorkflowRole => Ok(DirectiveApplicationPolicy::WorkflowRole {
            role: site.application.name.clone(),
        }),
        // The caller refuses these on a reviewed Core coordinate, so the site belongs to
        // another module.
        DirectivePolicy::TargetInactive => Ok(DirectiveApplicationPolicy::ModuleAuthoring {
            directive: site.application.name.clone(),
        }),
    }
}

fn string_argument(
    site: &ApplicationSite<'_>,
    name: &str,
    default: Option<&str>,
    code: DiagnosticCode,
) -> Result<String, Diagnostic> {
    let name = SchemaName::try_from(name).map_err(|()| {
        diagnostic(
            code,
            &site.coordinate,
            "directive policy argument is invalid",
        )
    })?;
    let Some(value) = site.application.arguments.get(&name) else {
        return default
            .map(str::to_owned)
            .ok_or_else(|| diagnostic(code, &site.coordinate, "directive argument is missing"));
    };
    let Some(value) = value else {
        return default
            .map(str::to_owned)
            .ok_or_else(|| diagnostic(code, &site.coordinate, "directive argument has no value"));
    };
    serde_json::from_str::<String>(value).map_err(|_| {
        diagnostic(
            code,
            &site.coordinate,
            "directive argument is not an encoded string",
        )
    })
}

fn diagnostic(code: DiagnosticCode, coordinate: &SchemaCoordinate, message: &str) -> Diagnostic {
    Diagnostic::new(
        code,
        Some(DiagnosticCoordinate::new(coordinate.as_str())),
        message,
    )
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::project_directives;
    use crate::diagnostic::DiagnosticCode;
    use crate::schema::{CoreCoordinateManifest, decode_and_validate, decode_and_validate_visible};
    use crate::target::CodegenTarget;

    const TARGET: &[u8] = include_bytes!("../../../codegen/target.json");
    const CORE_SCHEMA: &[u8] = include_bytes!("../../../codegen/schema.json");

    /// The checked schema with one more object type, as a module dependency adds one,
    /// that applies a module-authoring directive to itself, to a field and to an
    /// argument.
    fn schema_with_a_directed_extension() -> Vec<u8> {
        let mut document: Value =
            serde_json::from_slice(CORE_SCHEMA).expect("checked schema must decode");
        let directive = |name: &str, arguments: Value| json!([{"name": name, "args": arguments}]);
        document["__schema"]["types"]
            .as_array_mut()
            .expect("schema types must be an array")
            .push(json!({
                "kind": "OBJECT",
                "name": "Greeter",
                "description": null,
                "fields": [{
                    "name": "entries",
                    "description": null,
                    "type": {"kind": "SCALAR", "name": "String"},
                    "args": [{
                        "name": "source",
                        "description": null,
                        "type": {"kind": "SCALAR", "name": "String"},
                        "defaultValue": null,
                        "isDeprecated": false,
                        "deprecationReason": null,
                        "directives": directive(
                            "defaultPath",
                            json!([{"name": "path", "value": "\"/\""}])
                        )
                    }],
                    "isDeprecated": false,
                    "deprecationReason": null,
                    "directives": directive("up", json!([]))
                }],
                "inputFields": [],
                "interfaces": [],
                "enumValues": [],
                "possibleTypes": [],
                "directives": directive("collection", json!([]))
            }));
        serde_json::to_vec(&document).expect("extended schema must encode")
    }

    // A visible schema never reaches the projection with a module-authoring directive on
    // a reviewed Core coordinate, since verification refuses it first. The projection
    // decides by the reviewed coordinates all the same: the same applications are
    // accepted outside them and refused when the whole schema is reviewed Core.
    #[test]
    fn module_authoring_directives_are_accepted_only_outside_reviewed_core() {
        let target = CodegenTarget::decode_exact(TARGET).expect("checked target must decode");
        let reviewed = decode_and_validate(&target, CORE_SCHEMA).expect("checked schema is valid");
        let core = CoreCoordinateManifest::from_checked_schema(&reviewed)
            .expect("checked schema has a manifest");
        let visible =
            decode_and_validate_visible(&target, &schema_with_a_directed_extension(), &core, None)
                .expect("the extension leaves reviewed Core alone");

        let accepted = project_directives(&visible, Some(&core))
            .expect("the directives sit outside reviewed Core");
        for (name, coordinate) in [
            ("collection", "Greeter"),
            ("up", "Greeter.entries"),
            ("defaultPath", "Greeter.entries(source:)"),
        ] {
            let record = accepted
                .records()
                .values()
                .find(|record| record.name.as_str() == name)
                .expect("the target defines the directive");
            let sites = record
                .applications
                .keys()
                .map(|coordinate| coordinate.as_str())
                .collect::<Vec<_>>();
            assert_eq!(sites, [coordinate]);
        }

        let refused = project_directives(&visible, None)
            .expect_err("reviewed Core may not apply a module-authoring directive");
        let coordinates = refused
            .diagnostics()
            .iter()
            .map(|diagnostic| {
                assert_eq!(
                    diagnostic.code,
                    DiagnosticCode::TargetInactiveDirectiveChanged
                );
                diagnostic
                    .coordinate
                    .as_ref()
                    .expect("the refusal names its coordinate")
                    .as_str()
                    .to_owned()
            })
            .collect::<Vec<_>>();
        assert_eq!(
            coordinates,
            ["Greeter", "Greeter.entries", "Greeter.entries(source:)"]
        );
    }
}
