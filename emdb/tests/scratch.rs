use emdb::emql;

emql! {
    impl my_db as Planviz {
        path = "scratch.dot",
        display_types = off,
        display_ctx_ops = off,
        display_control = off
    };

    table people {
        name: String,
        friend: Option<String>,
    }

    query get_friendships() {
        use people |> fork(let person, friend);

        join(use person [
            left pred {
                if let Some(friend_name) = person.friend {
                    friend_name == friend.name
                } else {
                    false
                }
            }
        ] use friend)
            |> map(peep: String = person.name, buddy: String = friend.name)
            |> collect(friends)
            ~> return;
    }
}

fn main() {}
