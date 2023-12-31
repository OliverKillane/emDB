// use emdb::{bob, database};

// database! {
//     name my_databaseaa;
//     query bob (foo){
//         bar
//     };
//     table bar
// }

// database! {

//     name mydatabase;

//     query(x: i32, y: usize) = {
//         name |> map(|x| a) |> let result;
//         result |> groupby(field, it |> expr() |> unique()) |> let foo;

//         result |> someexpr() |> let bar;

//         tablename ~> filter() <! insert(something);
//     }

//     table tablename {
//         field: i32,
//         field2: String,
//         field3: bool,
//     } @ pk(field), unique(field2);

// }

// use emdb::database;

// database! {
//     name databaseaaaa;

//     table aaa {
//         a: i32
//     }

//     table aaa {

//     }

//     // query bob() {

//     // }
// }
