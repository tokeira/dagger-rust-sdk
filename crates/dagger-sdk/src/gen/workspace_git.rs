//! Generated bindings owned by the GraphQL `WorkspaceGit` type.
// @generated {"format":"dagger-rust-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:fa63be22540138c2888c5779db388e117bf8b168d9f7f59b2cb2a77dc4cc9f4d","target_revision":"a7a2e64e853be92728f05aad9dc07bebe63c7448"}
#[doc = "Local git state for a workspace."]
#[derive(Clone)]
pub struct WorkspaceGit {
    pub(crate) session: crate::lifecycle::SessionHandle,
    pub(crate) selection: crate::query::Selection,
}
impl crate::IntoID<crate::Id> for WorkspaceGit {
    fn into_id(
        self,
    ) -> core::pin::Pin<
        Box<dyn core::future::Future<Output = Result<crate::Id, crate::QueryError>> + Send>,
    > {
        Box::pin(async move { self.id().await })
    }
}
impl crate::loadable::private::Sealed for WorkspaceGit {
    fn graphql_type() -> &'static str {
        "WorkspaceGit"
    }
    fn from_query(
        session: crate::lifecycle::SessionHandle,
        selection: crate::query::Selection,
    ) -> Self {
        Self { session, selection }
    }
}
impl From<WorkspaceGit> for crate::IdInput<WorkspaceGit> {
    fn from(value: WorkspaceGit) -> Self {
        crate::IdInput::lazy(value)
    }
}
impl From<WorkspaceGit> for crate::IdInput<super::NodeClient> {
    fn from(value: WorkspaceGit) -> Self {
        crate::IdInput::lazy(value)
    }
}
impl WorkspaceGit {
    #[doc = "Return a self-contained Git metadata directory for this workspace's HEAD, including its full reachable history and an index matching HEAD.\n\nMount this directory at .git alongside workspace.directory(\"/\") to create a usable checkout. Pending workspace edits remain uncommitted; the original checkout's staging state is not preserved.\n\nThis is a snapshot: Git writes to a mounted copy do not update the workspace. The workspace must have a Git repository with a HEAD commit.\n\nSelects GraphQL field `directory` on `WorkspaceGit`."]
    #[must_use]
    pub fn directory(&self) -> super::Directory {
        let query = self.selection.select("directory");
        super::Directory {
            session: self.session.clone(),
            selection: query,
        }
    }
    #[doc = "The checked-out HEAD of this workspace.\n\nSelects GraphQL field `head` on `WorkspaceGit`."]
    #[must_use]
    pub fn head(&self) -> super::GitRef {
        let query = self.selection.select("head");
        super::GitRef {
            session: self.session.clone(),
            selection: query,
        }
    }
    #[doc = "A unique identifier for this WorkspaceGit.\n\nSelects GraphQL field `id` on `WorkspaceGit`."]
    pub async fn id(&self) -> Result<crate::Id, crate::QueryError> {
        let query = self.selection.select("id");
        query.execute(&self.session).await
    }
    #[doc = "Uncommitted changes in this workspace, using the same rules as GitRepository.uncommitted.\n\nSelects GraphQL field `uncommitted` on `WorkspaceGit`."]
    #[must_use]
    pub fn uncommitted(&self) -> super::Changeset {
        let query = self.selection.select("uncommitted");
        super::Changeset {
            session: self.session.clone(),
            selection: query,
        }
    }
}
impl super::Node for WorkspaceGit {
    fn id(
        &self,
    ) -> impl core::future::Future<Output = Result<crate::Id, crate::QueryError>> + Send {
        let query = self.selection.select("id");
        let session = self.session.clone();
        async move { query.execute(&session).await }
    }
}
