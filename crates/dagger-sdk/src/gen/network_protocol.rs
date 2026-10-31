//! Generated bindings owned by the GraphQL `NetworkProtocol` type.
// @generated {"format":"dagger-rust-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:d606e02207a4265f6c76adc93f73153a694ceaf51c12d36b2a3b8baf131ff252","target_revision":"f0a3d728cf7012e0954f772b5dc5c8471648afaa"}
#[doc = "Transport layer network protocol associated to a port."]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, serde :: Deserialize, serde :: Serialize)]
pub enum NetworkProtocol {
    #[doc = "GraphQL enum value `TCP`."]
    #[serde(rename = "TCP")]
    Tcp,
    #[doc = "GraphQL enum value `UDP`."]
    #[serde(rename = "UDP")]
    Udp,
}
