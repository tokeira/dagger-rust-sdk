//! One module's typed client, for a Cargo package that is not a module.
//!
//! A module calls the modules it depends on through `ModuleQuery`, the query root of
//! its own session, where the engine has already served them. A Cargo package that is
//! not a module has no such root: its session is served Core alone, and each module
//! it calls must be served to the session first. So a package gets one client for each
//! module, in a directory of its own. The directory's `mod.rs` declares a `Client`
//! built from the package's SDK client, which carries the module's address and pin so
//! that the SDK serves the module before the first request, and the directory holds
//! one file for each type the module owns.
//!
//! The types are bound by the dependency renderer, exactly as a module's dependencies
//! are, and `Client` takes the place `ModuleQuery` has there: the module's constructor
//! is a method of it, and it loads a handle of the module or of Core from an ID. Its
//! API is the module's alone. Dagger's Go SDK generates the same for a package: a
//! `Client` that holds only the bound module, built from the package's SDK client with
//! the module's address and pin (`GenerateUnifiedClient` in
//! cmd/dagger-go-sdk-codegen/generator/gogenerator/generate_unified.go of
//! github.com/dagger/go-sdk at bb23e8d).
//!
//! Invariants a future editor must preserve:
//!
//! - The first line of every file begins `// @generated`. The record that owns a
//!   client's files recognises a generated file by it.
//! - `mod.rs` re-exports every type file by glob, so each name it writes for itself is
//!   an absolute path: a module's type may be called `Result`, `Send` or `Into`, and a
//!   glob import takes the place of the standard prelude's name.
//! - `mod.rs` defines `Client` and `ClientLoadable`, and `Client` has the methods in
//!   [`CLIENT_METHODS`]. A type or a constructor of the module that would take one of
//!   those names is refused, since the name is the module's own and no declaration of
//!   the package's can change it.
//! - A client compiles without warnings as part of its author's crate, whatever the
//!   crate uses of it and whichever edition the crate has. `mod.rs` allows the lints
//!   that generated code cannot avoid.

use std::collections::BTreeMap;

use crate::diagnostic::{DiagnosticCode, DiagnosticSet};
use crate::naming::NameContext;
use crate::projection::types::TypeProjection;

use super::dependency::{BindingSurface, constructor_fields, method_name};
use super::model::{
    CandidateArtifactKind, ContentDomain, PostWorkPlan, RelativeOperationPath, operation_diagnostic,
};
use super::render::{checked_source, generated_header, render_owned_type, rust_source_paths};
use super::renderers::RendererOutput;
use super::visible::VisibleSchemaPlan;

/// The public names a client's `mod.rs` defines for itself. The module's types are
/// re-exported beside them, so a type that projects to one of these names cannot be
/// bound.
const CLIENT_SUPPORT_NAMES: &[&str] = &["Client", "ClientLoadable"];

/// The methods a client's `mod.rs` writes on `Client`. The module's constructor is a
/// method of `Client` named after the module, so a module may not project to one of
/// these names. (`__query_builder` is there too, and GraphQL reserves such a name.)
const CLIENT_METHODS: &[&str] = &["load", "new", "r#ref"];

/// Inputs of one module's client.
pub struct ModuleClientRenderInput<'a> {
    /// The schema the engine serves for the module's client: Core at the SDK's target
    /// plus the module, whose types carry `@sourceMap(module:)`.
    pub schema: &'a VisibleSchemaPlan,
    /// The client's directory, which the rendered paths lie beneath.
    pub output: &'a RelativeOperationPath,
    /// The name under which the package reaches the SDK: the key of its SDK
    /// dependency, with `_` for `-`.
    pub sdk_dependency_alias: &'a str,
    /// The address the engine serves the module from: a git ref, or a local module's
    /// path in its workspace.
    pub address: &'a str,
    /// The commit a git ref is pinned to, empty for a local module.
    pub pin: &'a str,
}

