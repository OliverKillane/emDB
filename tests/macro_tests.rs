use emdb::test_macro;

test_macro!( name MapExample;

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
    });
