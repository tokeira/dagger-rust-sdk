//! Generated bindings owned by the GraphQL `WorkspaceSDK` type.
// @generated {"format":"dagger-rust-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:9511bc82fb704520cdc632da73463fc7396b1bb081e0446b61665c30764a5e7c","target_revision":"400b82d6f3c09ef096d2f59b3e920267b5969bf4"}
#[doc = "An installed SDK: a module marked for scaffolding other modules and clients."]
#[derive(Clone)]
pub struct WorkspaceSdk {
    pub(crate) session: crate::lifecycle::SessionHandle,
    pub(crate) selection: crate::query::Selection,
}
impl crate::IntoID<crate::Id> for WorkspaceSdk {
    fn into_id(
        self,
    ) -> core::pin::Pin<
        Box<dyn core::future::Future<Output = Result<crate::Id, crate::QueryError>> + Send>,
    > {
        Box::pin(async move { self.id().await })
    }
}
impl crate::loadable::private::Sealed for WorkspaceSdk {
    fn graphql_type() -> &'static str {
        "WorkspaceSDK"
    }
    fn from_query(
        session: crate::lifecycle::SessionHandle,
        selection: crate::query::Selection,
    ) -> Self {
        Self { session, selection }
    }
}
impl From<WorkspaceSdk> for crate::IdInput<WorkspaceSdk> {
    fn from(value: WorkspaceSdk) -> Self {
        crate::IdInput::lazy(value)
    }
}
impl From<WorkspaceSdk> for crate::IdInput<super::NodeClient> {
    fn from(value: WorkspaceSdk) -> Self {
        crate::IdInput::lazy(value)
    }
}
impl WorkspaceSdk {
    #[doc = "Clients generated with this SDK.\n\nSelects GraphQL field `clients` on `WorkspaceSDK`."]
    pub async fn clients(&self) -> Result<Vec<super::WorkspaceModule>, crate::QueryError> {
        let query = self.selection.select("clients");
        let query = query.select("id");
        query
            .execute_reentry::<super::WorkspaceModule, Vec<crate::Id>>(
                &self.session,
                "WorkspaceModule",
            )
            .await
    }
    #[doc = "Generate the modules and clients managed by this SDK.\n\nSelects GraphQL field `generate` on `WorkspaceSDK`."]
    #[must_use]
    pub fn generate(&self) -> super::Changeset {
        let query = self.selection.select("generate");
        super::Changeset {
            session: self.session.clone(),
            selection: query,
        }
    }
    #[doc = "A unique identifier for this WorkspaceSDK.\n\nSelects GraphQL field `id` on `WorkspaceSDK`."]
    pub async fn id(&self) -> Result<crate::Id, crate::QueryError> {
        let query = self.selection.select("id");
        query.execute(&self.session).await
    }
    #[doc = "Modules authored with this SDK.\n\nSelects GraphQL field `modules` on `WorkspaceSDK`."]
    pub async fn modules(&self) -> Result<Vec<super::WorkspaceModule>, crate::QueryError> {
        let query = self.selection.select("modules");
        let query = query.select("id");
        query
            .execute_reentry::<super::WorkspaceModule, Vec<crate::Id>>(
                &self.session,
                "WorkspaceModule",
            )
            .await
    }
    #[doc = "The user-facing SDK name.\n\nSelects GraphQL field `name` on `WorkspaceSDK`."]
    pub async fn name(&self) -> Result<String, crate::QueryError> {
        let query = self.selection.select("name");
        query.execute(&self.session).await
    }
    #[doc = "The module reference this SDK was installed from.\n\nSelects GraphQL field `ref` on `WorkspaceSDK`."]
    pub async fn r#ref(&self) -> Result<String, crate::QueryError> {
        let query = self.selection.select("ref");
        query.execute(&self.session).await
    }
    #[doc = "The value a setting records in this SDK's scope settings, as a native JSON value.\n\nReads \\[sdks.&lt;name&gt;.scopes.\"&lt;scope&gt;\".settings\\] from the workspace config, in the scope whose key resolves to path. The setting key is matched without regard to case.\n\nNull when no scope resolves to path, or the scope records no such setting. Fails when the config is invalid, path escapes the workspace root, key matches several recorded settings, or the value cannot be represented as JSON.\n\nFails when this SDK is no longer installed in the workspace config, even if Workspace.sdk returned it earlier.\n\nSelects GraphQL field `scopeSetting` on `WorkspaceSDK`."]
    pub async fn scope_setting(
        &self,
        key: impl Into<String>,
        path: impl Into<String>,
    ) -> Result<Option<super::JsonValue>, crate::QueryError> {
        let query = self.selection.select("scopeSetting");
        let query = query.arg("key", key.into());
        let query = query.arg("path", path.into());
        let query = query.select("id");
        query
            .execute_reentry::<super::JsonValue, Option<crate::Id>>(&self.session, "JSONValue")
            .await
    }
}
impl super::Node for WorkspaceSdk {
    fn id(
        &self,
    ) -> impl core::future::Future<Output = Result<crate::Id, crate::QueryError>> + Send {
        let query = self.selection.select("id");
        let session = self.session.clone();
        async move { query.execute(&session).await }
    }
}
