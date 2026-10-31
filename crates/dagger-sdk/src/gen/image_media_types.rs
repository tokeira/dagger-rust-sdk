//! Generated bindings owned by the GraphQL `ImageMediaTypes` type.
// @generated {"format":"dagger-rust-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:d606e02207a4265f6c76adc93f73153a694ceaf51c12d36b2a3b8baf131ff252","target_revision":"f0a3d728cf7012e0954f772b5dc5c8471648afaa"}
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
