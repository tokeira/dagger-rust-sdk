//! Generated bindings owned by the GraphQL `CacheSharingMode` type.
// @generated {"format":"dagger-rust-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:d606e02207a4265f6c76adc93f73153a694ceaf51c12d36b2a3b8baf131ff252","target_revision":"f0a3d728cf7012e0954f772b5dc5c8471648afaa"}
#[doc = "Sharing mode of the cache volume."]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, serde :: Deserialize, serde :: Serialize)]
pub enum CacheSharingMode {
    #[doc = "Shares the cache volume amongst many build pipelines, but will serialize the writes"]
    #[serde(rename = "LOCKED")]
    Locked,
    #[doc = "Keeps a cache volume for a single build pipeline"]
    #[serde(rename = "PRIVATE")]
    Private,
    #[doc = "Shares the cache volume amongst many build pipelines"]
    #[serde(rename = "SHARED")]
    Shared,
}
impl crate::__private::ModuleInputCodec for CacheSharingMode {
    fn decode_input(
        value: &crate::__private::ModuleJson,
        _context: &crate::__private::ModuleContextBase,
    ) -> Result<Self, crate::__private::DecodeError> {
        crate::__private::decode_string_value(value)
    }
}
impl crate::__private::ModuleOutputCodec for CacheSharingMode {
    fn encode_output<'a>(
        self,
        _context: &'a crate::__private::ModuleContextBase,
    ) -> crate::__private::ModuleBoxFuture<
        'a,
        Result<crate::__private::ModuleJson, crate::__private::EncodeError>,
    >
    where
        Self: 'a,
    {
        crate::__private::encode_string_value(self)
    }
}
