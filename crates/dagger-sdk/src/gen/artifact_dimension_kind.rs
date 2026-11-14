//! Generated bindings owned by the GraphQL `ArtifactDimensionKind` type.
// @generated {"format":"dagger-rust-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:9511bc82fb704520cdc632da73463fc7396b1bb081e0446b61665c30764a5e7c","target_revision":"400b82d6f3c09ef096d2f59b3e920267b5969bf4"}
#[doc = "Generated enum for GraphQL `ArtifactDimensionKind`."]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, serde :: Deserialize, serde :: Serialize)]
pub enum ArtifactDimensionKind {
    #[doc = "GraphQL enum value `COLLECTION`."]
    #[serde(rename = "COLLECTION")]
    Collection,
    #[doc = "GraphQL enum value `MODULE`."]
    #[serde(rename = "MODULE")]
    Module,
    #[doc = "GraphQL enum value `TYPE`."]
    #[serde(rename = "TYPE")]
    Type,
}
impl crate::__private::ModuleInputCodec for ArtifactDimensionKind {
    fn decode_input(
        value: &crate::__private::ModuleJson,
        _context: &crate::__private::ModuleContextBase,
    ) -> Result<Self, crate::__private::DecodeError> {
        crate::__private::decode_string_value(value)
    }
}
impl crate::__private::ModuleOutputCodec for ArtifactDimensionKind {
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
