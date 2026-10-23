//! Generated bindings owned by the GraphQL `ImageMediaTypes` type.
// @generated {"format":"dagger-rust-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:fa63be22540138c2888c5779db388e117bf8b168d9f7f59b2cb2a77dc4cc9f4d","target_revision":"a7a2e64e853be92728f05aad9dc07bebe63c7448"}
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
