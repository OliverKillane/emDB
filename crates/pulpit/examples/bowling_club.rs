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

fn main() {
    // We generate a basic table, and open a window into it
    let mut x = bowling_club::Table::new(1024);
    let mut w = x.window();

    // We can then insert some data, which is checked against the predicates and unique constraints
    let bill_key = w
        .insert(bowling_club::insert::Insert {
            id: 0,
            fav_rgb_colour: RGB::Blue,
            name: String::from("Bill Bob"),
            age: 50,
        })
        .unwrap();

    // We can also update the data using the update method we described in the macro
    w.update_age(
        bowling_club::updates::update_age::Update { age: 51 },
        bill_key,
    )
    .unwrap();

    // The count is correct
    assert_eq!(w.count(), 1);

    // By committing the data, it can no longer be easily rolled back
    w.commit();

    // We try with another insert, however the age constraint is breached, so it fails
    let fred_insert = w.insert(bowling_club::insert::Insert {
        id: 1,
        fav_rgb_colour: RGB::Red,
        name: String::from("Fred Dey"),
        age: 101,
    });
    assert!(matches!(
        fred_insert,
        Err(bowling_club::insert::Error::age_cap)
    ));

    // With an updated age we can now insert
    let fred_key = w
        .insert(bowling_club::insert::Insert {
            id: 1,
            fav_rgb_colour: RGB::Red,
            name: String::from("Fred Dey"),
            age: 30,
        })
        .unwrap();

    // We can grab data from the table, as a retaining arena is used for the table, and we do not
    // update the names, we can pull references to the names that live as long as `w` (the window)
    let names = vec![w.get(fred_key).unwrap().name, w.get(bill_key).unwrap().name];

    // After deciding fred is not so cool, we roll back and un-insert him
    assert_eq!(w.count(), 2);
    w.abort();
    assert_eq!(w.count(), 1);

    // While the mutable data for the table is removed, the names are still valid & safely accessible
    // by these references until the window is destroyed.
    println!("{} and {}", names[0], names[1]);

    // we can hence discover that fred is no longer present by trying to get his reference_number
    assert!(matches!(
        w.unique_reference_number(&1),
        Err(bowling_club::unique::NotFound)
    ));
}
