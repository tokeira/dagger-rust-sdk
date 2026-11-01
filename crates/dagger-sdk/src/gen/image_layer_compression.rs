//! Generated bindings owned by the GraphQL `ImageLayerCompression` type.
// @generated {"format":"dagger-rust-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:d606e02207a4265f6c76adc93f73153a694ceaf51c12d36b2a3b8baf131ff252","target_revision":"f0a3d728cf7012e0954f772b5dc5c8471648afaa"}
#[doc = "Compression algorithm to use for image layers."]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, serde :: Deserialize, serde :: Serialize)]
pub enum ImageLayerCompression {
    #[doc = "GraphQL enum value `EStarGZ`."]
    #[serde(rename = "EStarGZ", alias = "ESTARGZ")]
    EStarGz,
    #[doc = "GraphQL enum value `Gzip`."]
    #[serde(rename = "Gzip", alias = "GZIP")]
    Gzip,
    #[doc = "GraphQL enum value `Uncompressed`."]
    #[serde(rename = "Uncompressed", alias = "UNCOMPRESSED")]
    Uncompressed,
    #[doc = "GraphQL enum value `Zstd`."]
    #[serde(rename = "Zstd", alias = "ZSTD")]
    Zstd,
}
impl crate::__private::ModuleInputCodec for ImageLayerCompression {
    fn decode_input(
        value: &crate::__private::ModuleJson,
        _context: &crate::__private::ModuleContextBase,
    ) -> Result<Self, crate::__private::DecodeError> {
        crate::__private::decode_string_value(value)
    }
}
impl crate::__private::ModuleOutputCodec for ImageLayerCompression {
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
