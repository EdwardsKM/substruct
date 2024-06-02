#[test]
fn test_from_str() {
    let t = trybuild::TestCases::new();
    t.pass("tests/01-structs.rs");
    //t.compile_fail("tests/02-enum-fail.rs");
    // Add more test cases as needed.
    // For example, to test for expected failures, you can use:
    // t.compile_fail("tests/02-compile-fail.rs");
}
