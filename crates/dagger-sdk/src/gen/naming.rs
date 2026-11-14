//! Generated bindings owned by the GraphQL `Naming` type.
// @generated {"format":"dagger-rust-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:9511bc82fb704520cdc632da73463fc7396b1bb081e0446b61665c30764a5e7c","target_revision":"400b82d6f3c09ef096d2f59b3e920267b5969bf4"}
#[doc = "The engine's identifier naming rules at an engine version."]
#[derive(Clone)]
pub struct Naming {
    pub(crate) session: crate::lifecycle::SessionHandle,
    pub(crate) selection: crate::query::Selection,
}
#[doc = "Owned optional arguments for GraphQL operation `Naming.formatIdentifiers`; reuse does not mutate caller state."]
#[derive(Clone, Debug, Default)]
#[non_exhaustive]
pub struct NamingFormatIdentifiersOpts {
    #[doc = "How to write acronyms and terms where a word starts with a capital.\n\n`None` omits GraphQL field `acronyms` and preserves engine default `Enum(SchemaName(\"UPPERCASE\"))`."]
    pub acronyms: Option<super::AcronymStyle>,
}
impl NamingFormatIdentifiersOpts {
    #[doc = "Sets GraphQL argument `acronyms` to a concrete value instead of omitting it."]
    #[must_use]
    pub fn with_acronyms(mut self, value: super::AcronymStyle) -> Self {
        self.acronyms = Some(value);
        self
    }
}
#[doc = "Owned optional arguments for GraphQL operation `Naming.module`; reuse does not mutate caller state."]
#[derive(Clone, Debug, Default)]
#[non_exhaustive]
pub struct NamingModuleOpts {
    #[doc = "The module's original name, when it's loaded under another (as a dependency alias): its main object and namespaced type names are spelled after it. Defaults to name.\n\n`None` omits GraphQL field `originalName`."]
    pub original_name: Option<String>,
}
impl NamingModuleOpts {
    #[doc = "Sets GraphQL argument `originalName` to a concrete value instead of omitting it."]
    #[must_use]
    pub fn with_original_name(mut self, value: impl Into<String>) -> Self {
        self.original_name = Some(value.into());
        self
    }
}
impl crate::IntoID<crate::Id> for Naming {
    fn into_id(
        self,
    ) -> core::pin::Pin<
        Box<dyn core::future::Future<Output = Result<crate::Id, crate::QueryError>> + Send>,
    > {
        Box::pin(async move { self.id().await })
    }
}
impl crate::loadable::private::Sealed for Naming {
    fn graphql_type() -> &'static str {
        "Naming"
    }
    fn from_query(
        session: crate::lifecycle::SessionHandle,
        selection: crate::query::Selection,
    ) -> Self {
        Self { session, selection }
    }
}
impl From<Naming> for crate::IdInput<Naming> {
    fn from(value: Naming) -> Self {
        crate::IdInput::lazy(value)
    }
}
impl From<Naming> for crate::IdInput<super::NodeClient> {
    fn from(value: Naming) -> Self {
        crate::IdInput::lazy(value)
    }
}
impl Naming {
    #[doc = "The acronyms and terms used to parse and format identifiers.\n\nSelects GraphQL field `dictionary` on `Naming`."]
    pub async fn dictionary(&self) -> Result<Vec<super::NamingTerm>, crate::QueryError> {
        let query = self.selection.select("dictionary");
        let query = query.select("id");
        query
            .execute_reentry::<super::NamingTerm, Vec<crate::Id>>(&self.session, "NamingTerm")
            .await
    }
    #[doc = "Format many names at once, for codegen. Returns them in input order.\n\nSelects GraphQL field `formatIdentifiers` on `Naming`."]
    pub async fn format_identifiers(
        &self,
        casing: super::Casing,
        names: Vec<impl Into<String>>,
    ) -> Result<Vec<String>, crate::QueryError> {
        let query = self.selection.select("formatIdentifiers");
        let query = query.arg("casing", casing);
        let names = names.into_iter().map(Into::into).collect::<Vec<String>>();
        let query = query.arg("names", names);
        query.execute(&self.session).await
    }
    #[doc = "Executes GraphQL operation `formatIdentifiers` with a borrowed, reusable `NamingFormatIdentifiersOpts` value."]
    pub async fn format_identifiers_opts(
        &self,
        casing: super::Casing,
        names: Vec<impl Into<String>>,
        opts: &NamingFormatIdentifiersOpts,
    ) -> Result<Vec<String>, crate::QueryError> {
        let query = self.selection.select("formatIdentifiers");
        let query = if let Some(value) = &opts.acronyms {
            query.arg("acronyms", value)
        } else {
            query
        };
        let query = query.arg("casing", casing);
        let names = names.into_iter().map(Into::into).collect::<Vec<String>>();
        let query = query.arg("names", names);
        query.execute(&self.session).await
    }
    #[doc = "A unique identifier for this Naming.\n\nSelects GraphQL field `id` on `Naming`."]
    pub async fn id(&self) -> Result<crate::Id, crate::QueryError> {
        let query = self.selection.select("id");
        query.execute(&self.session).await
    }
    #[doc = "Parse a name in any casing into words.\n\nKnown acronyms and terms come from the naming dictionary; everything else falls back to the case heuristic. Errors on non-ASCII input, input with no letters or digits, or input over 256 bytes.\n\nSelects GraphQL field `identifier` on `Naming`."]
    #[must_use]
    pub fn identifier(&self, name: impl Into<String>) -> super::Identifier {
        let query = self.selection.select("identifier");
        let query = query.arg("name", name.into());
        super::Identifier {
            session: self.session.clone(),
            selection: query,
        }
    }
    #[doc = "The names the engine gives the declarations of a module named by these rules.\n\nSelects GraphQL field `module` on `Naming`."]
    #[must_use]
    pub fn module(&self, name: impl Into<String>) -> super::ModuleNaming {
        let query = self.selection.select("module");
        let query = query.arg("name", name.into());
        super::ModuleNaming {
            session: self.session.clone(),
            selection: query,
        }
    }
    #[doc = "Executes GraphQL operation `module` with a borrowed, reusable `NamingModuleOpts` value."]
    #[must_use]
    pub fn module_opts(
        &self,
        name: impl Into<String>,
        opts: &NamingModuleOpts,
    ) -> super::ModuleNaming {
        let query = self.selection.select("module");
        let query = query.arg("name", name.into());
        let query = if let Some(value) = &opts.original_name {
            query.arg("originalName", value)
        } else {
            query
        };
        super::ModuleNaming {
            session: self.session.clone(),
            selection: query,
        }
    }
    #[doc = "The normalized engine version these rules answer for.\n\nSelects GraphQL field `version` on `Naming`."]
    pub async fn version(&self) -> Result<String, crate::QueryError> {
        let query = self.selection.select("version");
        query.execute(&self.session).await
    }
}
impl super::Node for Naming {
    fn id(
        &self,
    ) -> impl core::future::Future<Output = Result<crate::Id, crate::QueryError>> + Send {
        let query = self.selection.select("id");
        let session = self.session.clone();
        async move { query.execute(&session).await }
    }
}
