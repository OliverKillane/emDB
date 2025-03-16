use crate::{
    plan::ir::{Arenas, ExtraData, plan},
    utils::typeget::Has,
};
use better_arenas::arenas::Arena;
use std::fmt::Debug;

pub fn debug<Extra, Alias>(plan: &super::ir::Plan<Extra, impl Arenas<Extra>>)
where
    Extra: ExtraData,
    Alias: Debug,
    Extra::Item: Has<Alias>,
{
    // TODO: Example for iterating over the arena, but we want to go further
    //        - Should be able to check for debug information
    for entry in plan.items.iter() {
        let alias = entry.extra.get();
        println!("alias is: {alias:?}");

        match &entry.data {
            plan::Item::Array(array) => todo!(),
            plan::Item::Choice(choice) => todo!(),
            plan::Item::Tuple(tuple) => todo!(),
            plan::Item::Primitive(primitive) => match primitive {
                plan::Primitive::Byte(byte) => todo!(),
                plan::Primitive::Bit(bit) => todo!(),
                plan::Primitive::Integer(integer) => todo!(),
            },
        }
    }

    for entry in plan.stages.iter() {
        match &entry.data {
            plan::Stage::Item(item) => {
                let item2 = plan.items.read(item);
            }
            plan::Stage::Repeat(repeat) => todo!(),
            plan::Stage::Until(until) => todo!(),
            plan::Stage::Seq(seq) => todo!(),
        }
    }
}
