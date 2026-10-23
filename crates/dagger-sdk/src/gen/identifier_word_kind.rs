//! Generated bindings owned by the GraphQL `IdentifierWordKind` type.
// @generated {"format":"dagger-rust-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:fa63be22540138c2888c5779db388e117bf8b168d9f7f59b2cb2a77dc4cc9f4d","target_revision":"a7a2e64e853be92728f05aad9dc07bebe63c7448"}
#[doc = "The kind of a word in an identifier."]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, serde :: Deserialize, serde :: Serialize)]
pub enum IdentifierWordKind {
    #[doc = "An acronym, from the dictionary or a run of capitals."]
    #[serde(rename = "ACRONYM")]
    Acronym,
    #[doc = "A dictionary term with a fixed mixed-case spelling (GitHub, IPv6)."]
    #[serde(rename = "TERM")]
    Term,
    #[doc = "An ordinary word."]
    #[serde(rename = "WORD")]
    Word,
}
