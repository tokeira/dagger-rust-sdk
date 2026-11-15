//! Manifest-backed compatibility for engine-visible schemas.
//!
//! The checked core schema is reduced to coordinate-level wire-contract fingerprints:
//! what a client sends and what it receives, without descriptions, deprecation notes or
//! other documentation. Visible schemas may add operation-scoped coordinates, and a
//! later engine serving the target's API view may reword documentation. Every reviewed
//! core coordinate must stay present and keep its wire contract; module operations may
//! omit only the target's reviewed introspection scrub closure. An addition that would
//! change a reviewed core call, such as a new value on a core enum or a new required
//! argument, is refused. No name prefix is trusted as an ownership boundary.
//!
//! Generation then consumes the reviewed core definitions themselves, so documentation
//! drift and harmless engine additions neither reach rendered bindings nor change the
//! consumed schema's identity.

use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;

use crate::diagnostic::{Diagnostic, DiagnosticCode, DiagnosticCoordinate, DiagnosticSet};
use crate::projection::catalog::SemanticDigest;

use super::canonical::{
    ArgumentDefinition, CanonicalSchema, DirectiveApplication, DirectiveDefinition, EnumDefinition,
    FieldDefinition, InputObjectDefinition, InterfaceDefinition, ObjectDefinition,
    SchemaCoordinate, SchemaName, TypeDefinition, TypeShape, TypeUse,
};
use super::defaults::ConstValue;

/// Directive applications that document a coordinate, classify it for engine workflows,
/// or are satisfied by construction in generated handles. None of them changes what a
/// client sends or receives, so they take no part in the wire contract.
const DOCUMENTATION_DIRECTIVES: &[&str] = &[
    "check",
    "deprecated",
    "experimental",
    "generate",
    "reevaluate",
    "sourceMap",
];

/// Validation policy selected before canonical schema projection.
#[derive(Clone, Copy, Debug)]
pub enum SchemaCompatibilityMode<'a> {
    /// Require the checked snapshot's exact bytes and coordinate inventory.
    ExactTarget,
    /// Require exact checked core semantics while admitting additional coordinates.
    ExactCoreWithExtensions(&'a CoreCoordinateManifest),
}

/// Semantic fingerprints for every coordinate in the reviewed core schema.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoreCoordinateManifest {
    coordinates: BTreeMap<SchemaCoordinate, SemanticDigest>,
    permitted_omissions: BTreeSet<SchemaCoordinate>,
    types: BTreeSet<SchemaName>,
}

impl CoreCoordinateManifest {
    /// Derives the manifest from a schema that already passed exact-target validation.
    pub fn from_checked_schema(schema: &CanonicalSchema) -> Result<Self, DiagnosticSet> {
        let coordinates = coordinate_fingerprints(schema, None)?;
        Ok(Self {
            coordinates,
            permitted_omissions: BTreeSet::new(),
            types: schema.types().keys().cloned().collect(),
        })
    }

    /// Derives the exact omission closure produced by Dagger's module introspection
    /// scrub while retaining the full checked-core coordinate authority.
    pub(crate) fn from_checked_schema_with_scrub(
        schema: &CanonicalSchema,
        hidden_types: &[&str],
        hidden_fields: &[&str],
    ) -> Result<Self, DiagnosticSet> {
        let coordinates = coordinate_fingerprints(schema, None)?;
        let permitted_omissions = scrubbed_coordinates(schema, hidden_types, hidden_fields);
        Ok(Self {
            coordinates,
            permitted_omissions,
            types: schema.types().keys().cloned().collect(),
        })
    }

    /// Returns the complete reviewed coordinate set.
    #[must_use]
    pub fn coordinates(&self) -> &BTreeMap<SchemaCoordinate, SemanticDigest> {
        &self.coordinates
    }

    /// Returns whether a coordinate belongs to the checked core schema.
    #[must_use]
    pub fn contains(&self, coordinate: &SchemaCoordinate) -> bool {
        self.coordinates.contains_key(coordinate)
    }

    pub(super) fn permits_missing_type(&self, name: &super::canonical::SchemaName) -> bool {
        self.permitted_omissions
            .contains(&SchemaCoordinate::named_type(name))
    }

