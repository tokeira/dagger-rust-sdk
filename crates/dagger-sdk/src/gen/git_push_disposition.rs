//! Generated bindings owned by the GraphQL `GitPushDisposition` type.
// @generated {"format":"dagger-rust-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:d606e02207a4265f6c76adc93f73153a694ceaf51c12d36b2a3b8baf131ff252","target_revision":"f0a3d728cf7012e0954f772b5dc5c8471648afaa"}
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
