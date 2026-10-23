//! Generated bindings owned by the GraphQL `GitPushDisposition` type.
// @generated {"format":"dagger-rust-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:fa63be22540138c2888c5779db388e117bf8b168d9f7f59b2cb2a77dc4cc9f4d","target_revision":"a7a2e64e853be92728f05aad9dc07bebe63c7448"}
#[doc = "How a Git push updated the remote ref."]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, serde :: Deserialize, serde :: Serialize)]
pub enum GitPushDisposition {
    #[doc = "The remote ref was created."]
    #[serde(rename = "CREATED")]
    Created,
    #[doc = "The remote ref was fast-forwarded."]
    #[serde(rename = "FAST_FORWARD")]
    FastForward,
    #[doc = "The remote ref was replaced under an explicit lease."]
    #[serde(rename = "FORCED")]
    Forced,
    #[doc = "The remote ref already pointed to this commit."]
    #[serde(rename = "UP_TO_DATE")]
    UpToDate,
}
