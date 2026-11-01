//! Generated bindings owned by the GraphQL `FileType` type.
// @generated {"format":"dagger-rust-client-v1","ownership":"dagger-codegen","schema_digest":"sha256:d606e02207a4265f6c76adc93f73153a694ceaf51c12d36b2a3b8baf131ff252","target_revision":"f0a3d728cf7012e0954f772b5dc5c8471648afaa"}
#[doc = "File type."]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, serde :: Deserialize, serde :: Serialize)]
pub enum FileType {
    #[doc = "directory file type"]
    #[serde(rename = "DIRECTORY", alias = "DIRECTORY_TYPE")]
    Directory,
    #[doc = "regular file type"]
    #[serde(rename = "REGULAR", alias = "REGULAR_TYPE")]
    Regular,
    #[doc = "symlink file type"]
    #[serde(rename = "SYMLINK", alias = "SYMLINK_TYPE")]
    Symlink,
    #[doc = "unknown file type"]
    #[serde(rename = "UNKNOWN")]
    Unknown,
}
impl crate::__private::ModuleInputCodec for FileType {
    fn decode_input(
        value: &crate::__private::ModuleJson,
        _context: &crate::__private::ModuleContextBase,
    ) -> Result<Self, crate::__private::DecodeError> {
        crate::__private::decode_string_value(value)
    }
}
impl crate::__private::ModuleOutputCodec for FileType {
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
