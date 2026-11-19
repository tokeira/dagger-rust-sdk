//! Typed bindings for the types of the modules a module depends on.
//!
//! The engine serves a module the schema of Core plus its direct dependencies, and
//! stamps every type a dependency owns with `@sourceMap(module:)`, synthesizing the
//! directive when the dependency's SDK gave none (`namespaceSourceMap` in
//! core/module.go at the target). Dagger's Go SDK finds a module's dependencies by that
//! directive (`DependencyNames` in cmd/codegen/introspection/filters.go), and so does
//! this renderer: a type is a dependency's when it lies outside the reviewed Core API
//! and carries the directive. No name prefix is trusted, since the engine lets a
//! dependency be declared under any name.
//!
//! A dependency's type is rendered from the projection every other binding uses, in
//! the calling convention of the SDK's own Core bindings, so module code calls a
//! dependency as it calls Core. The bindings live in the module's crate and reach the
//! SDK only through its public surface: a handle is a `QueryBuilder` on the call's
//! session, never a session or selection of its own.
//!
//! Invariants a future editor must preserve:
//!
//! - Every path into the SDK goes through the alias the caller passes. A module may
//!   key its SDK dependency under any name.
//! - A handle's only constructor is the doc-hidden `__from_query`. GraphQL reserves
//!   names that begin with two underscores, so no function of a dependency can be
//!   called that; the `from_query` and `selection` an untyped handle has could be.
//! - A handle built from an ID that a query returned for the object keeps it; a
//!   handle built from any other ID asks the engine. The SDK's entry points encode the
//!   difference, and the renderer picks the keeping one only for an ID its own request
//!   just returned.
//! - A field that names a type with no typed binding fails generation with the field's
//!   coordinate. Rendering it untyped would hand the caller a value no other binding
//!   accepts.
//! - A dependency's author chose its names without Rust in mind, and a binding must
//!   compile beside any of them. Each type is bound in a file of its own, where its
//!   name is in scope: a dependency called `result` has the main object `Result`. So
//!   every item of the standard library is written from the crate root
//!   (`::core::result::Result`), and a method of a prelude trait is called by its path
//!   (`::core::clone::Clone::clone(value)`), never by a name the bound type could
//!   capture or by a trait it could shadow.
//! - What a method declares for itself has a name GraphQL cannot give an argument: its
//!   locals begin with two underscores, and the parameter that carries the optional
//!   arguments moves aside for a required argument called `opts`.
//! - A method's name never takes the place of one a handle already has through the
//!   prelude: see `PRELUDE_METHODS`.
//! - A dependency's documentation is emitted so that rustdoc does not collect it for
//!   doctests. It is prose from another module, and a code block in it is not Rust.

use proc_macro2::{Ident, TokenStream};
use quote::quote;

use crate::diagnostic::{Diagnostic, DiagnosticCode, DiagnosticCoordinate, DiagnosticSet};
use crate::projection::fields::{
    ArgumentPresence, ArgumentProjection, FieldProjection, FieldStrategy,
};
use crate::projection::types::{
    EnumProjection, InterfaceProjection, ObjectProjection, RustType, TypeProjection,
};
use crate::render::docs;
use crate::schema::canonical::{FieldDefinition, SchemaCoordinate, SchemaName, TypeDefinition};

use super::visible::VisibleSchemaPlan;

/// Where a set of bindings is compiled, which decides what a binding may name beside
/// the SDK and its sibling bindings.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BindingSurface {
    /// A library of bindings with no query root of its own.
    Library,
    /// A module's `dagger_generated`, beside the `ModuleQuery` and `ModuleLoadable`
    /// that the module compiler renders there. A dependency's bindings add its
    /// constructor to that query root and make its handles loadable from an ID.
    Module,
}

/// One rendered dependency type: its wire name, the Rust module that holds it, and the
/// module's source below the generated header.
pub(super) struct DependencyBinding {
    pub(super) wire_name: String,
    pub(super) module_name: String,
    pub(super) body: String,
}

/// Renders the typed binding of `projection` when a dependency owns the type.
///
/// Returns `None` for every other type, which the caller renders as before.
pub(super) fn render_dependency_type(
    plan: &VisibleSchemaPlan,
    projection: &TypeProjection,
    sdk: &str,
    surface: BindingSurface,
) -> Result<Option<DependencyBinding>, DiagnosticSet> {
    let renderer = Renderer::new(plan, sdk, surface).map_err(DiagnosticSet::one)?;
    let rendered = match projection {
        TypeProjection::Object(object) if plan.is_dependency_type(&object.wire_name) => Some((
            object.wire_name.to_string(),
            object.module_name.clone(),
            renderer.object(object),
        )),
        TypeProjection::Interface(interface) if plan.is_dependency_type(&interface.wire_name) => {
            Some((
                interface.wire_name.to_string(),
                interface.module_name.clone(),
                renderer.interface(interface),
            ))
        }
        TypeProjection::Enum(enumeration) if plan.is_dependency_type(&enumeration.wire_name) => {
            let module_name = plan
                .projection()
                .names()
                .get(&enumeration.coordinate, crate::naming::NameContext::Module)
                .map(|name| name.identifier.clone())
                .ok_or_else(|| {
                    DiagnosticSet::one(error(
                        DiagnosticCode::RustNameInvalid,
                        &enumeration.coordinate,
                        "dependency enum has no projected module name",
                    ))
                })?;
            Some((
                enumeration.wire_name.to_string(),
                module_name,
                renderer.enumeration(enumeration),
            ))
        }
        _ => None,
    };
    let Some((wire_name, module_name, tokens)) = rendered else {
        return Ok(None);
    };
    Ok(Some(DependencyBinding {
        wire_name,
        module_name,
        body: tokens.map_err(DiagnosticSet::one)?.to_string(),
    }))
}

/// How a method reaches the query it extends and returns its future.
#[derive(Clone, Copy, Eq, PartialEq)]
enum MethodForm {
    /// An inherent `async fn`, whose body borrows nothing past the call.
    Inherent,
    /// A trait method that returns `impl Future + Send`.
    Trait,
}

