//! What generated bindings cannot reach in the SDK: its private session and selection
//! state, the resolver behind a generated ID argument, and the sealed constructor of
//! a Core handle.

#![cfg(feature = "gen")]

#[test]
fn generated_client_compile_contract() {
    let cases = trybuild::TestCases::new();
    cases.compile_fail("tests/generated-ui/fail/private_runtime_state.rs");
    cases.compile_fail("tests/generated-ui/fail/raw_id_resolver.rs");
    cases.compile_fail("tests/generated-ui/fail/sealed_core_loadable.rs");
}
