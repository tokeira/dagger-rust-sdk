//! Generated bindings owned by the GraphQL `Casing` type.
// @generated {"format":"dagger-rust-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:fa63be22540138c2888c5779db388e117bf8b168d9f7f59b2cb2a77dc4cc9f4d","target_revision":"a7a2e64e853be92728f05aad9dc07bebe63c7448"}
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
