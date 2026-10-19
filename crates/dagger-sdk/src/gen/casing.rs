//! Generated bindings owned by the GraphQL `Casing` type.
// @generated {"format":"dagger-rust-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:df7839fc7381ed3e899c3e1262a3b7cf384f06c4ac6a72334be3bc543bf1e9be","target_revision":"80699ac67a1c72dc38d30c4186105cd9dd3afde6"}
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
