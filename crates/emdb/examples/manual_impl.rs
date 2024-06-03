use std::cmp::max;

use minister::{Basic, Physical};
use pulpit::macros::simple;

#[allow(dead_code)]
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
    predicates {
        adults_only: *age > 18,
        age_cap: *age < 100,
    },
    transactions: on,
    deletions: on,
    name: bowling_club
}

impl<'imm> bowling_club::Window<'imm> {
    fn get_longest_name(&self) -> usize {
        let keys = Basic::consume_stream(self.scan());
        let vals = Basic::map(keys, |key| self.get(key));
        let lengths = Basic::map(vals, |val| match val {
            Ok(row) => row.name.len(),
            Err(_) => 0,
        });
        let max = Basic::combine(lengths, max);
        Basic::export_single(max)
    }
}

fn main() {
    let mut t = bowling_club::Table::new(1024);
    let w = t.window();
    w.get_longest_name();
}
