//! Compile fixtures for the public module-authoring attributes.

#[test]
fn module_authoring_attributes_compile_or_fail_at_the_authored_item() {
    let cases = trybuild::TestCases::new();
    cases.pass("tests/fixtures/module_authoring/pass/*.rs");
    // These fixtures name generated Core types, such as a collection's `CollectionDelta`.
    #[cfg(feature = "gen")]
    cases.pass("tests/fixtures/module_authoring/pass_gen/*.rs");
    cases.compile_fail("tests/fixtures/module_authoring/fail/*.rs");
}
