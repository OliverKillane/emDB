#![allow(unused_variables)]
use emdb::macros::emql;

emql! {
    impl my_db as SimpleSerialized;

    table people {
        name: String,
        friend: Option<String>,
    }

    query get_friendships() {
        use people |> fork(let person, friend);

        join(use person [
            inner pred {
                if let Some(friend_name) = &left.friend {
                    friend_name == &right.name
                } else {
                    false
                }
            }
        ] use friend)
            |> map(peep: String = person.name, buddy: String = friend.name)
            |> collect(friends as type friendship)
            ~> return;
    }
}

fn main() {}