    pub(super) fn verify(&self, schema: &CanonicalSchema) -> Result<(), DiagnosticSet> {
        let actual = coordinate_fingerprints(schema, Some(&self.types))?;
        let mut diagnostics = Vec::new();
        for (coordinate, expected) in &self.coordinates {
            match actual.get(coordinate) {
                None if self.permitted_omissions.contains(coordinate) => {}
                None => diagnostics.push(Diagnostic::new(
                    DiagnosticCode::SchemaCoreCoordinateMissing,
                    Some(DiagnosticCoordinate::new(coordinate.as_str())),
                    "visible schema omits a reviewed core coordinate",
                )),
                Some(candidate) if candidate != expected => diagnostics.push(Diagnostic::new(
                    DiagnosticCode::SchemaCoreCoordinateIncompatible,
                    Some(DiagnosticCoordinate::new(coordinate.as_str())),
                    "visible schema changes the wire contract of a reviewed core coordinate",
                )),
                Some(_) => {}
            }
        }
        self.refuse_breaking_additions(schema, &mut diagnostics);
        match DiagnosticSet::new(diagnostics) {
            Some(diagnostics) => Err(diagnostics),
            None => Ok(()),
        }
    }

    /// Refuses additions to reviewed core definitions that change a reviewed call: the
    /// precompiled bindings decode core enums exhaustively and never send an argument
    /// or input field they do not know.
    fn refuse_breaking_additions(
        &self,
        schema: &CanonicalSchema,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        for definition in schema.types().values() {
            if !self.contains(&SchemaCoordinate::named_type(definition.name())) {
                continue;
            }
            match definition {
                TypeDefinition::Scalar(_) => {}
                TypeDefinition::Object(definition) => {
                    self.refuse_required_arguments(&definition.fields, diagnostics);
                }
                TypeDefinition::Interface(definition) => {
                    self.refuse_required_arguments(&definition.fields, diagnostics);
                }
                TypeDefinition::Enum(definition) => {
                    for value in definition.values.values() {
                        if !self.contains(&value.coordinate) {
                            diagnostics.push(addition(
                                &value.coordinate,
                                "visible schema adds a value to a reviewed core enum",
                            ));
                        }
                    }
                }
                TypeDefinition::InputObject(definition) => {
                    for field in definition.fields.values() {
                        if !self.contains(&field.coordinate)
                            && is_required(&field.type_use, field.default.as_ref())
                        {
                            diagnostics.push(addition(
                                &field.coordinate,
                                "visible schema adds a required field to a reviewed core input object",
                            ));
                        }
                    }
                }
            }
        }
    }

    fn refuse_required_arguments(
        &self,
        fields: &BTreeMap<SchemaName, FieldDefinition>,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        for field in fields
            .values()
            .filter(|field| self.contains(&field.coordinate))
        {
            for argument in field.arguments.values() {
                if !self.contains(&argument.coordinate)
                    && is_required(&argument.type_use, argument.default.as_ref())
                {
                    diagnostics.push(addition(
                        &argument.coordinate,
                        "visible schema adds a required argument to a reviewed core field",
                    ));
                }
            }
        }
    }

    /// Rebuilds a verified visible schema from the reviewed core definitions.
    ///
    /// Reviewed core coordinates take their checked definitions, documentation
    /// included, so documentation drift reaches neither bindings nor the consumed
    /// schema's identity. Coordinates an operation adds, such as a module's types and
    /// its constructor on `Query`, are kept as the engine states them. A field a later
    /// engine adds to a core type, with no operation-scoped type in its signature, and
    /// an optional argument or input field it adds to a core definition, are not
    /// consumed: the precompiled bindings never select or send them.
    pub(crate) fn consumed_schema(
        &self,
        reviewed: &CanonicalSchema,
        visible: &CanonicalSchema,
    ) -> CanonicalSchema {
        let types = visible
            .types()
            .iter()
            .map(|(name, definition)| {
                let consumed = reviewed
                    .types()
                    .get(name)
                    .filter(|reviewed| {
                        self.contains(&SchemaCoordinate::named_type(reviewed.name()))
                    })
                    .map_or_else(
                        || definition.clone(),
                        |reviewed| self.consumed_type(reviewed, definition),
                    );
                (name.clone(), consumed)
            })
            .collect::<BTreeMap<_, _>>();
        let directives = visible
            .directives()
            .iter()
            .map(|(name, definition)| {
                let consumed = reviewed
                    .directives()
                    .get(name)
                    .filter(|reviewed| self.contains(&reviewed.coordinate))
                    .unwrap_or(definition)
                    .clone();
                (name.clone(), consumed)
            })
            .collect::<BTreeMap<_, _>>();
        let inventory = super::validate::inventory(&types, &directives, true);
        CanonicalSchema::new(
            visible.target().clone(),
            visible.query().clone(),
            types,
            directives,
            inventory,
        )
    }

