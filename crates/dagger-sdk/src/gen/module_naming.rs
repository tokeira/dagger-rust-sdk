//! Generated bindings owned by the GraphQL `ModuleNaming` type.
// @generated {"format":"dagger-rust-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:9511bc82fb704520cdc632da73463fc7396b1bb081e0446b61665c30764a5e7c","target_revision":"400b82d6f3c09ef096d2f59b3e920267b5969bf4"}
#[doc = "The names the engine gives a module's own declarations, by the naming rules of the version the module is named by.\n\nEvery list-taking field answers in input order."]
#[derive(Clone)]
pub struct ModuleNaming {
    pub(crate) session: crate::lifecycle::SessionHandle,
    pub(crate) selection: crate::query::Selection,
}
impl crate::IntoID<crate::Id> for ModuleNaming {
    fn into_id(
        self,
    ) -> core::pin::Pin<
        Box<dyn core::future::Future<Output = Result<crate::Id, crate::QueryError>> + Send>,
    > {
        Box::pin(async move { self.id().await })
    }
}
impl crate::loadable::private::Sealed for ModuleNaming {
    fn graphql_type() -> &'static str {
        "ModuleNaming"
    }
    fn from_query(
        session: crate::lifecycle::SessionHandle,
        selection: crate::query::Selection,
    ) -> Self {
        Self { session, selection }
    }
}
impl From<ModuleNaming> for crate::IdInput<ModuleNaming> {
    fn from(value: ModuleNaming) -> Self {
        crate::IdInput::lazy(value)
    }
}
impl From<ModuleNaming> for crate::IdInput<super::NodeClient> {
    fn from(value: ModuleNaming) -> Self {
        crate::IdInput::lazy(value)
    }
}
impl ModuleNaming {
    #[doc = "The schema names of enum members this module declares, in input order: SCREAMING_SNAKE_CASE.\n\nA name already written in GraphQL's enum member convention (HTTP2, P_256) is kept as it is. With the engine's identifier naming rules, Arm64 is ARM64 and GitHub is GITHUB; with the legacy strcase rules, Arm64 is ARM_64.\n\nErrors on a name the module's naming rules can't parse (non-ASCII, no letters or digits, or over 256 bytes). The legacy rules name anything.\n\nSelects GraphQL field `enumMemberNames` on `ModuleNaming`."]
    pub async fn enum_member_names(
        &self,
        names: Vec<impl Into<String>>,
    ) -> Result<Vec<String>, crate::QueryError> {
        let query = self.selection.select("enumMemberNames");
        let names = names.into_iter().map(Into::into).collect::<Vec<String>>();
        let query = query.arg("names", names);
        query.execute(&self.session).await
    }
    #[doc = "The schema names of functions, object fields and function arguments this module declares, including interface functions and their arguments, in input order.\n\nWith the engine's identifier naming rules, acronyms are written like words: getURL is getUrl, baseURL is baseUrl, HTTPClient is httpClient. With the legacy strcase rules, HTTPClient is httpclient and e2eTest is e2ETest.\n\nErrors on a name the module's naming rules can't parse (non-ASCII, no letters or digits, or over 256 bytes). The legacy rules name anything.\n\nSelects GraphQL field `fieldNames` on `ModuleNaming`."]
    pub async fn field_names(
        &self,
        names: Vec<impl Into<String>>,
    ) -> Result<Vec<String>, crate::QueryError> {
        let query = self.selection.select("fieldNames");
        let names = names.into_iter().map(Into::into).collect::<Vec<String>>();
        let query = query.arg("names", names);
        query.execute(&self.session).await
    }
    #[doc = "A unique identifier for this ModuleNaming.\n\nSelects GraphQL field `id` on `ModuleNaming`."]
    pub async fn id(&self) -> Result<crate::Id, crate::QueryError> {
        let query = self.selection.select("id");
        query.execute(&self.session).await
    }
    #[doc = "Which of the candidates, the names of objects this module declares, the engine takes as the module's main object; null if none.\n\nThe engine finds the main object by the module's name, in any casing: the first candidate, in input order, whose name normalizes to the same type name as the module's, or is spelled as that type name already (K8STools for module k8s-tools, by either rules: strcase spells k8s-tools K8STools, though it would normalize K8STools again to K8Stools). Failing that, with the engine's identifier naming rules only, the only candidate spelled the same once lowercased without word boundaries (Sha256Sum for module sha256sum); several such candidates are ambiguous, and give null.\n\nErrors on a name the module's naming rules can't parse (non-ASCII, no letters or digits, or over 256 bytes). The legacy rules name anything.\n\nSelects GraphQL field `mainObject` on `ModuleNaming`."]
    pub async fn main_object(
        &self,
        candidates: Vec<impl Into<String>>,
    ) -> Result<Option<String>, crate::QueryError> {
        let query = self.selection.select("mainObject");
        let candidates = candidates
            .into_iter()
            .map(Into::into)
            .collect::<Vec<String>>();
        let query = query.arg("candidates", candidates);
        query.execute(&self.session).await
    }
    #[doc = "The module's name, which namespaces its types.\n\nSelects GraphQL field `name` on `ModuleNaming`."]
    pub async fn name(&self) -> Result<String, crate::QueryError> {
        let query = self.selection.select("name");
        query.execute(&self.session).await
    }
    #[doc = "The module's original name, which its main object and namespaced type names are spelled after. It differs from name when the module is loaded under another name, as a dependency alias.\n\nSelects GraphQL field `originalName` on `ModuleNaming`."]
    pub async fn original_name(&self) -> Result<String, crate::QueryError> {
        let query = self.selection.select("originalName");
        query.execute(&self.session).await
    }
    #[doc = "The schema names of objects, interfaces and enums this module declares, as the engine installs them: namespaced under the module's name, in input order.\n\nA name is prefixed with the module's name unless it already starts with the module's name on a word boundary. With the engine's identifier naming rules: HttpClient in module my-mod is MyModHTTPClient; Postman in module post is PostPostman, while PostMan stays PostMan; K8STools in module k8s-tools is K8sTools, the module's own name. With the legacy strcase rules, HttpClient in module my-mod is MyModHttpClient.\n\nErrors on a name the module's naming rules can't parse (non-ASCII, no letters or digits, or over 256 bytes). The legacy rules name anything.\n\nSelects GraphQL field `typeNames` on `ModuleNaming`."]
    pub async fn type_names(
        &self,
        names: Vec<impl Into<String>>,
    ) -> Result<Vec<String>, crate::QueryError> {
        let query = self.selection.select("typeNames");
        let names = names.into_iter().map(Into::into).collect::<Vec<String>>();
        let query = query.arg("names", names);
        query.execute(&self.session).await
    }
    #[doc = "The normalized engine version whose naming rules name the module.\n\nSelects GraphQL field `version` on `ModuleNaming`."]
    pub async fn version(&self) -> Result<String, crate::QueryError> {
        let query = self.selection.select("version");
        query.execute(&self.session).await
    }
}
impl super::Node for ModuleNaming {
    fn id(
        &self,
    ) -> impl core::future::Future<Output = Result<crate::Id, crate::QueryError>> + Send {
        let query = self.selection.select("id");
        let session = self.session.clone();
        async move { query.execute(&session).await }
    }
}
