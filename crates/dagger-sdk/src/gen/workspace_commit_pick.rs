//! Generated bindings owned by the GraphQL `WorkspaceCommitPick` type.
// @generated {"format":"dagger-rust-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:fa63be22540138c2888c5779db388e117bf8b168d9f7f59b2cb2a77dc4cc9f4d","target_revision":"a7a2e64e853be92728f05aad9dc07bebe63c7448"}
#[doc = "A source commit classified against the receiving workspace."]
#[derive(Clone)]
pub struct WorkspaceCommitPick {
    pub(crate) session: crate::lifecycle::SessionHandle,
    pub(crate) selection: crate::query::Selection,
}
impl crate::IntoID<crate::Id> for WorkspaceCommitPick {
    fn into_id(
        self,
    ) -> core::pin::Pin<
        Box<dyn core::future::Future<Output = Result<crate::Id, crate::QueryError>> + Send>,
    > {
        Box::pin(async move { self.id().await })
    }
}
impl crate::loadable::private::Sealed for WorkspaceCommitPick {
    fn graphql_type() -> &'static str {
        "WorkspaceCommitPick"
    }
    fn from_query(
        session: crate::lifecycle::SessionHandle,
        selection: crate::query::Selection,
    ) -> Self {
        Self { session, selection }
    }
}
impl From<WorkspaceCommitPick> for crate::IdInput<WorkspaceCommitPick> {
    fn from(value: WorkspaceCommitPick) -> Self {
        crate::IdInput::lazy(value)
    }
}
impl From<WorkspaceCommitPick> for crate::IdInput<super::NodeClient> {
    fn from(value: WorkspaceCommitPick) -> Self {
        crate::IdInput::lazy(value)
    }
}
impl WorkspaceCommitPick {
    #[doc = "The commit in the source workspace.\n\nSelects GraphQL field `commit` on `WorkspaceCommitPick`."]
    #[must_use]
    pub fn commit(&self) -> super::GitCommit {
        let query = self.selection.select("commit");
        super::GitCommit {
            session: self.session.clone(),
            selection: query,
        }
    }
    #[doc = "Workspace-root-relative conflicting paths. Empty unless the status is CONFLICT.\n\nSelects GraphQL field `conflictPaths` on `WorkspaceCommitPick`."]
    pub async fn conflict_paths(&self) -> Result<Vec<String>, crate::QueryError> {
        let query = self.selection.select("conflictPaths");
        query.execute(&self.session).await
    }
    #[doc = "A unique identifier for this WorkspaceCommitPick.\n\nSelects GraphQL field `id` on `WorkspaceCommitPick`."]
    pub async fn id(&self) -> Result<crate::Id, crate::QueryError> {
        let query = self.selection.select("id");
        query.execute(&self.session).await
    }
    #[doc = "Why the commit conflicts, or NONE.\n\nSelects GraphQL field `reason` on `WorkspaceCommitPick`."]
    pub async fn reason(&self) -> Result<super::WorkspaceCommitPickReason, crate::QueryError> {
        let query = self.selection.select("reason");
        query.execute(&self.session).await
    }
    #[doc = "Whether this commit can be applied.\n\nSelects GraphQL field `status` on `WorkspaceCommitPick`."]
    pub async fn status(&self) -> Result<super::WorkspaceCommitPickStatus, crate::QueryError> {
        let query = self.selection.select("status");
        query.execute(&self.session).await
    }
}
impl super::Node for WorkspaceCommitPick {
    fn id(
        &self,
    ) -> impl core::future::Future<Output = Result<crate::Id, crate::QueryError>> + Send {
        let query = self.selection.select("id");
        let session = self.session.clone();
        async move { query.execute(&session).await }
    }
}
