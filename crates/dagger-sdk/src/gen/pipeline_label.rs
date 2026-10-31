//! Generated bindings owned by the GraphQL `PipelineLabel` type.
// @generated {"format":"dagger-rust-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:d606e02207a4265f6c76adc93f73153a694ceaf51c12d36b2a3b8baf131ff252","target_revision":"f0a3d728cf7012e0954f772b5dc5c8471648afaa"}
#[doc = "Key value object that represents a pipeline label."]
#[derive(Clone, Debug, PartialEq, serde :: Deserialize, serde :: Serialize)]
#[non_exhaustive]
pub struct PipelineLabel {
    #[doc = "Label name."]
    #[serde(rename = "name")]
    pub name: String,
    #[doc = "Label value."]
    #[serde(rename = "value")]
    pub value: String,
}
impl PipelineLabel {
    #[doc = "Creates `PipelineLabel` with every required GraphQL input field."]
    #[must_use]
    pub fn new(name: String, value: String) -> Self {
        Self { name, value }
    }
}
