use std::collections::HashMap;
use smart_arenas::arena::Arena;
use super::*;

pub struct Names;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error<N: ir::Namer> {
    Duplicate {
        name: N::Name,
        old: N::Span,
        new: N::Span,
    },
}

impl<'msgs, 'seqs, 'stages, 'items, 'bools, 'ints, 'consts, N: ir::Namer> Semantic<'msgs, 'seqs, 'stages, 'items, 'bools, 'ints, 'consts, N> for Names {
    type Error = Error<N>;

    fn analyse(
        plan: &ir::Plan<'msgs, 'seqs, 'stages, 'items, 'bools, 'ints, 'consts, N>,
    ) -> Result<(), Self::Error> {
        let mut usages: HashMap<N::Name, N::Span> = HashMap::new();

        let msgs = plan.msgs.iter().map(ir::Assigned::get_name_and_span);
        let items = plan.items.iter().map(ir::Assigned::get_name_and_span);
        let consts = plan.consts.iter().map(ir::Assigned::get_name_and_span);

        for (name, new) in msgs.chain(items).chain(consts) {
            if let Some(old) = usages.insert(name.clone(), new.clone()) {
                return Err(Error::Duplicate {
                    name: name,
                    old,
                    new: new,
                });
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        plan::{
            helpers::tests::*,
            ir::{Assigned, Spanned},
        },
        plan_tests,
    };
    use smallvec::smallvec;

    fn basic<'msgs, 'seqs, 'stages, 'items, 'bools, 'ints, 'consts>(
        mut plan: ir::Plan<'msgs, 'seqs, 'stages, 'items, 'bools, 'ints, 'consts, TestNamer>,
    ) {
        let item = plan
            .items
            .insert(ir::Assigned {
                ident: TestIdent {
                    name: TestName("item1"),
                    span: TestSpan(1),
                },
                data: Spanned {
                    span: TestSpan(2),
                    data: ir::Item::Primitive(ir::Primitive::Byte(ir::Constraint::None)),
                },
            })
            .unwrap();

        let stage = plan.stages.insert(ir::Stage::Single(item)).unwrap();

        let seq = plan
            .seqs
            .insert(ir::Seq {
                stages: smallvec![stage],
            })
            .unwrap();

        plan.msgs
            .insert(Assigned {
                ident: TestIdent {
                    name: TestName("msg1"),
                    span: TestSpan(3),
                },
                data: Spanned {
                    span: TestSpan(4),
                    data: ir::Msg { seq },
                },
            })
            .unwrap();

        let name = TestName("const");
        let old = TestSpan(5);
        let new = TestSpan(8);

        plan.consts
            .insert(ir::Assigned {
                ident: TestIdent {
                    name: name.clone(),
                    span: old.clone(),
                },
                data: Spanned {
                    span: TestSpan(6),
                    data: ir::Constant {
                        value: ir::Primitive::Byte(b'A'),
                    },
                },
            })
            .unwrap();

        assert_eq!(Names::analyse(&plan), Ok(()));

        plan.consts
            .insert(ir::Assigned {
                ident: TestIdent {
                    name: name.clone(),
                    span: new.clone(),
                },
                data: Spanned {
                    span: TestSpan(8),
                    data: ir::Constant {
                        value: ir::Primitive::Byte(b'B'),
                    },
                },
            })
            .unwrap();

        assert_eq!(
            Names::analyse(&plan),
            Err(Error::Duplicate { name, old, new })
        );
    }

    plan_tests! {inner => basic,}
}
