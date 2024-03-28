//! Not implemented yet

use emdb::database;

database! {
    
    table customers {
        forename: String,
        surname: String,
        age: u8,
    } @ [pred(age < 256) as sensible_ages]

    query get_sensible_ages() {
        use customers
            |> groupby( age for { 
                    map(len: usize = forename.len(), surname: String = surname ) 
                 |> sort(len desc) 
                 |> first() 
              })
            |> return;
    }

}