    fn consumed_type(&self, reviewed: &TypeDefinition, visible: &TypeDefinition) -> TypeDefinition {
        match (reviewed, visible) {
            (TypeDefinition::Scalar(reviewed), TypeDefinition::Scalar(_)) => {
                TypeDefinition::Scalar(reviewed.clone())
            }
            // An object's interface set and an interface's possible types keep the
            // operation's own types beside the verified core ones.
            (TypeDefinition::Object(reviewed), TypeDefinition::Object(visible)) => {
                TypeDefinition::Object(ObjectDefinition {
                    name: reviewed.name.clone(),
                    coordinate: reviewed.coordinate.clone(),
                    description: reviewed.description.clone(),
                    fields: self.consumed_fields(&reviewed.fields, &visible.fields),
                    interfaces: visible.interfaces.clone(),
                    directives: reviewed.directives.clone(),
                })
            }
            (TypeDefinition::Interface(reviewed), TypeDefinition::Interface(visible)) => {
                TypeDefinition::Interface(InterfaceDefinition {
                    name: reviewed.name.clone(),
                    coordinate: reviewed.coordinate.clone(),
                    description: reviewed.description.clone(),
                    fields: self.consumed_fields(&reviewed.fields, &visible.fields),
                    interfaces: visible.interfaces.clone(),
                    possible_types: visible.possible_types.clone(),
                    directives: reviewed.directives.clone(),
                })
            }
            (TypeDefinition::Enum(reviewed), TypeDefinition::Enum(visible)) => {
                TypeDefinition::Enum(EnumDefinition {
                    name: reviewed.name.clone(),
                    coordinate: reviewed.coordinate.clone(),
                    description: reviewed.description.clone(),
                    values: present(&reviewed.values, &visible.values),
                    directives: reviewed.directives.clone(),
                })
            }
            (TypeDefinition::InputObject(reviewed), TypeDefinition::InputObject(visible)) => {
                TypeDefinition::InputObject(InputObjectDefinition {
                    name: reviewed.name.clone(),
                    coordinate: reviewed.coordinate.clone(),
                    description: reviewed.description.clone(),
                    fields: present(&reviewed.fields, &visible.fields),
                    directives: reviewed.directives.clone(),
                })
            }
            // Verification refuses a changed definition kind at the type coordinate.
            _ => visible.clone(),
        }
    }

    fn consumed_fields(
        &self,
        reviewed: &BTreeMap<SchemaName, FieldDefinition>,
        visible: &BTreeMap<SchemaName, FieldDefinition>,
    ) -> BTreeMap<SchemaName, FieldDefinition> {
        visible
            .iter()
            .filter_map(|(name, field)| {
                match reviewed
                    .get(name)
                    .filter(|reviewed| self.contains(&reviewed.coordinate))
                {
                    Some(reviewed) => Some((
                        name.clone(),
                        FieldDefinition {
                            arguments: present(&reviewed.arguments, &field.arguments),
                            ..reviewed.clone()
                        },
                    )),
                    None if self.names_only_reviewed_types(field) => None,
                    None => Some((name.clone(), field.clone())),
                }
            })
            .collect()
    }

    fn names_only_reviewed_types(&self, field: &FieldDefinition) -> bool {
        self.types.contains(leaf_type(&field.type_use))
            && field
                .arguments
                .values()
                .all(|argument| self.types.contains(leaf_type(&argument.type_use)))
    }
}

/// Keeps the reviewed members the visible definition still has.
fn present<T: Clone>(
    reviewed: &BTreeMap<SchemaName, T>,
    visible: &BTreeMap<SchemaName, T>,
) -> BTreeMap<SchemaName, T> {
    reviewed
        .iter()
        .filter(|(name, _)| visible.contains_key(*name))
        .map(|(name, member)| (name.clone(), member.clone()))
        .collect()
}

fn leaf_type(type_use: &TypeUse) -> &SchemaName {
    match &type_use.shape {
        TypeShape::Named(name) => name,
        TypeShape::List(element) => leaf_type(element),
    }
}

fn is_required(type_use: &TypeUse, default: Option<&ConstValue>) -> bool {
    !type_use.nullable && default.is_none()
}

fn addition(coordinate: &SchemaCoordinate, message: &str) -> Diagnostic {
    Diagnostic::new(
        DiagnosticCode::SchemaCoreCoordinateIncompatible,
        Some(DiagnosticCoordinate::new(coordinate.as_str())),
        message,
    )
}

