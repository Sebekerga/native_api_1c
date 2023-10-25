#[test]
fn tests() {
    let t = trybuild::TestCases::new();

    t.pass("tests/to_build/props.rs");
}
