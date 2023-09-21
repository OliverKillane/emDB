use emdb::database;

enum State {
    Young,
    Old,
}

database!(
    name MapExample;

    table people {
        firstname: smalltext[6],
        surname: smalltext[7],
        age: i32,
        state: `State`,
    } @ unique[first_name, second_name];

    query add_newborn(fname: `&str`, sname: `&str`) = {
        people <| insert(
                    firstname = fname, 
                    surname = sname,
                    age = 0,
                    state = `::State::Young`
                );
    }

    query new_year() = {
        people
            <| update(`|row| row.age += 1`)
            |> size()
            |> return;
    }

    query print_members() {
        people
            |> foreach(`|row| println!("Safe print from transaction! {}", row)`)
            |> size()
            |> return;
    }
);

fn demo() {
    let db = MapExample::DB::new();

    assert!(db.add_newborn("bob", "smith").is_ok());
    assert!(db.add_newborn("bob", "smith").is_err()); // unique constraint violation
    db.print_members();
    db.new_year();
    db.print_members();
}