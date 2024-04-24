use emdb::emql;

pub fn cool_thing(x: String) -> usize { x.len() }

emql! {
    impl my_db as Planviz {
        path = "scratch.dot",
        display_types = off,
        display_ctx_ops = off,
        display_control = off
    };
    impl my_sem as SemCheck;

    table people {
        name: String,
        friend: Option<String>,
    }

    query get_friendships() {
        use people |> map(x: usize = {use super::super::cool_thing; cool_thing(friend)}) |> collect(foo) ~> return;
    }
}

fn main() {}