fn scrubbed_coordinates(
    schema: &CanonicalSchema,
    hidden_types: &[&str],
    hidden_fields: &[&str],
) -> BTreeSet<SchemaCoordinate> {
    let mut removed_types = BTreeSet::new();
    let mut removed_members = BTreeSet::new();

    // Dagger applies ScrubType sequentially. A non-scalar definition which becomes
    // empty during one pass disappears, but that derived removal does not start a new
    // recursive scrub pass of its own.
    for hidden in hidden_types {
        for definition in schema.types().values() {
            let name = definition.name().as_str();
            if removed_types.contains(name) {
                continue;
            }
            if name == *hidden {
                removed_types.insert(name.to_owned());
                continue;
            }

            match definition {
                TypeDefinition::Scalar(_) => {}
                TypeDefinition::Object(definition) => {
                    scrub_fields(&definition.fields, hidden, &mut removed_members);
                    if definition
                        .fields
                        .values()
                        .all(|field| removed_members.contains(&field.coordinate))
                    {
                        removed_types.insert(name.to_owned());
                    }
                }
                TypeDefinition::Interface(definition) => {
                    scrub_fields(&definition.fields, hidden, &mut removed_members);
                    if definition
                        .fields
                        .values()
                        .all(|field| removed_members.contains(&field.coordinate))
                    {
                        removed_types.insert(name.to_owned());
                    }
                }
                TypeDefinition::InputObject(definition) => {
                    for field in definition.fields.values() {
                        if type_use_references(&field.type_use, hidden) {
                            removed_members.insert(field.coordinate.clone());
                        }
                    }
                    if definition
                        .fields
                        .values()
                        .all(|field| removed_members.contains(&field.coordinate))
                    {
                        removed_types.insert(name.to_owned());
                    }
                }
                TypeDefinition::Enum(definition) => {
                    for value in definition.values.values() {
                        if value.name.as_str() == *hidden {
                            removed_members.insert(value.coordinate.clone());
                        }
                    }
                    if definition
                        .values
                        .values()
                        .all(|value| removed_members.contains(&value.coordinate))
                    {
                        removed_types.insert(name.to_owned());
                    }
                }
            }
        }
    }

    // ScrubField runs after type scrubbing and deliberately does not remove an
    // otherwise empty parent definition.
    for hidden in hidden_fields {
        for definition in schema.types().values() {
            match definition {
                TypeDefinition::Object(definition) => {
                    mark_named_field(&definition.fields, hidden, &mut removed_members);
                }
                TypeDefinition::Interface(definition) => {
                    mark_named_field(&definition.fields, hidden, &mut removed_members);
                }
                _ => {}
            }
        }
    }

    let mut omitted = removed_members;
    for definition in schema.types().values() {
        if removed_types.contains(definition.name().as_str()) {
            add_definition_coordinates(definition, &mut omitted);
        }
    }
    omitted
}

fn scrub_fields(
    fields: &BTreeMap<super::canonical::SchemaName, super::canonical::FieldDefinition>,
    hidden: &str,
    removed: &mut BTreeSet<SchemaCoordinate>,
) {
    for field in fields.values() {
        let references_hidden = type_use_references(&field.type_use, hidden)
            || field
                .arguments
                .values()
                .any(|argument| type_use_references(&argument.type_use, hidden));
        if references_hidden {
            removed.insert(field.coordinate.clone());
            removed.extend(
                field
                    .arguments
                    .values()
                    .map(|argument| argument.coordinate.clone()),
            );
        }
    }
}

fn mark_named_field(
    fields: &BTreeMap<super::canonical::SchemaName, super::canonical::FieldDefinition>,
    hidden: &str,
    removed: &mut BTreeSet<SchemaCoordinate>,
) {
    let Some(field) = fields
        .values()
        .find(|field| field.coordinate.as_str() == hidden)
    else {
        return;
    };
    removed.insert(field.coordinate.clone());
    removed.extend(
        field
            .arguments
            .values()
            .map(|argument| argument.coordinate.clone()),
    );
}

fn type_use_references(type_use: &super::canonical::TypeUse, hidden: &str) -> bool {
    match &type_use.shape {
        super::canonical::TypeShape::Named(name) => name.as_str() == hidden,
        super::canonical::TypeShape::List(element) => type_use_references(element, hidden),
    }
}

