//! Generated bindings owned by the GraphQL `BuildArg` type.
// @generated {"format":"dagger-rust-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:4dfe7942ff12bd22554ccd2cb26a5427e3a9eeece22dbba9d8aa89e4590e97ff","target_revision":"a45241b20a5e07fbfc5954479ef0b61da843499c"}
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
