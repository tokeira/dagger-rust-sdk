//! Generated bindings owned by the GraphQL `IdentifierWordKind` type.
// @generated {"format":"dagger-rust-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:d606e02207a4265f6c76adc93f73153a694ceaf51c12d36b2a3b8baf131ff252","target_revision":"f0a3d728cf7012e0954f772b5dc5c8471648afaa"}
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
