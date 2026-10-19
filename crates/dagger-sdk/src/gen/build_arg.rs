//! Generated bindings owned by the GraphQL `BuildArg` type.
// @generated {"format":"dagger-rust-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:df7839fc7381ed3e899c3e1262a3b7cf384f06c4ac6a72334be3bc543bf1e9be","target_revision":"80699ac67a1c72dc38d30c4186105cd9dd3afde6"}
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
