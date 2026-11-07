//! Generated bindings owned by GraphQL coordinate `MinimalConfig`.
// @generated {"format":"dagger-rust-standalone-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:506c57349967eeaceb1235a4465193229f99ff7b15c2a58a4e8fd04be3d5008f","target_revision":"a45241b20a5e07fbfc5954479ef0b61da843499c"}
#[doc = "Client fixture input MinimalConfig."]
#[derive(Clone, Debug, dagger_sdk::__private::serde::Serialize)]
#[serde(crate = "dagger_sdk::__private::serde")]
#[non_exhaustive]
pub struct Config {
    #[doc = "Client fixture argument enabled."]
    #[serde(rename = "enabled", skip_serializing_if = "Option::is_none")]
    pub enabled: Option<Option<bool>>,
}
impl Config {
    /// Creates `Config` with every required GraphQL input.
    #[must_use]
    pub fn new() -> Self {
        Self { enabled: None }
    }
    /// Supplies GraphQL input `enabled`; calling this method preserves explicit null and zero values.
    #[must_use]
    pub fn with_enabled(mut self, value: bool) -> Self {
        self.enabled = Some(Some(value));
        self
    }
    /// Supplies an explicit GraphQL null for input `enabled` rather than omitting it.
    #[must_use]
    pub fn with_enabled_null(mut self) -> Self {
        self.enabled = Some(None);
        self
    }
}
impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}
