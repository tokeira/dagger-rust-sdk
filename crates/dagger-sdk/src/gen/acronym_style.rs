//! Generated bindings owned by the GraphQL `AcronymStyle` type.
// @generated {"format":"dagger-rust-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:fa63be22540138c2888c5779db388e117bf8b168d9f7f59b2cb2a77dc4cc9f4d","target_revision":"a7a2e64e853be92728f05aad9dc07bebe63c7448"}
#[doc = "How acronyms and terms are written where a word starts with a capital."]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, serde :: Deserialize, serde :: Serialize)]
pub enum AcronymStyle {
    #[doc = "HttpClient, Ipv6Address, GitHubRepo"]
    #[serde(rename = "CAPITALIZED")]
    Capitalized,
    #[doc = "HTTPClient, IPv6Address, GitHubRepo"]
    #[serde(rename = "UPPERCASE")]
    Uppercase,
}
