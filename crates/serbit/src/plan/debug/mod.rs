use crate::{
    plan::ir::{Arenas, ExtraData, plan},
    utils::typeget::Has,
};
use better_arenas::arenas::Arena;
use std::fmt::Debug;

pub fn debug<Extra: ExtraData, Alias: Debug>(plan: &super::ir::Plan<Extra, impl Arenas<Extra>>)
where
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
}