struct Renderer<'a> {
    plan: &'a VisibleSchemaPlan,
    sdk: Ident,
    surface: BindingSurface,
}

impl<'a> Renderer<'a> {
    fn new(
        plan: &'a VisibleSchemaPlan,
        sdk: &str,
        surface: BindingSurface,
    ) -> Result<Self, Diagnostic> {
        let sdk = syn::parse_str::<Ident>(sdk).map_err(|_| {
            Diagnostic::new(
                DiagnosticCode::RustNameInvalid,
                Some(DiagnosticCoordinate::new("operation.authoring")),
                "the module's SDK dependency name is not a Rust identifier",
            )
        })?;
        Ok(Self { plan, sdk, surface })
    }

    fn object(&self, object: &ObjectProjection) -> Result<TokenStream, Diagnostic> {
        let name = ident(&object.rust_name, &object.coordinate)?;
        let definition = match self.plan.canonical().types().get(&object.wire_name) {
            Some(TypeDefinition::Object(definition)) => definition,
            _ => return Err(lost(&object.coordinate, "object")),
        };
        let attributes = public_attributes(
            &object.coordinate,
            definition.description.as_deref(),
            &format!("Lazy handle for GraphQL object `{}`.", object.wire_name),
            None,
            None,
        )?;
        let fields = self.fields(&object.fields)?;
        let options = self.options(&object.wire_name, &fields)?;
        let support = self.handle_support(&object.wire_name, &name, object.has_id)?;
        let methods = self.inherent_methods(&fields)?;
        let interfaces = self.interface_impls(&object.wire_name, &name)?;
        let loadable = self.loadable(&object.wire_name, &name, object.has_id);
        let constructors = self.constructors(object)?;
        let sdk = &self.sdk;
        Ok(quote! {
            #attributes
            #[derive(Clone)]
            pub struct #name {
                query: #sdk::QueryBuilder,
            }

            #options
            #support
            #loadable

            impl #name {
                #[doc(hidden)]
                pub fn __from_query(query: #sdk::QueryBuilder) -> Self {
                    Self { query }
                }

                #methods
            }

            #interfaces
            #constructors
        })
    }

    fn interface(&self, interface: &InterfaceProjection) -> Result<TokenStream, Diagnostic> {
        let trait_name = ident(&interface.trait_name, &interface.coordinate)?;
        let client_name = ident(&interface.client_name, &interface.coordinate)?;
        let definition = match self.plan.canonical().types().get(&interface.wire_name) {
            Some(TypeDefinition::Interface(definition)) => definition,
            _ => return Err(lost(&interface.coordinate, "interface")),
        };
        let trait_attributes = public_attributes(
            &interface.coordinate,
            definition.description.as_deref(),
            &format!(
                "Generated trait for GraphQL interface `{}`.",
                interface.wire_name
            ),
            None,
            None,
        )?;
        let client_attributes = public_attributes(
            &interface.coordinate,
            None,
            &format!(
                "Lazy client handle for GraphQL interface `{}`.",
                interface.wire_name
            ),
            None,
            None,
        )?;
        let fields = self.fields(&interface.fields)?;
        let options = self.options(&interface.wire_name, &fields)?;
        let support = self.handle_support(&interface.wire_name, &client_name, interface.has_id)?;
        let declarations = fields
            .iter()
            .map(|field| self.trait_declaration(field))
            .collect::<Result<Vec<_>, _>>()?;
        let methods = self.inherent_methods(&fields)?;
        let own_impl = self.trait_impl(&trait_name, &client_name, &fields)?;
        let parents = self.interface_impls(&interface.wire_name, &client_name)?;
        let loadable = self.loadable(&interface.wire_name, &client_name, interface.has_id);
        let sdk = &self.sdk;
        Ok(quote! {
            #trait_attributes
            pub trait #trait_name: ::core::clone::Clone + ::core::marker::Send + ::core::marker::Sync {
                #(#declarations)*
            }

            #client_attributes
            #[derive(Clone)]
            pub struct #client_name {
                query: #sdk::QueryBuilder,
            }

            #options
            #support
            #loadable

            impl #client_name {
                #[doc(hidden)]
                pub fn __from_query(query: #sdk::QueryBuilder) -> Self {
                    Self { query }
                }

                #methods
            }

            #own_impl
            #parents
        })
    }

    fn enumeration(&self, enumeration: &EnumProjection) -> Result<TokenStream, Diagnostic> {
        let name = ident(&enumeration.rust_name, &enumeration.coordinate)?;
        let definition = match self.plan.canonical().types().get(&enumeration.wire_name) {
            Some(TypeDefinition::Enum(definition)) => definition,
            _ => return Err(lost(&enumeration.coordinate, "enum")),
        };
        let attributes = public_attributes(
            &enumeration.coordinate,
            definition.description.as_deref(),
            &format!("Generated enum for GraphQL `{}`.", enumeration.wire_name),
            None,
            None,
        )?;
        let mut variants = Vec::new();
        for variant in enumeration.variants.values() {
            let variant_name = ident(&variant.rust_name, &variant.coordinate)?;
            let aliases = enumeration
                .aliases
                .values()
                .filter(|alias| alias.canonical_wire_name == variant.wire_name)
                .map(|alias| alias.wire_name.as_str());
            let wire_name = variant.wire_name.as_str();
            let variant_attributes = public_attributes(
                &variant.coordinate,
                variant.description.as_deref(),
                &format!("GraphQL enum value `{wire_name}`."),
                variant.deprecation.as_deref(),
                variant.experimental.as_deref(),
            )?;
            variants.push(quote! {
                #variant_attributes
                #[serde(rename = #wire_name #(, alias = #aliases)*)]
                #variant_name,
            });
        }
        let sdk = &self.sdk;
        // The derives expand to paths into `serde`, which a module's crate does not
        // depend on, so they are pointed at the SDK's re-export.
        let serde_path = format!("{sdk}::__private::serde");
        Ok(quote! {
            #attributes
            #[derive(
                Clone,
                Copy,
                Debug,
                Eq,
                Hash,
                PartialEq,
                #sdk::__private::serde::Deserialize,
                #sdk::__private::serde::Serialize,
            )]
            #[serde(crate = #serde_path)]
            pub enum #name {
                #(#variants)*
            }
        })
    }

    /// The public fields of one owner, in coordinate order.
    fn fields(
        &self,
        coordinates: &std::collections::BTreeSet<SchemaCoordinate>,
    ) -> Result<Vec<&'a FieldProjection>, Diagnostic> {
        let mut fields = Vec::new();
        for coordinate in coordinates {
            let field = self
                .plan
                .projection()
                .fields()
                .get(coordinate)
                .ok_or_else(|| lost(coordinate, "field"))?;
            // Single-underscore engine metadata has no public Rust operation.
            if !matches!(field.strategy, FieldStrategy::TargetPrivate) {
                fields.push(field);
            }
        }
        Ok(fields)
    }

    fn options(
        &self,
        owner: &SchemaName,
        fields: &[&FieldProjection],
    ) -> Result<TokenStream, Diagnostic> {
        let mut rendered = Vec::new();
        for field in fields {
            let Some(type_name) = field.options_type_name.as_deref() else {
                continue;
            };
            let name = ident(type_name, &field.coordinate)?;
            let type_doc = docs::documentation(
                &field.coordinate,
                None,
                &format!(
                    "Owned optional arguments for GraphQL operation `{}.{}`; reuse does not mutate caller state.",
                    owner, field.wire_name
                ),
            )?;
            let definition = self.field_definition(field)?;
            let mut members = Vec::new();
            let mut setters = Vec::new();
            for argument in field
                .arguments
                .iter()
                .filter(|argument| argument.presence.is_omittable())
            {
                let member = ident(&argument.rust_name, &argument.coordinate)?;
                let value_type = self.rust_type(&argument.rust_type, &argument.coordinate)?;
                let canonical = definition
                    .arguments
                    .get(&argument.wire_name)
                    .ok_or_else(|| lost(&argument.coordinate, "argument"))?;
                let member_doc = option_documentation(argument, canonical.description.as_deref())?;
                members.push(quote! {
                    #[cfg_attr(not(doctest), doc = #member_doc)]
                    pub #member: ::core::option::Option<#value_type>,
                });
                let setter = ident(
                    &format!("with_{}", argument.rust_name.trim_start_matches("r#")),
                    &argument.coordinate,
                )?;
                let setter_doc = docs::documentation(
                    &argument.coordinate,
                    None,
                    &format!(
                        "Sets GraphQL argument `{}` to a concrete value instead of omitting it.",
                        argument.wire_name
                    ),
                )?;
                let (parameter, value) = match &argument.rust_type {
                    RustType::String => (
                        quote! { impl ::core::convert::Into<::std::string::String> },
                        quote! { ::core::convert::Into::into(value) },
                    ),
                    RustType::Vec(inner) if **inner == RustType::String => (
                        quote! {
                            ::std::vec::Vec<impl ::core::convert::Into<::std::string::String>>
                        },
                        quote! {
                            ::core::iter::Iterator::collect(::core::iter::Iterator::map(
                                ::core::iter::IntoIterator::into_iter(value),
                                ::core::convert::Into::into,
                            ))
                        },
                    ),
                    _ => (value_type.clone(), quote! { value }),
                };
                setters.push(quote! {
                    #[cfg_attr(not(doctest), doc = #setter_doc)]
                    #[must_use]
                    pub fn #setter(mut self, value: #parameter) -> Self {
                        self.#member = ::core::option::Option::Some(#value);
                        self
                    }
                });
            }
            rendered.push(quote! {
                #[cfg_attr(not(doctest), doc = #type_doc)]
                #[derive(Clone, Debug, Default)]
                #[non_exhaustive]
                pub struct #name {
                    #(#members)*
                }

                impl #name {
                    #(#setters)*
                }
            });
        }
        Ok(quote! { #(#rendered)* })
    }

    /// The conversions that let a handle be passed wherever its ID is expected.
    fn handle_support(
        &self,
        wire_name: &SchemaName,
        handle: &Ident,
        has_id: bool,
    ) -> Result<TokenStream, Diagnostic> {
        if !has_id {
            return Ok(TokenStream::new());
        }
        let sdk = &self.sdk;
        let mut targets = vec![quote! { #handle }];
        for edge in self
            .plan
            .projection()
            .implementations()
            .iter()
            .filter(|edge| &edge.implementor == wire_name)
        {
            targets.push(self.type_path(&edge.interface, &edge.coordinate)?);
        }
        Ok(quote! {
            impl #sdk::IntoID<#sdk::Id> for #handle {
                fn into_id(
                    self,
                ) -> ::core::pin::Pin<
                    ::std::boxed::Box<
                        dyn ::core::future::Future<
                                Output = ::core::result::Result<#sdk::Id, #sdk::QueryError>,
                            > + ::core::marker::Send,
                    >,
                > {
                    ::std::boxed::Box::pin(async move { self.id().await })
                }
            }

            #(
                impl ::core::convert::From<#handle> for #sdk::IdInput<#targets> {
                    fn from(value: #handle) -> Self {
                        #sdk::IdInput::generated_lazy(value)
                    }
                }
            )*
        })
    }

    fn inherent_methods(&self, fields: &[&FieldProjection]) -> Result<TokenStream, Diagnostic> {
        refuse_duplicate_methods(fields)?;
        let methods = fields
            .iter()
            .map(|field| self.inherent_method(field, &quote! { self.query }))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(quote! { #(#methods)* })
    }

    /// Renders the method of `field`, and its `_opts` variant, on the type whose query
    /// is `receiver`.
    fn inherent_method(
        &self,
        field: &FieldProjection,
        receiver: &TokenStream,
    ) -> Result<TokenStream, Diagnostic> {
        let sdk = &self.sdk;
        let attributes = self.method_attributes(field)?;
        let name = ident(&method_name(field), &field.coordinate)?;
        let opts = options_parameter(field)?;
        let required = self.required_parameters(field, MethodForm::Inherent)?;
        let output = self.output_type(field)?;
        let lazy = matches!(field.strategy, FieldStrategy::LazyHandle { .. });
        let setup = self.setup(field, receiver, false)?;
        let body = self.execution(field)?;
        let ordinary = if lazy {
            quote! {
                #attributes
                #[must_use]
                pub fn #name(&self #(, #required)*) -> #output {
                    #setup
                    #body
                }
            }
        } else {
            quote! {
                #attributes
                pub async fn #name(&self #(, #required)*) -> ::core::result::Result<#output, #sdk::QueryError> {
                    #setup
                    #body
                }
            }
        };
        let Some((options_method, options_type)) = self.options_names(field)? else {
            return Ok(ordinary);
        };
        let options_attributes = self.options_method_attributes(field)?;
        let setup = self.setup(field, receiver, true)?;
        let options = if lazy {
            quote! {
                #options_attributes
                #[must_use]
                pub fn #options_method(&self, #(#required,)* #opts: &#options_type) -> #output {
                    #setup
                    #body
                }
            }
        } else {
            quote! {
                #options_attributes
                pub async fn #options_method(
                    &self,
                    #(#required,)*
                    #opts: &#options_type,
                ) -> ::core::result::Result<#output, #sdk::QueryError> {
                    #setup
                    #body
                }
            }
        };
        Ok(quote! { #ordinary #options })
    }

    fn trait_declaration(&self, field: &FieldProjection) -> Result<TokenStream, Diagnostic> {
        let sdk = &self.sdk;
        let attributes = self.method_attributes(field)?;
        let name = ident(&method_name(field), &field.coordinate)?;
        let opts = options_parameter(field)?;
        let required = self.required_parameters(field, MethodForm::Trait)?;
        let output = self.output_type(field)?;
        let lazy = matches!(field.strategy, FieldStrategy::LazyHandle { .. });
        let ordinary = if lazy {
            quote! {
                #attributes
                #[must_use]
                fn #name(&self #(, #required)*) -> #output;
            }
        } else {
            quote! {
                #attributes
                fn #name(&self #(, #required)*)
                    -> impl ::core::future::Future<Output = ::core::result::Result<#output, #sdk::QueryError>> + ::core::marker::Send;
            }
        };
        let Some((options_method, options_type)) = self.options_names(field)? else {
            return Ok(ordinary);
        };
        let options_attributes = self.options_method_attributes(field)?;
        let options = if lazy {
            quote! {
                #options_attributes
                #[must_use]
                fn #options_method(&self, #(#required,)* #opts: &#options_type) -> #output;
            }
        } else {
            quote! {
                #options_attributes
                fn #options_method(
                    &self,
                    #(#required,)*
                    #opts: &#options_type,
                ) -> impl ::core::future::Future<Output = ::core::result::Result<#output, #sdk::QueryError>> + ::core::marker::Send;
            }
        };
        Ok(quote! { #ordinary #options })
    }

    fn trait_impl(
        &self,
        trait_name: &Ident,
        implementor: &Ident,
        fields: &[&FieldProjection],
    ) -> Result<TokenStream, Diagnostic> {
        let methods = fields
            .iter()
            .map(|field| self.trait_impl_method(field))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(quote! {
            impl super::#trait_name for #implementor {
                #(#methods)*
            }
        })
    }

    fn trait_impl_method(&self, field: &FieldProjection) -> Result<TokenStream, Diagnostic> {
        let sdk = &self.sdk;
        let name = ident(&method_name(field), &field.coordinate)?;
        let opts = options_parameter(field)?;
        let required = self.required_parameters(field, MethodForm::Trait)?;
        let output = self.output_type(field)?;
        let lazy = matches!(field.strategy, FieldStrategy::LazyHandle { .. });
        let body = self.execution(field)?;
        // The request is built from the arguments before the future exists, so the
        // future owns its query and borrows neither the handle nor the options.
        let render = |setup: TokenStream, signature: TokenStream| {
            if lazy {
                quote! {
                    #signature -> #output {
                        #setup
                        #body
                    }
                }
            } else {
                quote! {
                    #signature
                        -> impl ::core::future::Future<Output = ::core::result::Result<#output, #sdk::QueryError>> + ::core::marker::Send
                    {
                        #setup
                        async move { #body }
                    }
                }
            }
        };
        let ordinary = render(
            self.setup(field, &quote! { self.query }, false)?,
            quote! { fn #name(&self #(, #required)*) },
        );
        let Some((options_method, options_type)) = self.options_names(field)? else {
            return Ok(ordinary);
        };
        let options = render(
            self.setup(field, &quote! { self.query }, true)?,
            quote! { fn #options_method(&self, #(#required,)* #opts: &super::#options_type) },
        );
        Ok(quote! { #ordinary #options })
    }

    /// The trait implementations a handle owes the dependency interfaces it declares.
    ///
    /// A Core interface's trait is implemented only inside the SDK, so a handle
    /// converts to such an interface's ID input and does not implement its trait.
    fn interface_impls(
        &self,
        implementor_wire_name: &SchemaName,
        implementor: &Ident,
    ) -> Result<TokenStream, Diagnostic> {
        let mut implementations = Vec::new();
        for edge in self
            .plan
            .projection()
            .implementations()
            .iter()
            .filter(|edge| &edge.implementor == implementor_wire_name)
        {
            if !self.plan.is_dependency_type(&edge.interface) {
                continue;
            }
            let interface = match self.plan.projection().named_types().get(&edge.interface) {
                Some(TypeProjection::Interface(interface)) => interface,
                _ => return Err(lost(&edge.coordinate, "interface edge")),
            };
            let trait_name = ident(&interface.trait_name, &edge.coordinate)?;
            let fields = self.fields(&interface.fields)?;
            implementations.push(self.trait_impl(&trait_name, implementor, &fields)?);
        }
        Ok(quote! { #(#implementations)* })
    }

    /// The constructors of `object` on the module's query root: one method for each
    /// field the object's dependency adds to `Query`.
    ///
    /// The engine installs a dependency's constructor as a field of `Query` named
    /// after the dependency, with the arguments of the constructor its author wrote
    /// (`ModuleObject.Install` in core/object.go at the target). Dagger's Go SDK
    /// renders it as a method on the query root that its `dag` embeds
    /// (cmd/codegen/generator/go/templates/src/internal/dagger/_dep.gen.go.tmpl). The
    /// field is recognised by what it returns, since `Query` itself is a Core type:
    /// a field outside the reviewed Core API that returns a dependency's object.
    fn constructors(&self, object: &ObjectProjection) -> Result<TokenStream, Diagnostic> {
        if self.surface != BindingSurface::Module {
            return Ok(TokenStream::new());
        }
        let fields = constructor_fields(self.plan)
            .filter(|(_, target)| *target == &object.wire_name)
            .map(|(field, _)| field)
            .collect::<Vec<_>>();
        if fields.is_empty() {
            return Ok(TokenStream::new());
        }
        let options = self.options(self.plan.canonical().query(), &fields)?;
        let methods = fields
            .iter()
            .map(|field| self.inherent_method(field, &quote! { self.__query_builder() }))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(quote! {
            #options

            impl super::ModuleQuery {
                #(#methods)*
            }
        })
    }

    /// Makes a handle loadable from an ID through the module's query root.
    ///
    /// `ModuleQuery::r#ref` and `ModuleQuery::load` are generic over the trait this
    /// implements, as `Ref` and `Load` of Dagger's Go SDK are generic over every
    /// generated object and interface client
    /// (cmd/codegen/generator/go/templates/src/_dagger.gen.go/defs.go.tmpl). A
    /// reference keeps no ID, since the caller's ID need not be in the form `id`
    /// returns. A load asks for `id` through the type's inline fragment, which fails
    /// for a node that is missing or of another type, and keeps the ID it is given
    /// back.
    fn loadable(&self, wire_name: &SchemaName, handle: &Ident, has_id: bool) -> TokenStream {
        if self.surface != BindingSurface::Module || !has_id {
            return TokenStream::new();
        }
        let sdk = &self.sdk;
        let graphql_name = wire_name.as_str();
        quote! {
            impl super::ModuleLoadable for #handle {
                fn __module_ref(__query: &#sdk::QueryBuilder, __id: #sdk::IdInput<Self>) -> Self {
                    Self::__from_query(
                        __query.generated_lazy_reentry_builder(__id, #graphql_name),
                    )
                }

                fn __module_load(
                    __query: &#sdk::QueryBuilder,
                    __id: #sdk::IdInput<Self>,
                ) -> impl ::core::future::Future<
                    Output = ::core::result::Result<Self, #sdk::QueryError>,
                > + ::core::marker::Send {
                    let __query = ::core::clone::Clone::clone(__query);
                    async move {
                        let __id: #sdk::Id = __query
                            .generated_lazy_reentry_builder(__id, #graphql_name)
                            .select("id")
                            .execute()
                            .await?;
                        ::core::result::Result::Ok(Self::__from_query(
                            __query.generated_reentry_builder(__id, #graphql_name),
                        ))
                    }
                }
            }
        }
    }

    fn required_parameters(
        &self,
        field: &FieldProjection,
        form: MethodForm,
    ) -> Result<Vec<TokenStream>, Diagnostic> {
        let send = (form == MethodForm::Trait).then(|| quote! { + ::core::marker::Send });
        field
            .arguments
            .iter()
            .filter(|argument| argument.presence == ArgumentPresence::Required)
            .map(|argument| {
                let name = ident(&argument.rust_name, &argument.coordinate)?;
                let value_type = self.rust_type(&argument.rust_type, &argument.coordinate)?;
                Ok(match &argument.rust_type {
                    RustType::String => quote! {
                        #name: impl ::core::convert::Into<::std::string::String> #send
                    },
                    RustType::Vec(inner) if **inner == RustType::String => quote! {
                        #name: ::std::vec::Vec<
                            impl ::core::convert::Into<::std::string::String> #send
                        >
                    },
                    RustType::IdInput(_) => quote! {
                        #name: impl ::core::convert::Into<#value_type> #send
                    },
                    _ => quote! { #name: #value_type },
                })
            })
            .collect()
    }

    /// Selects the field on `receiver` and records its arguments in wire-name order.
    ///
    /// A value is encoded as it is recorded. An ID input is kept and resolved when the
    /// request is built, so a handle passed as an argument costs no request until then.
    fn setup(
        &self,
        field: &FieldProjection,
        receiver: &TokenStream,
        include_options: bool,
    ) -> Result<TokenStream, Diagnostic> {
        let wire_name = field.wire_name.as_str();
        let opts = options_parameter(field)?;
        let mut statements = Vec::new();
        for argument in &field.arguments {
            let name = ident(&argument.rust_name, &argument.coordinate)?;
            let argument_wire_name = argument.wire_name.as_str();
            match &argument.presence {
                ArgumentPresence::Required => {
                    if argument.encoder.contains_lazy_id() {
                        let value = if matches!(argument.rust_type, RustType::IdInput(_)) {
                            let target =
                                self.rust_type(&argument.rust_type, &argument.coordinate)?;
                            quote! { ::core::convert::Into::<#target>::into(#name) }
                        } else {
                            quote! { #name }
                        };
                        statements.push(quote! {
                            let __query =
                                __query.generated_argument_id_shape(#argument_wire_name, #value);
                        });
                    } else if matches!(argument.rust_type, RustType::String) {
                        statements.push(quote! {
                            let __query = __query.argument(
                                #argument_wire_name,
                                ::core::convert::Into::<::std::string::String>::into(#name),
                            );
                        });
                    } else if matches!(
                        &argument.rust_type,
                        RustType::Vec(inner) if **inner == RustType::String
                    ) {
                        statements.push(quote! {
                            let #name = ::core::iter::Iterator::collect::<
                                ::std::vec::Vec<::std::string::String>,
                            >(::core::iter::Iterator::map(
                                ::core::iter::IntoIterator::into_iter(#name),
                                ::core::convert::Into::into,
                            ));
                            let __query = __query.argument(#argument_wire_name, #name);
                        });
                    } else {
                        statements.push(quote! {
                            let __query = __query.argument(#argument_wire_name, #name);
                        });
                    }
                }
                ArgumentPresence::Omittable { .. } if include_options => {
                    if argument.encoder.contains_lazy_id() {
                        statements.push(quote! {
                            let __query =
                                if let ::core::option::Option::Some(__value) = &#opts.#name {
                                    __query.generated_argument_id_shape(
                                        #argument_wire_name,
                                        ::core::clone::Clone::clone(__value),
                                    )
                                } else {
                                    __query
                                };
                        });
                    } else {
                        statements.push(quote! {
                            let __query =
                                if let ::core::option::Option::Some(__value) = &#opts.#name {
                                    __query.argument(#argument_wire_name, __value)
                                } else {
                                    __query
                                };
                        });
                    }
                }
                ArgumentPresence::Omittable { .. } => {}
            }
        }
        Ok(quote! {
            let __query = #receiver.select(#wire_name);
            #(#statements)*
        })
    }

    /// The expression a method ends with, by the field's projected strategy.
    fn execution(&self, field: &FieldProjection) -> Result<TokenStream, Diagnostic> {
        let sdk = &self.sdk;
        match &field.strategy {
            FieldStrategy::LazyHandle { target } => self.handle(target, &field.coordinate, None),
            FieldStrategy::NullableHandle { target, .. }
            | FieldStrategy::ReenterList { target, .. } => {
                let ids = self.id_shape(&field.return_type, &field.coordinate)?;
                let reentered =
                    self.reentry(target, &field.return_type, quote! { __ids }, 0, field)?;
                Ok(quote! {
                    let __ids: #ids = __query.select("id").execute().await?;
                    ::core::result::Result::Ok(#reentered)
                })
            }
            // The engine answers a `Void` field with an empty object, which a strict
            // unit decode refuses, so the SDK gives the type its own execution path.
            FieldStrategy::ExecuteValue {
                output: RustType::Unit,
            } => Ok(quote! { __query.execute_void().await }),
            FieldStrategy::ExecuteValue { .. } => Ok(quote! { __query.execute().await }),
            FieldStrategy::ExpectedTypeSelf { parent, .. } => {
                let reentered = self.handle(parent, &field.coordinate, Some(quote! { __id }))?;
                Ok(quote! {
                    let __id: #sdk::Id = __query.execute().await?;
                    ::core::result::Result::Ok(#reentered)
                })
            }
            FieldStrategy::TargetPrivate => Err(error(
                DiagnosticCode::SchemaFieldUnmapped,
                &field.coordinate,
                "target-private field reached the dependency renderer",
            )),
        }
    }

    /// Builds the handle of `target` on `query`, or on the `node(id:)` that loads
    /// `returned_id`.
    ///
    /// `returned_id` is an ID this field's own query returned for the object, so the
    /// builder keeps it and the handle answers `id` without another request.
    fn handle(
        &self,
        target: &SchemaName,
        coordinate: &SchemaCoordinate,
        returned_id: Option<TokenStream>,
    ) -> Result<TokenStream, Diagnostic> {
        let target_type = self.type_path(target, coordinate)?;
        let graphql_name = target.as_str();
        let builder = match returned_id {
            Some(id) => quote! { __query.generated_reentry_builder(#id, #graphql_name) },
            None => quote! { __query },
        };
        if self.plan.is_dependency_type(target) {
            Ok(quote! { #target_type::__from_query(#builder) })
        } else {
            // A Core handle's constructor is sealed inside the SDK, which builds it
            // from the builder's session and selection.
            Ok(quote! { #builder.generated_core_handle::<#target_type>() })
        }
    }

    fn reentry(
        &self,
        target: &SchemaName,
        rust_type: &RustType,
        value: TokenStream,
        depth: usize,
        field: &FieldProjection,
    ) -> Result<TokenStream, Diagnostic> {
        match rust_type {
            RustType::Handle(actual) | RustType::InterfaceHandle(actual) if actual == target => {
                self.handle(target, &field.coordinate, Some(value))
            }
            RustType::Option(inner) => {
                let nested = ident(&format!("__value_{depth}"), &field.coordinate)?;
                let expression =
                    self.reentry(target, inner, quote! { #nested }, depth + 1, field)?;
                Ok(quote! { ::core::option::Option::map(#value, |#nested| #expression) })
            }
            RustType::Vec(inner) => {
                let nested = ident(&format!("__value_{depth}"), &field.coordinate)?;
                let expression =
                    self.reentry(target, inner, quote! { #nested }, depth + 1, field)?;
                Ok(quote! {
                    ::core::iter::Iterator::collect(::core::iter::Iterator::map(
                        ::core::iter::IntoIterator::into_iter(#value),
                        |#nested| #expression,
                    ))
                })
            }
            _ => Err(error(
                DiagnosticCode::SchemaFieldUnmapped,
                &field.coordinate,
                "handle re-entry strategy disagrees with its projected output",
            )),
        }
    }

    fn id_shape(
        &self,
        rust_type: &RustType,
        coordinate: &SchemaCoordinate,
    ) -> Result<TokenStream, Diagnostic> {
        let sdk = &self.sdk;
        match rust_type {
            RustType::Handle(_) | RustType::InterfaceHandle(_) => Ok(quote! { #sdk::Id }),
            RustType::Option(inner) => {
                let inner = self.id_shape(inner, coordinate)?;
                Ok(quote! { ::core::option::Option<#inner> })
            }
            RustType::Vec(inner) => {
                let inner = self.id_shape(inner, coordinate)?;
                Ok(quote! { ::std::vec::Vec<#inner> })
            }
            _ => Err(error(
                DiagnosticCode::SchemaFieldUnmapped,
                coordinate,
                "handle re-entry output contains a non-handle leaf",
            )),
        }
    }

    fn output_type(&self, field: &FieldProjection) -> Result<TokenStream, Diagnostic> {
        if let FieldStrategy::ExpectedTypeSelf { parent, .. } = &field.strategy {
            return self.type_path(parent, &field.coordinate);
        }
        self.rust_type(&field.return_type, &field.coordinate)
    }

    fn rust_type(
        &self,
        rust_type: &RustType,
        coordinate: &SchemaCoordinate,
    ) -> Result<TokenStream, Diagnostic> {
        let sdk = &self.sdk;
        Ok(match rust_type {
            RustType::Bool => quote! { bool },
            RustType::F64 => quote! { f64 },
            RustType::I64 => quote! { i64 },
            RustType::String => quote! { ::std::string::String },
            RustType::Id => quote! { #sdk::Id },
            RustType::Json => quote! { #sdk::Json },
            RustType::Platform => quote! { #sdk::Platform },
            RustType::Unit => quote! { () },
            RustType::Enum(name)
            | RustType::Input(name)
            | RustType::CustomScalar(name)
            | RustType::Handle(name)
            | RustType::InterfaceHandle(name) => self.type_path(name, coordinate)?,
            RustType::IdInput(name) => {
                let target = self.type_path(name, coordinate)?;
                quote! { #sdk::IdInput<#target> }
            }
            RustType::Option(inner) => {
                let inner = self.rust_type(inner, coordinate)?;
                quote! { ::core::option::Option<#inner> }
            }
            RustType::Vec(inner) => {
                let inner = self.rust_type(inner, coordinate)?;
                quote! { ::std::vec::Vec<#inner> }
            }
        })
    }

    /// The path of a named type's typed Rust binding: a sibling module's for a
    /// dependency's type, and the SDK's for a reviewed Core type.
    ///
    /// `coordinate` is the field or argument that names the type, which the diagnostic
    /// reports when the type has no typed binding.
    fn type_path(
        &self,
        name: &SchemaName,
        coordinate: &SchemaCoordinate,
    ) -> Result<TokenStream, Diagnostic> {
        let projection = self
            .plan
            .projection()
            .named_types()
            .get(name)
            .ok_or_else(|| lost(coordinate, "named type"))?;
        let rust_name = match projection {
            TypeProjection::Object(object) => object.rust_name.as_str(),
            TypeProjection::Interface(interface) => interface.client_name.as_str(),
            TypeProjection::Enum(enumeration) => enumeration.rust_name.as_str(),
            TypeProjection::InputObject(input) => input.rust_name.as_str(),
            TypeProjection::Scalar(scalar) => self
                .plan
                .projection()
                .names()
                .get(&scalar.coordinate, crate::naming::NameContext::Type)
                .map(|name| name.identifier.as_str())
                .ok_or_else(|| lost(coordinate, "scalar"))?,
            TypeProjection::TargetPrivate(_) => {
                return Err(error(
                    DiagnosticCode::SchemaFieldUnmapped,
                    coordinate,
                    "dependency coordinate names a target-private type",
                ));
            }
        };
        let rust_name = ident(rust_name, coordinate)?;
        if self.plan.is_dependency_type(name) {
            return Ok(quote! { super::#rust_name });
        }
        if self.plan.is_extension_type(name) {
            // Outside the reviewed Core API and owned by no dependency: a type a later
            // engine added, bound as an untyped handle that no typed call can accept
            // or return.
            return Err(error(
                DiagnosticCode::SchemaFieldUnmapped,
                coordinate,
                &format!(
                    "dependency coordinate names `{name}`, which is neither a reviewed Core type nor a type of a module dependency"
                ),
            ));
        }
        let sdk = &self.sdk;
        Ok(quote! { #sdk::#rust_name })
    }

    fn options_names(&self, field: &FieldProjection) -> Result<Option<(Ident, Ident)>, Diagnostic> {
        let (Some(method), Some(type_name)) = (
            field.options_method_name.as_deref(),
            field.options_type_name.as_deref(),
        ) else {
            return Ok(None);
        };
        Ok(Some((
            ident(method, &field.coordinate)?,
            ident(type_name, &field.coordinate)?,
        )))
    }

    fn method_attributes(&self, field: &FieldProjection) -> Result<TokenStream, Diagnostic> {
        let canonical = self.field_definition(field)?;
        let mut description = canonical.description.clone().unwrap_or_default();
        if !description.is_empty() {
            description.push_str("\n\n");
        }
        description.push_str(&format!(
            "Selects GraphQL field `{}` on `{}`.",
            field.wire_name, field.owner
        ));
        public_attributes(
            &field.coordinate,
            Some(&description),
            &description,
            field.deprecation.as_deref(),
            field.experimental.as_deref(),
        )
    }

    fn options_method_attributes(
        &self,
        field: &FieldProjection,
    ) -> Result<TokenStream, Diagnostic> {
        let options_name = field.options_type_name.as_deref().unwrap_or_default();
        public_attributes(
            &field.coordinate,
            None,
            &format!(
                "Executes GraphQL operation `{}` with a borrowed, reusable `{options_name}` value.",
                field.wire_name
            ),
            field.deprecation.as_deref(),
            field.experimental.as_deref(),
        )
    }

    fn field_definition(&self, field: &FieldProjection) -> Result<&'a FieldDefinition, Diagnostic> {
        self.plan
            .canonical()
            .types()
            .get(&field.owner)
            .and_then(|definition| match definition {
                TypeDefinition::Object(definition) => definition.fields.get(&field.wire_name),
                TypeDefinition::Interface(definition) => definition.fields.get(&field.wire_name),
                _ => None,
            })
            .ok_or_else(|| lost(&field.coordinate, "field"))
    }
}

/// The names of methods every handle already has through a trait of the standard
/// prelude: `Clone` (handles derive it), `ToOwned` (through `Clone`), and `Into` and
/// `TryInto` (through the reflexive `From`). An inherent method of one of these names
/// would take the trait's place in a method call: `handle.clone()` would select the
/// dependency's `clone` field, with the same return type, and nothing would flag it.
const PRELUDE_METHODS: &[&str] = &[
    "clone",
    "clone_from",
    "clone_into",
    "into",
    "to_owned",
    "try_into",
];

/// The Rust name of the method that binds `field`: the field's projected name, with an
/// underscore appended when a handle already has a method of that name (see
/// [`PRELUDE_METHODS`]). The `_opts` method keeps the projected name, which no trait
/// method has.
pub(super) fn method_name(field: &FieldProjection) -> String {
    if PRELUDE_METHODS.contains(&field.rust_name.as_str()) {
        format!("{}_", field.rust_name)
    } else {
        field.rust_name.clone()
    }
}

/// The name of the parameter that carries a field's optional arguments: `opts`, with
/// underscores appended while a required argument of the field has that name.
fn options_parameter(field: &FieldProjection) -> Result<Ident, Diagnostic> {
    let mut name = "opts".to_owned();
    while field.arguments.iter().any(|argument| {
        argument.presence == ArgumentPresence::Required && argument.rust_name == name
    }) {
        name.push('_');
    }
    ident(&name, &field.coordinate)
}

/// Refuses two methods of one owner with the same Rust name.
///
/// The projection's name map already refuses two fields, or a field and another
/// field's `_opts`, that project to one name, and it projects a field the dependency
/// calls `clone_` to `clone`, so no name is known to meet one that [`method_name`]
/// escapes. This is the backstop for that: a duplicate would otherwise surface as a
/// compiler error in the module's generated code.
fn refuse_duplicate_methods(fields: &[&FieldProjection]) -> Result<(), Diagnostic> {
    let mut names = std::collections::BTreeMap::new();
    for field in fields {
        let escaped = method_name(field);
        for name in [Some(escaped), field.options_method_name.clone()]
            .into_iter()
            .flatten()
        {
            if let Some(previous) = names.insert(name.clone(), &field.coordinate)
                && previous != &field.coordinate
            {
                return Err(error(
                    DiagnosticCode::RustNameCollision,
                    &field.coordinate,
                    &format!(
                        "dependency function would be bound as `{name}`, as `{}` already is",
                        previous.as_str()
                    ),
                ));
            }
        }
    }
    Ok(())
}

/// The fields a module's dependencies add to `Query`, each with the dependency object
/// it constructs, in coordinate order.
pub(super) fn constructor_fields(
    plan: &VisibleSchemaPlan,
) -> impl Iterator<Item = (&FieldProjection, &SchemaName)> {
    plan.projection().fields().values().filter_map(|field| {
        if &field.owner != plan.canonical().query()
            || !plan.extension_coordinates().contains(&field.coordinate)
        {
            return None;
        }
        match &field.strategy {
            FieldStrategy::LazyHandle { target }
                if plan.is_dependency_type(target)
                    && matches!(
                        plan.projection().named_types().get(target),
                        Some(TypeProjection::Object(_))
                    ) =>
            {
                Some((field, target))
            }
            _ => None,
        }
    })
}

fn option_documentation(
    argument: &ArgumentProjection,
    description: Option<&str>,
) -> Result<String, Diagnostic> {
    let omission = match &argument.presence {
        ArgumentPresence::Required => "This argument is required.".to_owned(),
        ArgumentPresence::Omittable { engine_default } => match engine_default {
            Some(default) => format!(
                "`None` omits GraphQL field `{}` and preserves engine default `{default:?}`.",
                argument.wire_name
            ),
            None => format!("`None` omits GraphQL field `{}`.", argument.wire_name),
        },
    };
    let mut source = description.unwrap_or_default().to_owned();
    if !source.is_empty() {
        source.push_str("\n\n");
    }
    source.push_str(&omission);
    if let Some(reason) = argument.deprecation.as_deref() {
        source.push_str(&format!("\n\n**Deprecated:** {reason}"));
    }
    if let Some(reason) = argument.experimental.as_deref() {
        source.push_str(&format!("\n\n**Experimental:** {reason}"));
    }
    docs::documentation(&argument.coordinate, Some(&source), &omission)
}

fn public_attributes(
    coordinate: &SchemaCoordinate,
    description: Option<&str>,
    fallback: &str,
    deprecation: Option<&str>,
    experimental: Option<&str>,
) -> Result<TokenStream, Diagnostic> {
    let mut documentation = docs::documentation(coordinate, description, fallback)?;
    if let Some(reason) = deprecation {
        let reason = docs::sanitize(coordinate, reason)?;
        documentation.push_str(&format!("\n\n**Deprecated:** {reason}"));
    }
    if let Some(reason) = experimental {
        let reason = docs::sanitize(coordinate, reason)?;
        documentation.push_str(&format!("\n\n**Experimental:** {reason}"));
    }
    let deprecated = deprecation.map(|reason| {
        let note = reason.replace(['\r', '\n'], " ");
        quote! { #[deprecated(note = #note)] }
    });
    // A dependency's documentation is its author's text, and an indented or fenced
    // block in it is no Rust for the module's `cargo test` to compile as a doctest.
    Ok(quote! {
        #[cfg_attr(not(doctest), doc = #documentation)]
        #deprecated
    })
}

fn ident(source: &str, coordinate: &SchemaCoordinate) -> Result<Ident, Diagnostic> {
    syn::parse_str::<Ident>(source).map_err(|_| {
        error(
            DiagnosticCode::RustNameInvalid,
            coordinate,
            "projected Rust identifier is not valid syntax",
        )
    })
}

fn lost(coordinate: &SchemaCoordinate, kind: &str) -> Diagnostic {
    error(
        DiagnosticCode::GeneratedProvenanceInvalid,
        coordinate,
        &format!("{kind} projection lost its canonical definition"),
    )
}

fn error(code: DiagnosticCode, coordinate: &SchemaCoordinate, message: &str) -> Diagnostic {
    Diagnostic::new(
        code,
        Some(DiagnosticCoordinate::new(coordinate.as_str())),
        message,
    )
}
