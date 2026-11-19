//! Bindings owned by visible GraphQL type `GreeterMode`.
// @generated {"format":"dagger-rust-visible-v1","ownership":"dagger-codegen","schema_digest":"sha256:9225b2a362f6f3f1db7f20f8a02598f3e741c199f3ded63561adc4b5a6f77687","target_revision":"400b82d6f3c09ef096d2f59b3e920267b5969bf4"}
#[cfg_attr(not(doctest), doc = "Client fixture enum GreeterMode.")]
#[derive(
    Clone,
    Copy,
    Debug,
    Eq,
    Hash,
    PartialEq,
    dagger_sdk :: __private :: serde :: Deserialize,
    dagger_sdk :: __private :: serde :: Serialize,
)]
#[serde(crate = "dagger_sdk::__private::serde")]
pub enum GreeterMode {
    #[cfg_attr(not(doctest), doc = "Client fixture value LOUD.")]
    #[serde(rename = "LOUD")]
    Loud,
    #[cfg_attr(not(doctest), doc = "Client fixture value QUIET.")]
    #[serde(rename = "QUIET")]
    Quiet,
}
impl dagger_sdk::__private::ModuleInputCodec for GreeterMode {
    fn decode_input(
        value: &dagger_sdk::__private::ModuleJson,
        _context: &dagger_sdk::__private::ModuleContextBase,
    ) -> ::core::result::Result<Self, dagger_sdk::__private::DecodeError> {
        dagger_sdk::__private::decode_string_value(value)
    }
}
impl dagger_sdk::__private::ModuleOutputCodec for GreeterMode {
    fn encode_output<'a>(
        self,
        _context: &'a dagger_sdk::__private::ModuleContextBase,
    ) -> dagger_sdk::__private::ModuleBoxFuture<
        'a,
        ::core::result::Result<
            dagger_sdk::__private::ModuleJson,
            dagger_sdk::__private::EncodeError,
        >,
    >
    where
        Self: 'a,
    {
        dagger_sdk::__private::encode_string_value(self)
    }
}
