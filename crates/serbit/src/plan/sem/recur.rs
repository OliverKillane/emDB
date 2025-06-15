use indexmap::IndexSet;
use smart_arenas::arena::Arena;

use super::*;

pub enum Error<N: ir::namer::Namer> {
    StageRecursion {
        message: N::Ident,
        path: IndexSet<N::Ident>,
        repeat: N::Ident,
    },
    DataRecursion {
        path: IndexSet<N::Ident>,
        repeat: N::Ident,
    },
}

struct Recursion;

impl<'consts, 'datas, 'stages, N: ir::namer::Namer> Semantic<'consts, 'datas, 'stages, N>
    for Recursion
{
    type Error = Error<N>;

    fn analyse(plan: &ir::Plan<'consts, 'datas, 'stages, N>) -> Result<(), Self::Error> {
        let mut datas_visted = IndexSet::with_capacity(plan.datas.len());
        let mut stages_visited = IndexSet::with_capacity(plan.stages.len());

        for (wk, _) in plan.datas.iter_with_weak_key() {
            check_data(&plan.datas, &mut datas_visted, IndexSet::new(), wk.key())?;
        }

        for msg in plan.messages.iter() {
            check_stage(
                &plan.stages,
                &mut stages_visited,
                &msg.ident,
                IndexSet::new(),
                &msg.data,
            )?;
        }
        Ok(())
    }
}

fn check_data<'datas, N: ir::namer::Namer>(
    datas: &ir::arenas::Datas<'datas, N>,
    visited: &mut IndexSet<N::Ident>,
    mut path: IndexSet<N::Ident>,
    key: &ir::keys::Datas<'datas>,
) -> Result<IndexSet<N::Ident>, Error<N>> {
    let ir::namer::Assigned { ident, data } = datas.read(key);

    if !visited.insert(ident.clone()) {
        Ok(path)
    } else {
        if path.insert(ident.clone()) {
            match data {
                ir::datas::DataType::Struct { fields } => {
                    for field in fields {
                        path = check_data(datas, visited, path, &field.data.data.key)?;
                    }
                }
                ir::datas::DataType::Array {
                    data_type,
                    num_items: _,
                } => path = check_data(datas, visited, path, &data_type.key)?,
                ir::datas::DataType::Union(cases) => {
                    for case in &cases.cases {
                        path = check_data(datas, visited, path, &case.data.case.key)?;
                    }
                    if let Some(otherwise) = &cases.otherwise {
                        path = check_data(datas, visited, path, &otherwise.data.key)?;
                    }
                }
                ir::datas::DataType::Primitive(_) => {}
            }
            path.pop().unwrap();
            Ok(path)
        } else {
            Err(Error::DataRecursion {
                path,
                repeat: ident.clone(),
            })
        }
    }
}

fn check_stage<'stages, N: ir::namer::Namer>(
    stages: &ir::arenas::Stages<'stages, '_, N>,
    visited: &mut IndexSet<N::Ident>,
    message: &N::Ident,
    mut path: IndexSet<N::Ident>,
    key: &ir::keys::Stages<'stages>,
) -> Result<IndexSet<N::Ident>, Error<N>> {
    let ir::namer::Assigned { ident, data } = stages.read(key);
    if !visited.insert(ident.clone()) {
        Ok(path)
    } else {
        if path.insert(ident.clone()) {
            match data {
                ir::stages::Stage::Instance {
                    input_ctx: _,
                    data: _,
                    append_to_msg_ctx: _,
                } => {}
                ir::stages::Stage::Choice(cases) => {
                    for case in &cases.cases {
                        path = check_stage(stages, visited, message, path, &case.data.case)?;
                    }
                    if let Some(otherwise) = &cases.otherwise {
                        path = check_stage(stages, visited, message, path, &otherwise.data)?;
                    }
                }
                ir::stages::Stage::Sequence { stages: seq_stages } => {
                    for stage in seq_stages {
                        path = check_stage(stages, visited, message, path, stage)?;
                    }
                }
                ir::stages::Stage::Until {
                    condition: _,
                    stage,
                    include_end: _,
                }
                | ir::stages::Stage::Repeat { size: _, stage } => {
                    path = check_stage(stages, visited, message, path, stage)?;
                }
            }
            path.pop().unwrap();
            Ok(path)
        } else {
            Err(Error::StageRecursion {
                message: message.clone(),
                path,
                repeat: ident.clone(),
            })
        }
    }
}
