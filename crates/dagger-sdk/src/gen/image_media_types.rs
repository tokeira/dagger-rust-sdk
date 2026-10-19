//! Generated bindings owned by the GraphQL `ImageMediaTypes` type.
// @generated {"format":"dagger-rust-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:df7839fc7381ed3e899c3e1262a3b7cf384f06c4ac6a72334be3bc543bf1e9be","target_revision":"80699ac67a1c72dc38d30c4186105cd9dd3afde6"}
#[doc = "Mediatypes to use in published or exported image metadata."]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, serde :: Deserialize, serde :: Serialize)]
pub enum ImageMediaTypes {
    #[doc = "GraphQL enum value `DockerMediaTypes`."]
    #[serde(rename = "DockerMediaTypes", alias = "DOCKER")]
    DockerMediaTypes,
    #[doc = "GraphQL enum value `OCIMediaTypes`."]
    #[serde(rename = "OCIMediaTypes", alias = "OCI")]
    OciMediaTypes,
}
