use proc_macro2::Ident;

struct Bool;
struct Int;

struct Item {}
struct Expr;

pub enum Stage {
    Single(Item),
    Repeat { count: Int, seq: Box<Seq> },
    Until { expr: Bool, seq: Box<Seq> },
}

struct Seq {
    stages: Vec<Stage>,
}

struct Msg {
    name: Ident,
    seq: Seq,
}
