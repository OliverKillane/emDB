use std::collections::HashMap;

use smart_arenas::arena::Arena;

use super::*;

/// Uniqueness of names is enforced in:
///  - [ir::datas::DataType::Struct]
///  - [ir::stages::Stage]es (within a message)
///  - [ir::Plan::messages] within a plan
struct Names;

#[derive(Debug, Clone)]
pub enum NameKind {
    Message,
    StageInMessage,
    StructField,
    Data,
    Union,
}

#[derive(thiserror::Error, Debug)]
#[error("Duplicate {kind:?} name: `{first}` repeated at `{second}`")]
pub struct Error<N: ir::namer::Namer> {
    pub kind: NameKind,
    pub first: N::Ident,
    pub second: N::Ident,
}

impl<'consts, 'datas, 'stages, N: ir::namer::Namer> Semantic<'consts, 'datas, 'stages, N>
    for Names
{
    type Error = Error<N>;

    fn analyse(plan: &ir::Plan<'consts, 'datas, 'stages, N>) -> Result<(), Self::Error> {
        check_from_iter(NameKind::Message, plan.messages.iter())?;
        check_from_iter(NameKind::Data, plan.datas.iter())?;

        for data in plan.datas.iter() {
            match &data.data {
                ir::datas::DataType::Struct { fields } => {
                    check_from_iter(NameKind::StructField, fields.iter())?;
                }
                ir::datas::DataType::Union(fields) => {
                    check_cases(&fields)?;
                }
                _ => (),
            }
        }

        for stage in plan.stages.iter() {
            if let ir::stages::Stage::Choice(cases) = &stage.data {
                check_cases(&cases)?;
            }
        }

        for msg in &plan.messages {
            check_message(&plan.stages, &mut HashMap::new(), &msg.data)?;
        }

        Ok(())
    }
}

fn check_duplidate<'brw, N: ir::namer::Namer>(
    kind: NameKind,
    seen: &mut HashMap<N::Name, N::Ident>,
    new: &N::Ident,
) -> Result<(), Error<N>> {
    if let Some(existing) = seen.get(&N::Name::from(new.clone())) {
        Err(Error {
            kind,
            first: existing.clone(),
            second: new.clone(),
        })
    } else {
        seen.insert(N::Name::from(new.clone()), new.clone());
        Ok(())
    }
}

fn check_cases<N: ir::namer::Namer, E, C>(
    cases: &ir::utils::Cases<N, E, C>,
) -> Result<(), Error<N>> {
    let mut duplicates = HashMap::new();
    cases
        .cases
        .iter()
        .try_for_each(|case| check_duplidate(NameKind::Union, &mut duplicates, &case.ident))?;
    if let Some(otherwise) = &cases.otherwise {
        check_duplidate(NameKind::Union, &mut duplicates, &otherwise.ident)?;
    }
    Ok(())
}

fn check_from_iter<'brw, D: 'brw, N: ir::namer::Namer>(
    kind: NameKind,
    names: impl Iterator<Item = &'brw ir::namer::Assigned<N, D>>,
) -> Result<(), Error<N>> {
    let mut seen = HashMap::<N::Name, N::Ident>::new();
    for ir::namer::Assigned { ident, data: _ } in names {
        check_duplidate(kind.clone(), &mut seen, ident)?;
    }
    Ok(())
}

fn check_message<'brw, 'stages, N: ir::namer::Namer>(
    arena: &'brw ir::arenas::Stages<'stages, '_, N>,
    seen: &mut HashMap<N::Name, N::Ident>,
    start: &ir::keys::Stages<'stages>,
) -> Result<(), Error<N>> {
    let ir::namer::Assigned { ident, data } = arena.read(&start);
    check_duplidate(NameKind::StageInMessage, seen, ident)?;
    match data {
        ir::stages::Stage::Instance {
                        input_ctx: _,
                        data: _,
                        append_to_msg_ctx: _,
            } => {}
        ir::stages::Stage::Sequence { stages } => {
                for stage in stages {
                    check_message(arena, seen, stage)?;
                }
            }
        ir::stages::Stage::Until {
                condition: _,
                stage,
                include_end: _,
            } | ir::stages::Stage::Repeat { size: _, stage } => {
                check_message(arena, seen, stage)?;
            }
        ir::stages::Stage::Choice(cases) => {
                for case in &cases.cases {
                    check_message(arena, seen, &case.data.case)?;
                }
                if let Some(otherwise) = &cases.otherwise {
                    check_message(arena, seen, &otherwise.data)?;
                }
            }
    }

    Ok(())
}