fn add_definition_coordinates(
    definition: &TypeDefinition,
    omitted: &mut BTreeSet<SchemaCoordinate>,
) {
    omitted.insert(SchemaCoordinate::named_type(definition.name()));
    match definition {
        TypeDefinition::Scalar(_) => {}
        TypeDefinition::Object(definition) => add_field_coordinates(&definition.fields, omitted),
        TypeDefinition::Interface(definition) => {
            add_field_coordinates(&definition.fields, omitted);
        }
        TypeDefinition::Enum(definition) => {
            omitted.extend(
                definition
                    .values
                    .values()
                    .map(|value| value.coordinate.clone()),
            );
        }
        TypeDefinition::InputObject(definition) => {
            omitted.extend(
                definition
                    .fields
                    .values()
                    .map(|field| field.coordinate.clone()),
            );
        }
    }
}

fn add_field_coordinates(
    fields: &BTreeMap<super::canonical::SchemaName, super::canonical::FieldDefinition>,
    omitted: &mut BTreeSet<SchemaCoordinate>,
) {
    for field in fields.values() {
        omitted.insert(field.coordinate.clone());
        omitted.extend(
            field
                .arguments
                .values()
                .map(|argument| argument.coordinate.clone()),
        );
    }
}

/// The wire contract of one coordinate: names, types, defaults and the directive
/// applications that change what a client sends or receives.
#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
enum WireContract<'a> {
    Scalar {
        name: &'a SchemaName,
        directives: Vec<&'a DirectiveApplication>,
    },
    Object {
        name: &'a SchemaName,
        interfaces: BTreeSet<&'a SchemaName>,
        directives: Vec<&'a DirectiveApplication>,
    },
    Interface {
        name: &'a SchemaName,
        interfaces: BTreeSet<&'a SchemaName>,
        possible_types: BTreeSet<&'a SchemaName>,
        directives: Vec<&'a DirectiveApplication>,
    },
    Enum {
        name: &'a SchemaName,
        directives: Vec<&'a DirectiveApplication>,
    },
    InputObject {
        name: &'a SchemaName,
        directives: Vec<&'a DirectiveApplication>,
    },
    Field {
        name: &'a SchemaName,
        type_use: &'a TypeUse,
        directives: Vec<&'a DirectiveApplication>,
    },
    Argument {
        name: &'a SchemaName,
        type_use: &'a TypeUse,
        default: Option<&'a ConstValue>,
        directives: Vec<&'a DirectiveApplication>,
    },
    EnumValue {
        name: &'a SchemaName,
        directives: Vec<&'a DirectiveApplication>,
    },
    Directive {
        name: &'a SchemaName,
        locations: &'a BTreeSet<String>,
    },
}

fn wire_directives(directives: &[DirectiveApplication]) -> Vec<&DirectiveApplication> {
    directives
        .iter()
        .filter(|directive| !DOCUMENTATION_DIRECTIVES.contains(&directive.name.as_str()))
        .collect()
}

/// Restricts a set of type names to the reviewed core types, when a scope is given. An
/// operation's own types may implement a core interface, and a core object may
/// structurally satisfy an operation's interface; neither changes a core contract.
fn scoped<'a>(
    names: &'a BTreeSet<SchemaName>,
    reviewed: Option<&BTreeSet<SchemaName>>,
) -> BTreeSet<&'a SchemaName> {
    names
        .iter()
        .filter(|name| reviewed.is_none_or(|reviewed| reviewed.contains(*name)))
        .collect()
}

