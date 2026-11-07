//! Generated bindings owned by the GraphQL `ReturnType` type.
// @generated {"format":"dagger-rust-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:4dfe7942ff12bd22554ccd2cb26a5427e3a9eeece22dbba9d8aa89e4590e97ff","target_revision":"a45241b20a5e07fbfc5954479ef0b61da843499c"}
#[doc = "Expected return type of an execution"]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, serde :: Deserialize, serde :: Serialize)]
pub enum ReturnType {
    #[doc = "Any execution (exit codes 0-127 and 192-255)"]
    #[serde(rename = "ANY")]
    Any,
    #[doc = "A failed execution (exit codes 1-127 and 192-255)"]
    #[serde(rename = "FAILURE")]
    Failure,
    #[doc = "A successful execution (exit code 0)"]
    #[serde(rename = "SUCCESS")]
    Success,
}
impl crate::__private::ModuleInputCodec for ReturnType {
    fn decode_input(
        value: &crate::__private::ModuleJson,
        _context: &crate::__private::ModuleContextBase,
    ) -> Result<Self, crate::__private::DecodeError> {
        crate::__private::decode_string_value(value)
    }
}
impl crate::__private::ModuleOutputCodec for ReturnType {
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