/// Renders one module's client into its directory: `mod.rs`, with the module's
/// `Client`, and one file for each type the module owns.
///
/// The output lists every file as generated Rust source to format. Nothing is rendered
/// for Core, which a client reaches through the package's SDK dependency.
///
/// # Errors
///
/// Returns diagnostics when the schema holds no module, when a type or the constructor
/// of the module would take a name the client defines for itself, and for whatever the
/// bindings of a module's types refuse.
pub fn render_module_client(
    input: ModuleClientRenderInput<'_>,
) -> Result<RendererOutput, DiagnosticSet> {
    let plan = input.schema;
    let sdk = input.sdk_dependency_alias;
    if syn::parse_str::<syn::Ident>(sdk).is_err() {
        return Err(DiagnosticSet::one(operation_diagnostic(
            DiagnosticCode::RustNameInvalid,
            "client.sdk-dependency",
            "the package's SDK dependency name is not a Rust identifier",
        )));
    }
    refuse_client_collisions(plan)?;

    let mut output = RendererOutput::new(ContentDomain::ModuleClient);
    let mut modules = Vec::new();
    for projection in plan.projection().named_types().values() {
        let Some((_, module_name, source)) =
            render_owned_type(plan, projection, sdk, BindingSurface::Client)?
        else {
            continue;
        };
        let stem = module_name.strip_prefix("r#").unwrap_or(&module_name);
        let file_name = format!("{stem}.rs");
        output.insert_artifact(
            input.output.join(&file_name)?,
            CandidateArtifactKind::RustSource,
            source,
        )?;
        modules.push((module_name, file_name));
    }
    modules.sort();
    let header = generated_header(
        plan,
        "A typed client for one Dagger module, generated for this package.",
        BindingSurface::Client,
    )?;
    let index = checked_source(
        input.output.as_str(),
        render_mod(&header, sdk, input.address, input.pin, &modules),
    )?;
    output.insert_artifact(
        input.output.join("mod.rs")?,
        CandidateArtifactKind::RustSource,
        index,
    )?;
    let rust_files = rust_source_paths(&output.artifacts);
    output.vcs_generated.extend(rust_files.iter().cloned());
    output
        .post_work
        .push(PostWorkPlan::FormatRust { files: rust_files });
    Ok(output)
}

// The module's types are bound in the client's directory, each in a file named after
// it and re-exported from `mod.rs`, and its constructor is a method of `Client`. A
// name of the module that meets one `mod.rs` writes for itself would shadow the type
// or define the method twice, so it is refused here with its own coordinate rather
// than as a compiler error in the package's generated code. The engine names a
// module's types and constructor after the module (`Namer.NamespaceObject` in
// core/gqlformat.go and `ModuleObject.Install` in core/object.go of dagger/dagger at
// v1.0.0-beta.27), so nothing in the package can rename them.
fn refuse_client_collisions(plan: &VisibleSchemaPlan) -> Result<(), DiagnosticSet> {
    let mut diagnostics = Vec::new();
    let mut owned = 0_usize;
    for projection in plan.projection().named_types().values() {
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
                plan.projection()
                    .names()
                    .get(&enumeration.coordinate, NameContext::Module)
                    .map(|name| name.identifier.clone()),
            ),
            TypeProjection::InputObject(_)
            | TypeProjection::Scalar(_)
            | TypeProjection::TargetPrivate(_) => continue,
        };
        let Some(owner) = plan.dependency_module(wire_name) else {
            continue;
        };
        owned += 1;
        for rust_name in rust_names {
            if CLIENT_SUPPORT_NAMES.contains(&rust_name) {
                diagnostics.push(operation_diagnostic(
                    DiagnosticCode::RustNameCollision,
                    coordinate.as_str(),
                    &format!(
                        "type `{wire_name}` of module `{owner}` would be bound as `{rust_name}`, which every generated client defines; no client can be generated for a module with a type of this name"
                    ),
                ));
            }
        }
        if let Some(module_name) = module_name
            && module_name.strip_prefix("r#").unwrap_or(&module_name) == "mod"
        {
            diagnostics.push(operation_diagnostic(
                DiagnosticCode::OperationArtifactCollision,
                coordinate.as_str(),
                &format!(
                    "type `{wire_name}` of module `{owner}` would be bound in `mod.rs`, which holds the generated client; no client can be generated for a module with a type of this name"
                ),
            ));
        }
    }
    let mut constructors = BTreeMap::new();
    for (field, target) in constructor_fields(plan) {
        let owner = plan.dependency_module(target).unwrap_or_default();
        for method in [Some(method_name(field)), field.options_method_name.clone()]
            .into_iter()
            .flatten()
        {
            let stated = method.strip_prefix("r#").unwrap_or(&method);
            if CLIENT_METHODS.contains(&method.as_str()) {
                diagnostics.push(operation_diagnostic(
                    DiagnosticCode::RustNameCollision,
                    field.coordinate.as_str(),
                    &format!(
                        "the constructor of module `{owner}` would be bound as `Client::{stated}`, which every generated client defines; no client can be generated for a module of this name"
                    ),
                ));
            } else if let Some(previous) = constructors.insert(method.clone(), &field.coordinate)
                && previous != &field.coordinate
            {
                diagnostics.push(operation_diagnostic(
                    DiagnosticCode::RustNameCollision,
                    field.coordinate.as_str(),
                    &format!(
                        "the constructor of module `{owner}` would be bound as `Client::{stated}`, as `{}` already is",
                        previous.as_str()
                    ),
                ));
            }
        }
    }
    if owned == 0 || (diagnostics.is_empty() && constructors.is_empty()) {
        diagnostics.push(operation_diagnostic(
            DiagnosticCode::ClientModuleRootInvalid,
            plan.canonical().query().as_str(),
            "the schema holds no module to generate a client for: no type carries `@sourceMap(module:)`, or `Query` has no field that constructs one",
        ));
    }
    DiagnosticSet::new(diagnostics).map_or(Ok(()), Err)
}

