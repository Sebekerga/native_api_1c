#[test]
fn tests() {
    let t = trybuild::TestCases::new();

    t.pass("tests/to_build/readme_example.rs");
}
