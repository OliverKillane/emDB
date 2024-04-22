use emdb::emql;

emql! {
    table people {
        name: String,
        friend: Option<String>,
    }

    query get_friendships() {
        use people |> fork(let person, friend);

        join(use person (left pred(
            if let Some(friend_name) = person.friend {
                friend_name == friend.name
            } else {
                false
            }
        )) use friend)
            |> map(peep: String = person.name, buddy: String = friend.name)
            |> collect(friends as type friendship)
            ~> return;
    }
}

fn main() {}
