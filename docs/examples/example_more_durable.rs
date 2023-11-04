use emdb::database;
use crate::helpers::{backup_to_file, read_from_file};

database!(
    name Special;

    table durable_stuff {
        id: u32 @ unique,
        body: bigtext,
    }

    query add_stuff(content: `&str`) = {
        durable_stuff 
            |> size()
            |> let durable_size;

        durable_stuff 
            <| insert(id = durable_size, body = content);
    
        durable_size 
            |> return;
    }

    query retreive(id: u32) = {
        durable_stuff
            |> unique(it[id], id)
            |> map_one(it[body])
            |> return;
    }

    query get_all() = {
        durable_stuff |> collect[`Vec`] |> return;
    }

    query from_all(all: `&[(i32, &'str)]`) = {
        durable_stuff
            <| delete()
            <| all
            |> size()
            |> return;
    }
);

static db = Special::DB::new();

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // on panick, save the contents.
    backup_to_file(db.get_all())
}

fn demo() {
    let content = read_from_file();
    assert!(!content.is_empty());
    db.from_all(content).expect("Should be no duplicates in file");
    db.add_stuff(content.len() - 1).expect("Valid ID");

    // but oh no! Some faulty application logic
    if 1 + 1 != 3 {
        panic!()
    }
}
