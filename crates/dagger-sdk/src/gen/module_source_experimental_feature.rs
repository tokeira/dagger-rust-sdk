//! Generated bindings owned by the GraphQL `ModuleSourceExperimentalFeature` type.
// @generated {"format":"dagger-rust-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:4dfe7942ff12bd22554ccd2cb26a5427e3a9eeece22dbba9d8aa89e4590e97ff","target_revision":"a45241b20a5e07fbfc5954479ef0b61da843499c"}
#[doc = "Experimental features of a module"]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, serde :: Deserialize, serde :: Serialize)]
pub enum ModuleSourceExperimentalFeature {
    #[doc = "Self calls"]
    #[serde(rename = "SELF_CALLS")]
    SelfCalls,
}
impl crate::__private::ModuleInputCodec for ModuleSourceExperimentalFeature {
    fn decode_input(
        value: &crate::__private::ModuleJson,
        _context: &crate::__private::ModuleContextBase,
    ) -> Result<Self, crate::__private::DecodeError> {
        crate::__private::decode_string_value(value)
    }
}
impl crate::__private::ModuleOutputCodec for ModuleSourceExperimentalFeature {
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
