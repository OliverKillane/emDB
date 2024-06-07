use glob::glob;
use trybuild::TestCases;

/// Test compilation failure, and the error messages produced match expected
#[test]
fn should_fail() {
    let t = TestCases::new();
    for entry in glob("tests/invalid/**/*.rs").unwrap() {
        t.compile_fail(entry.unwrap());
    }
}

/// Overcomplicated macro because importing each separately is boring + IDE picks
/// it up nicely
macro_rules! valid_tests {
    ( $($section:ident { $($test:ident),+ } ),+ ) => {
        mod valid;
        $(
            mod $section {
                $(
                    #[test]
                    fn $test() {
                        super::valid::$section::$test::test();
                    }
                )+
            }
        )+
    };
}

valid_tests!(
    complex {
        favourite_colours,
        dereferencing,
        userdetails
    },
    context {
        lift_stream,
        lift_single,
        groupby
    },
    extreme {
        empty_emql,
        empty_items,
        just_maths
    },
    simple {
        no_errors,
        basic_join,
        limited_table,
        sums,
        counts
    }
);
