#[allow(dead_code, clippy::upper_case_acronyms)]
#[derive(Clone)]
enum RGB {
    Red,
    Green,
    Blue,
}

pulpit::macros::simple! {
    fields {
        name: String,
        id: usize @ unique(unique_reference_number),
        age: u8,
        fav_rgb_colour: crate::RGB,
    },
    updates {
        update_age: [age],
    },
    gets {
        get_all: [name, id, age, fav_rgb_colour],
    },
    predicates {
        adults_only: *age > 18,
        age_cap: *age < 100,
    },
    limit {
        cool_limit: 2000
    },
    transactions: on,
    deletions: on,
    name: bowling_club
}

// TODO: Write some tests (other than that the generated code compiles)
