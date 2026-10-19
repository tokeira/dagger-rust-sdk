//! Generated bindings owned by the GraphQL `Generator` type.
// @generated {"format":"dagger-rust-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:df7839fc7381ed3e899c3e1262a3b7cf384f06c4ac6a72334be3bc543bf1e9be","target_revision":"80699ac67a1c72dc38d30c4186105cd9dd3afde6"}
#[doc = "A generation function and its staleness check. Reading changeset runs the function."]
#[derive(Clone)]
pub struct Generator {
    pub(crate) session: crate::lifecycle::SessionHandle,
    pub(crate) selection: crate::query::Selection,
}
impl crate::IntoID<crate::Id> for Generator {
    fn into_id(
        self,
    ) -> core::pin::Pin<
        Box<dyn core::future::Future<Output = Result<crate::Id, crate::QueryError>> + Send>,
    > {
        Box::pin(async move { self.id().await })
    }
}
impl crate::loadable::private::Sealed for Generator {
    fn graphql_type() -> &'static str {
        "Generator"
    }
    fn from_query(
        session: crate::lifecycle::SessionHandle,
        selection: crate::query::Selection,
    ) -> Self {
        Self { session, selection }
    }
}
impl From<Generator> for crate::IdInput<Generator> {
    fn from(value: Generator) -> Self {
        crate::IdInput::lazy(value)
    }
}
impl From<Generator> for crate::IdInput<super::NodeClient> {
    fn from(value: Generator) -> Self {
        crate::IdInput::lazy(value)
    }
}
impl Generator {
    #[doc = "Run the generator and return its changes.\n\nSelects GraphQL field `changeset` on `Generator`."]
    #[must_use]
    pub fn changeset(&self) -> super::Changeset {
        let query = self.selection.select("changeset");
        super::Changeset {
            session: self.session.clone(),
            selection: query,
        }
    }
    #[doc = "A unique identifier for this Generator.\n\nSelects GraphQL field `id` on `Generator`."]
    pub async fn id(&self) -> Result<crate::Id, crate::QueryError> {
        let query = self.selection.select("id");
        query.execute(&self.session).await
    }
    #[doc = "A check that passes when this generator would produce no changes.\n\nSelects GraphQL field `stale` on `Generator`."]
    #[must_use]
    pub fn stale(&self) -> super::Check {
        let query = self.selection.select("stale");
        super::Check {
            session: self.session.clone(),
            selection: query,
        }
    }
    #[doc = "Run the generator and retain its result.\n\nSelects GraphQL field `sync` on `Generator`."]
    #[must_use]
    pub fn sync(&self) -> super::Generator {
        let query = self.selection.select("sync");
        super::Generator {
            session: self.session.clone(),
            selection: query,
        }
    }
}
impl super::Node for Generator {
    fn id(
        &self,
    ) -> impl core::future::Future<Output = Result<crate::Id, crate::QueryError>> + Send {
        let query = self.selection.select("id");
        let session = self.session.clone();
        async move { query.execute(&session).await }
    }
}
