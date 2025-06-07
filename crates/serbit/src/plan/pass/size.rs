use std::num::NonZero;
use crate::plan::ir;
use smart_arenas::prelude::*;

#[derive(PartialEq, Eq, Debug)]
pub enum ItemSize<'ints> {
    Fixed(NonZero<usize>),
    Repeat {
        item: Box<Self>,
        repeat: ir::keys::Int<'ints>,
    },
    Sum(Vec<Self>),
}

pub fn size<'msgs, 'seqs, 'stages, 'items, 'bools, 'ints, 'consts, 'exprs, N: ir::Namer>(
    plan: &ir::Plan<'msgs, 'seqs, 'stages, 'items, 'bools, 'ints, 'consts, N>,
    item: &ir::keys::Item<'items>,
) -> ItemSize<'ints> {
    match &plan.items.read(item).data.data {
        ir::Item::Array { count, item } => ItemSize::Repeat {
            item: Box::new(size(plan, item)),
            repeat: plan.ints.copy_key(count),
        },
        ir::Item::Tuple { items } => ItemSize::Sum(items.iter().map(|i| size(plan, i)).collect()),
        ir::Item::Primitive(primitive) => ItemSize::Fixed(primitive.size()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        plan::helpers::tests::{TestIdent, TestName, TestNamer, TestSpan},
        plan_tests,
    };
    use smallvec::smallvec;

    fn basic<'msgs, 'seqs, 'stages, 'items, 'bools, 'ints, 'consts>(
        mut plan: ir::Plan<'msgs, 'seqs, 'stages, 'items, 'bools, 'ints, 'consts, TestNamer>,
    ) {
        let integer_bits = NonZero::new(3).unwrap();
        let integer_item = plan
            .items
            .insert(ir::Assigned {
                ident: TestIdent {
                    name: TestName("foo"),
                    span: TestSpan(0),
                },
                data: ir::Spanned {
                    span: TestSpan(1),
                    data: ir::Item::Primitive(ir::Primitive::Integer {
                        underlying: ir::Integer {
                            signed: false,
                            bits: integer_bits,
                            encoding: ir::Encoding::LittleEndian,
                        },
                        assoc: ir::Constraint::None,
                    }),
                },
            })
            .unwrap();

        let integer_expr = plan
            .ints
            .insert(ir::Spanned {
                span: TestSpan(4),
                data: ir::Int::Ref(plan.items.copy_key(&integer_item)),
            })
            .unwrap();

        let array_item = plan
            .items
            .insert(ir::Assigned {
                ident: TestIdent {
                    name: TestName("foo"),
                    span: TestSpan(0),
                },
                data: ir::Spanned {
                    span: TestSpan(2),
                    data: ir::Item::Array {
                        count: plan.ints.copy_key(&integer_expr),
                        item: plan.items.copy_key(&integer_item),
                    },
                },
            })
            .unwrap();

        let tuple_item = plan
            .items
            .insert(ir::Assigned {
                ident: TestIdent {
                    name: TestName("bar"),
                    span: TestSpan(1),
                },
                data: ir::Spanned {
                    span: TestSpan(2),
                    data: ir::Item::Tuple {
                        items: smallvec![
                            plan.items.copy_key(&array_item),
                            plan.items.copy_key(&integer_item)
                        ],
                    },
                },
            })
            .unwrap();

        let integer_size = ItemSize::Fixed(integer_bits.into());
        assert_eq!(&size(&plan, &integer_item), &integer_size);

        let array_size = ItemSize::Repeat {
            item: Box::new(integer_size),
            repeat: integer_expr,
        };
        assert_eq!(&size(&plan, &array_item), &array_size);

        assert_eq!(
            size(&plan, &tuple_item),
            ItemSize::Sum(vec![array_size, ItemSize::Fixed(integer_bits.into())])
        )
    }

    plan_tests! {inner => basic,}
}
