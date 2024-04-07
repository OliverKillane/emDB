use glob::glob;
use trybuild::TestCases;

#[test]
fn should_compile() {
    let t = TestCases::new();
    for entry in glob("tests/valid/**/*.rs").unwrap() {
        t.pass(entry.unwrap());
    }
}

#[test]
fn should_fail() {
    let t = TestCases::new();

    for entry in glob("tests/invalid /**/*.rs").unwrap() {
        t.compile_fail(entry.unwrap());
    }
}
