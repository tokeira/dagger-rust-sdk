//! Generated bindings owned by the GraphQL `BuildArg` type.
// @generated {"format":"dagger-rust-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:9511bc82fb704520cdc632da73463fc7396b1bb081e0446b61665c30764a5e7c","target_revision":"400b82d6f3c09ef096d2f59b3e920267b5969bf4"}
#[doc = "Key value object that represents a build argument."]
#[derive(Clone, Debug, PartialEq, serde :: Deserialize, serde :: Serialize)]
#[non_exhaustive]
pub struct BuildArg {
    #[doc = "The build argument name."]
    #[serde(rename = "name")]
    pub name: String,
    #[doc = "The build argument value."]
    #[serde(rename = "value")]
    pub value: String,
}
impl BuildArg {
    #[doc = "Creates `BuildArg` with every required GraphQL input field."]
    #[must_use]
    pub fn new(name: String, value: String) -> Self {
        Self { name, value }
    }
}