// The SDK is named from the root of the extern prelude, and every standard item from
// the crate root: the module's types are bound in modules declared in this file and
// re-exported into it by glob, and any of them may be called what this file would
// otherwise write bare.
//
// `Client::r#ref` and `Client::load` are generic over a trait of the client's own, as
// `ModuleQuery`'s are over `ModuleLoadable`: the SDK's `Query::r#ref` and `Query::load`
// accept only Core handles, whose constructor is sealed inside the SDK. The SDK's
// handles implement the trait here and the module's handles implement it in their
// bindings, so one spelling loads either.
fn render_mod(
    header: &str,
    alias: &str,
    address: &str,
    pin: &str,
    modules: &[(String, String)],
) -> String {
    let mut source = format!(
        "{header}\n\
         // An author's crate compiles this client as its own code. Whatever the crate does\n\
         // not call is dead code in a binary, and generated code handles every item of\n\
         // the module, deprecated ones included.\n\
         #![allow(dead_code, deprecated)]\n\
         // The bindings take the module's functions' names and arguments as its author\n\
         // chose them, whatever Clippy would say of a method so named, and bind the type\n\
         // named after the module in a file named as this directory is.\n\
         #![allow(\n    clippy::module_inception,\n    clippy::new_ret_no_self,\n    clippy::should_implement_trait,\n    clippy::too_many_arguments,\n    clippy::wrong_self_convention\n)]\n\n\
         /// The module's client on one session.\n///\n/// It has one method for the module's constructor, which returns the module's main\n/// object, and loads the module's objects and Core's from their IDs. Core itself is\n/// reached through the SDK client this one is made from.\n#[derive(Clone)]\npub struct Client {{\n    query: ::{alias}::QueryBuilder,\n}}\n\n\
         /// A generated object or interface handle that `Client::r#ref` and `Client::load`\n/// build from an ID: a handle of the module, or a Core handle.\n///\n/// Generated code implements this trait; its methods are not part of the client's API.\npub trait ClientLoadable: ::core::marker::Sized {{\n    #[doc(hidden)]\n    fn __client_ref(query: &::{alias}::QueryBuilder, id: ::{alias}::IdInput<Self>) -> Self;\n    #[doc(hidden)]\n    fn __client_load(\n        query: &::{alias}::QueryBuilder,\n        id: ::{alias}::IdInput<Self>,\n    ) -> impl ::core::future::Future<Output = ::core::result::Result<Self, ::{alias}::QueryError>>\n    + ::core::marker::Send;\n}}\n\n\
         impl<T> ClientLoadable for T\nwhere\n    T: ::{alias}::Loadable + ::core::marker::Send + 'static,\n{{\n    fn __client_ref(query: &::{alias}::QueryBuilder, id: ::{alias}::IdInput<Self>) -> Self {{\n        query.generated_query_root().r#ref(id)\n    }}\n\n    fn __client_load(\n        query: &::{alias}::QueryBuilder,\n        id: ::{alias}::IdInput<Self>,\n    ) -> impl ::core::future::Future<Output = ::core::result::Result<Self, ::{alias}::QueryError>>\n    + ::core::marker::Send {{\n        let root = query.generated_query_root();\n        async move {{ root.load(id).await }}\n    }}\n}}\n\n\
         impl Client {{\n    /// Returns the module's client on the session of `client`, without a request.\n    ///\n    /// The session has the engine serve the module before the first request made\n    /// through this client or through anything built from it.\n    #[must_use]\n    pub fn new(client: &::{alias}::Client) -> Self {{\n        Self {{\n            query: client.generated_module_query_builder({address:?}, {pin:?}),\n        }}\n    }}\n\n    \
         /// Borrows the builder on which generated bindings select the module's fields.\n    #[doc(hidden)]\n    fn __query_builder(&self) -> &::{alias}::QueryBuilder {{\n        &self.query\n    }}\n\n    \
         /// Returns a lazy reference to the object with this ID, without a request.\n    ///\n    /// The handle asks the engine for its ID when it is used as one.\n    pub fn r#ref<T: ClientLoadable>(\n        &self,\n        id: impl ::core::convert::Into<::{alias}::IdInput<T>>,\n    ) -> T {{\n        T::__client_ref(self.__query_builder(), ::core::convert::Into::into(id))\n    }}\n\n    \
         /// Loads the object with this ID after checking that it exists and has this type.\n    pub async fn load<T: ClientLoadable>(\n        &self,\n        id: impl ::core::convert::Into<::{alias}::IdInput<T>>,\n    ) -> ::core::result::Result<T, ::{alias}::QueryError> {{\n        T::__client_load(self.__query_builder(), ::core::convert::Into::into(id)).await\n    }}\n}}\n\n"
    );
    source.push_str(
        "// The module's types, each bound in a file of its own and re-exported here. Nothing\n\
         // else in the client names a type that none of the module's functions returns, so\n\
         // its re-export is unused unless the crate names the type.\n",
    );
    for (module, file) in modules {
        source.push_str(&format!(
            "#[path = {file:?}]\nmod {module};\n#[allow(unused_imports)]\npub use self::{module}::*;\n"
        ));
    }
    source
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::{CLIENT_METHODS, CLIENT_SUPPORT_NAMES, render_mod};

    // Rendering refuses a type or a constructor of the module that would take a name
    // `mod.rs` writes for itself, by two lists that must name everything it defines.
    #[test]
    fn the_refused_names_are_everything_the_client_defines() {
        let source = render_mod(
            "// @generated {}\n//! A client.\n",
            "dagger_sdk",
            "/greeter",
            "",
            &[],
        );
        let file = syn::parse_file(&source).expect("a client's mod.rs is valid Rust");
        let mut defined = BTreeSet::new();
        let mut methods = BTreeSet::new();
        for item in &file.items {
            match item {
                syn::Item::Struct(item) => {
                    defined.insert(item.ident.to_string());
                }
                syn::Item::Trait(item) => {
                    defined.insert(item.ident.to_string());
                }
                syn::Item::Impl(item) if item.trait_.is_none() => {
                    for member in &item.items {
                        if let syn::ImplItem::Fn(function) = member {
                            methods.insert(function.sig.ident.to_string());
                        }
                    }
                }
                syn::Item::Impl(_) => {}
                other => panic!("mod.rs declares an item no list covers: {other:?}"),
            }
        }
        assert_eq!(
            defined,
            CLIENT_SUPPORT_NAMES
                .iter()
                .map(|name| (*name).to_owned())
                .collect()
        );
        // GraphQL reserves a name that begins with two underscores, so no constructor
        // can be called what the bindings' own entry point is.
        methods.remove("__query_builder");
        assert_eq!(
            methods,
            CLIENT_METHODS
                .iter()
                .map(|name| (*name).to_owned())
                .collect()
        );
    }
}
