use better_arenas::arenas::{Arena, WriteArena};

pub mod expr;
pub mod plan;

struct Plan {
    stages: (), // arena for stages
    items: (),  // area for items
}

struct FixedPlan<
// need an arenaselector esque way to remove the bounds cycle
// TODO: 
// we can make a trait for datamaker, that has a generic associated type taking the key value
// we can then use this in the arena, to generate the data type used inside the arena
// we can then specialize, with a trait atop arena, that resolved for basic data

/*
trait ArenaData {
    type Data<Key>;
}
trait Arena<Data: ArenaData> {
    Data = ... Data::Data<Self::Key>   
}

and a
struct Plain<Data>;

impl<Data> ArenaData for Plan<Data> {
    type Data = Data;
}
*/

    Stages: Arena<plan::Stage<Items::Key, Stages::Key>>,
    Items: Arena<plan::Item<Items::Key>>,
> {
    stages: Stages,
    items: Items,
}