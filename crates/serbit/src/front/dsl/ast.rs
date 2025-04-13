use proc_macro2::Ident;

struct Item;
struct Expr;

enum Stage {
    Single(Item),
    Repeat { count: Expr, seq: Box<Seq> },
    Until { expr: Expr, seq: Box<Seq> },
}

struct Seq {
    stages: Vec<Stage>,
}

struct Msg {
    name: Ident,
    seq: Seq,
}