fn coordinate_fingerprints(
    schema: &CanonicalSchema,
    reviewed: Option<&BTreeSet<SchemaName>>,
) -> Result<BTreeMap<SchemaCoordinate, SemanticDigest>, DiagnosticSet> {
    let mut coordinates = BTreeMap::new();
    insert_fingerprint(
        &mut coordinates,
        SchemaCoordinate::query_root(),
        schema.query(),
    )?;
    for definition in schema.types().values() {
        match definition {
            TypeDefinition::Scalar(definition) => insert_fingerprint(
                &mut coordinates,
                definition.coordinate.clone(),
                &WireContract::Scalar {
                    name: &definition.name,
                    directives: wire_directives(&definition.directives),
                },
            )?,
            TypeDefinition::Object(definition) => {
                insert_fingerprint(
                    &mut coordinates,
                    definition.coordinate.clone(),
                    &WireContract::Object {
                        name: &definition.name,
                        interfaces: scoped(&definition.interfaces, reviewed),
                        directives: wire_directives(&definition.directives),
                    },
                )?;
                insert_fields(&mut coordinates, &definition.fields)?;
            }
            TypeDefinition::Interface(definition) => {
                insert_fingerprint(
                    &mut coordinates,
                    definition.coordinate.clone(),
                    &WireContract::Interface {
                        name: &definition.name,
                        interfaces: scoped(&definition.interfaces, reviewed),
                        possible_types: scoped(&definition.possible_types, reviewed),
                        directives: wire_directives(&definition.directives),
                    },
                )?;
                insert_fields(&mut coordinates, &definition.fields)?;
            }
            TypeDefinition::Enum(definition) => {
                insert_fingerprint(
                    &mut coordinates,
                    definition.coordinate.clone(),
                    &WireContract::Enum {
                        name: &definition.name,
                        directives: wire_directives(&definition.directives),
                    },
                )?;
                for value in definition.values.values() {
                    insert_fingerprint(
                        &mut coordinates,
                        value.coordinate.clone(),
                        &WireContract::EnumValue {
                            name: &value.name,
                            directives: wire_directives(&value.directives),
                        },
                    )?;
                }
            }
            TypeDefinition::InputObject(definition) => {
                insert_fingerprint(
                    &mut coordinates,
                    definition.coordinate.clone(),
                    &WireContract::InputObject {
                        name: &definition.name,
                        directives: wire_directives(&definition.directives),
                    },
                )?;
                for field in definition.fields.values() {
                    insert_fingerprint(
                        &mut coordinates,
                        field.coordinate.clone(),
                        &WireContract::Argument {
                            name: &field.name,
                            type_use: &field.type_use,
                            default: field.default.as_ref(),
                            directives: wire_directives(&field.directives),
                        },
                    )?;
                }
            }
        }
    }
    for directive in schema.directives().values() {
        insert_directive(&mut coordinates, directive)?;
    }
    Ok(coordinates)
}

fn insert_fields(
    coordinates: &mut BTreeMap<SchemaCoordinate, SemanticDigest>,
    fields: &BTreeMap<SchemaName, FieldDefinition>,
) -> Result<(), DiagnosticSet> {
    for field in fields.values() {
        insert_fingerprint(
            coordinates,
            field.coordinate.clone(),
            &WireContract::Field {
                name: &field.name,
                type_use: &field.type_use,
                directives: wire_directives(&field.directives),
            },
        )?;
        insert_arguments(coordinates, &field.arguments)?;
    }
    Ok(())
}

fn insert_arguments(
    coordinates: &mut BTreeMap<SchemaCoordinate, SemanticDigest>,
    arguments: &BTreeMap<SchemaName, ArgumentDefinition>,
) -> Result<(), DiagnosticSet> {
    for argument in arguments.values() {
        insert_fingerprint(
            coordinates,
            argument.coordinate.clone(),
            &WireContract::Argument {
                name: &argument.name,
                type_use: &argument.type_use,
                default: argument.default.as_ref(),
                directives: wire_directives(&argument.directives),
            },
        )?;
    }
    Ok(())
}

fn insert_directive(
    coordinates: &mut BTreeMap<SchemaCoordinate, SemanticDigest>,
    definition: &DirectiveDefinition,
) -> Result<(), DiagnosticSet> {
    insert_fingerprint(
        coordinates,
        definition.coordinate.clone(),
        &WireContract::Directive {
            name: &definition.name,
            locations: &definition.locations,
        },
    )?;
    insert_arguments(coordinates, &definition.arguments)
}

fn insert_fingerprint<T: Serialize>(
    coordinates: &mut BTreeMap<SchemaCoordinate, SemanticDigest>,
    coordinate: SchemaCoordinate,
    semantics: &T,
) -> Result<(), DiagnosticSet> {
    let fingerprint = SemanticDigest::for_value(semantics).map_err(|_| {
        DiagnosticSet::one(Diagnostic::new(
            DiagnosticCode::GeneratedProvenanceInvalid,
            Some(DiagnosticCoordinate::new(coordinate.as_str())),
            "schema coordinate could not be fingerprinted",
        ))
    })?;
    // Canonical validation already makes coordinates unique. Treat a duplicate here
    // as provenance failure so a future coordinate family cannot silently alias one.
    if coordinates
        .insert(coordinate.clone(), fingerprint)
        .is_some()
    {
        return Err(DiagnosticSet::one(Diagnostic::new(
            DiagnosticCode::GeneratedProvenanceInvalid,
            Some(DiagnosticCoordinate::new(coordinate.as_str())),
            "schema coordinate fingerprint aliases an existing coordinate",
        )));
    }
    Ok(())
}
