//! Generated bindings owned by the GraphQL `NamingTerm` type.
// @generated {"format":"dagger-rust-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:fa63be22540138c2888c5779db388e117bf8b168d9f7f59b2cb2a77dc4cc9f4d","target_revision":"a7a2e64e853be92728f05aad9dc07bebe63c7448"}
#[doc = "An entry in the naming dictionary."]
#[derive(Clone)]
pub struct NamingTerm {
    pub(crate) session: crate::lifecycle::SessionHandle,
    pub(crate) selection: crate::query::Selection,
}
impl crate::IntoID<crate::Id> for NamingTerm {
    fn into_id(
        self,
    ) -> core::pin::Pin<
        Box<dyn core::future::Future<Output = Result<crate::Id, crate::QueryError>> + Send>,
    > {
        Box::pin(async move { self.id().await })
    }
}
impl crate::loadable::private::Sealed for NamingTerm {
    fn graphql_type() -> &'static str {
        "NamingTerm"
    }
    fn from_query(
        session: crate::lifecycle::SessionHandle,
        selection: crate::query::Selection,
    ) -> Self {
        Self { session, selection }
    }
}
impl From<NamingTerm> for crate::IdInput<NamingTerm> {
    fn from(value: NamingTerm) -> Self {
        crate::IdInput::lazy(value)
    }
}
impl From<NamingTerm> for crate::IdInput<super::NodeClient> {
    fn from(value: NamingTerm) -> Self {
        crate::IdInput::lazy(value)
    }
}
impl NamingTerm {
    #[doc = "Spelling in the CAPITALIZED style: \"Http\", \"Ipv6\", \"GitHub\", \"Ios\".\n\nSelects GraphQL field `capitalized` on `NamingTerm`."]
    pub async fn capitalized(&self) -> Result<String, crate::QueryError> {
        let query = self.selection.select("capitalized");
        query.execute(&self.session).await
    }
    #[doc = "A unique identifier for this NamingTerm.\n\nSelects GraphQL field `id` on `NamingTerm`."]
    pub async fn id(&self) -> Result<crate::Id, crate::QueryError> {
        let query = self.selection.select("id");
        query.execute(&self.session).await
    }
    #[doc = "Standard spelling: \"HTTP\", \"IPv6\", \"GitHub\", \"iOS\".\n\nSelects GraphQL field `spelling` on `NamingTerm`."]
    pub async fn spelling(&self) -> Result<String, crate::QueryError> {
        let query = self.selection.select("spelling");
        query.execute(&self.session).await
    }
}
impl super::Node for NamingTerm {
    fn id(
        &self,
    ) -> impl core::future::Future<Output = Result<crate::Id, crate::QueryError>> + Send {
        let query = self.selection.select("id");
        let session = self.session.clone();
        async move { query.execute(&session).await }
    }
}
