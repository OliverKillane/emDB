// use super::*;

// TODO(oliverkillane): Think about this.

// pub struct Params;

// pub enum CtxKind {
//     Datas,
//     Stages(),
// }

// pub enum Error<N: ir::namer::Namer> {
//     MissingParams {

//     }
// }

// impl<'consts, 'datas, 'stages, N: ir::namer::Namer> Semantic<'consts, 'datas, 'stages, N>
//     for Params
// {
//     type Error = Error<N>;
// } 


// #[derive(Debug)]
// pub struct TypeCtxParams {
//     pub size_inclusive: Option<ir::datas::TypeSizeCtx>,
//     pub bool_inclusive: Option<ir::datas::TypeBoolCtx>,
// }



// impl <'data_types, N: ir::namer::Namer> ir::datas::DataType<'data_types, N> {
//     fn required_args(&self) -> TypeCtxParams {
//         match self {
//             ir::datas::DataType::Primitive(primitive) => {
//                 TypeCtxParams {
//                     size_inclusive: None,
//                     bool_inclusive: None,
//                 }
//             },
//             ir::datas::DataType::Struct { fields } => {

//             },
//             ir::datas::DataType::Array { data_type, num_items } => todo!(),
//             ir::datas::DataType::Union(cases) => todo!(),
//         }
//         unimplemented!()
//     }
// }
