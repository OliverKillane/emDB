//! Not implemented yet

use emdb::emql;

emql! {
    
    table customers {
        forename: String,
        surname: String,
        age: u8,
    } @ [pred(age < 256) as sensible_ages]

    // To be implemented later
    // query get_sensible_ages() {
    //     use customers
    //         |> groupby( age for { 
    //                 map(len: usize = forename.len(), surname: String = surname ) 
    //              |> sort(len desc) 
    //              |> collect(type foo)
    //           })
    //         |> map(age, b: type foo = )
    //         ~> return;
    // }

}

fn main() {}