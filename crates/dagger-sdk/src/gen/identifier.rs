//! Generated bindings owned by the GraphQL `Identifier` type.
// @generated {"format":"dagger-rust-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:d606e02207a4265f6c76adc93f73153a694ceaf51c12d36b2a3b8baf131ff252","target_revision":"f0a3d728cf7012e0954f772b5dc5c8471648afaa"}
#[doc = "A name parsed into words, which can be formatted in any casing."]
#[derive(Clone)]
pub struct Identifier {
    pub(crate) session: crate::lifecycle::SessionHandle,
    pub(crate) selection: crate::query::Selection,
}
#[doc = "Owned optional arguments for GraphQL operation `Identifier.format`; reuse does not mutate caller state."]
#[derive(Clone, Debug, Default)]
#[non_exhaustive]
pub struct IdentifierFormatOpts {
    #[doc = "How to write acronyms and terms where a word starts with a capital.\n\n`None` omits GraphQL field `acronyms` and preserves engine default `Enum(SchemaName(\"UPPERCASE\"))`."]
    pub acronyms: Option<super::AcronymStyle>,
}
impl IdentifierFormatOpts {
    #[doc = "Sets GraphQL argument `acronyms` to a concrete value instead of omitting it."]
    #[must_use]
    pub fn with_acronyms(mut self, value: super::AcronymStyle) -> Self {
        self.acronyms = Some(value);
        self
    }
}
impl crate::IntoID<crate::Id> for Identifier {
    fn into_id(
        self,
    ) -> core::pin::Pin<
        Box<dyn core::future::Future<Output = Result<crate::Id, crate::QueryError>> + Send>,
    > {
        Box::pin(async move { self.id().await })
    }
}
impl crate::loadable::private::Sealed for Identifier {
    fn graphql_type() -> &'static str {
        "Identifier"
    }
    fn from_query(
        session: crate::lifecycle::SessionHandle,
        selection: crate::query::Selection,
    ) -> Self {
        Self { session, selection }
    }
}
impl From<Identifier> for crate::IdInput<Identifier> {
    fn from(value: Identifier) -> Self {
        crate::IdInput::lazy(value)
    }
}
impl From<Identifier> for crate::IdInput<super::NodeClient> {
    fn from(value: Identifier) -> Self {
        crate::IdInput::lazy(value)
    }
}
impl Identifier {
    #[doc = "Format the identifier in a casing.\n\nSelects GraphQL field `format` on `Identifier`."]
    pub async fn format(&self, casing: super::Casing) -> Result<String, crate::QueryError> {
        let query = self.selection.select("format");
        let query = query.arg("casing", casing);
        query.execute(&self.session).await
    }
    #[doc = "Executes GraphQL operation `format` with a borrowed, reusable `IdentifierFormatOpts` value."]
    pub async fn format_opts(
        &self,
        casing: super::Casing,
        opts: &IdentifierFormatOpts,
    ) -> Result<String, crate::QueryError> {
        let query = self.selection.select("format");
        let query = if let Some(value) = &opts.acronyms {
            query.arg("acronyms", value)
        } else {
            query
        };
        let query = query.arg("casing", casing);
        query.execute(&self.session).await
    }
    #[doc = "A unique identifier for this Identifier.\n\nSelects GraphQL field `id` on `Identifier`."]
    pub async fn id(&self) -> Result<crate::Id, crate::QueryError> {
        let query = self.selection.select("id");
        query.execute(&self.session).await
    }
    #[doc = "The name as given.\n\nSelects GraphQL field `name` on `Identifier`."]
    pub async fn name(&self) -> Result<String, crate::QueryError> {
        let query = self.selection.select("name");
        query.execute(&self.session).await
    }
    #[doc = "The words that make up the name, in order.\n\nSelects GraphQL field `words` on `Identifier`."]
    pub async fn words(&self) -> Result<Vec<super::IdentifierWord>, crate::QueryError> {
        let query = self.selection.select("words");
        let query = query.select("id");
        query
            .execute_reentry::<super::IdentifierWord, Vec<crate::Id>>(
                &self.session,
                "IdentifierWord",
            )
            .await
    }
}
impl super::Node for Identifier {
    fn id(
        &self,
    ) -> impl core::future::Future<Output = Result<crate::Id, crate::QueryError>> + Send {
        let query = self.selection.select("id");
        let session = self.session.clone();
        async move { query.execute(&session).await }
    }
}
