//! Generated bindings owned by the GraphQL `Casing` type.
// @generated {"format":"dagger-rust-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:d606e02207a4265f6c76adc93f73153a694ceaf51c12d36b2a3b8baf131ff252","target_revision":"f0a3d728cf7012e0954f772b5dc5c8471648afaa"}
#[doc = "A convention for joining words into an identifier."]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, serde :: Deserialize, serde :: Serialize)]
pub enum Casing {
    #[doc = "httpClient"]
    #[serde(rename = "CAMEL")]
    Camel,
    #[doc = "httpclient (output only: drops word boundaries)"]
    #[serde(rename = "FLAT")]
    Flat,
    #[doc = "http-client"]
    #[serde(rename = "KEBAB")]
    Kebab,
    #[doc = "HTTPClient"]
    #[serde(rename = "PASCAL")]
    Pascal,
    #[doc = "HTTP_CLIENT"]
    #[serde(rename = "SCREAMING_SNAKE")]
    ScreamingSnake,
    #[doc = "http_client"]
    #[serde(rename = "SNAKE")]
    Snake,
}
impl crate::__private::ModuleInputCodec for Casing {
    fn decode_input(
        value: &crate::__private::ModuleJson,
        _context: &crate::__private::ModuleContextBase,
    ) -> Result<Self, crate::__private::DecodeError> {
        crate::__private::decode_string_value(value)
    }
}
impl crate::__private::ModuleOutputCodec for Casing {
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
