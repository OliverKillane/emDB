mod debug_code {
    #![allow(unused_variables)]
    #![allow(dead_code)]
    struct RecordType0 {
        surname: ScalarType1,
        forename: ScalarType0,
        age: ScalarType2,
    }
    struct RecordType1 {
        recordfield_internal_id_0: ScalarType4,
    }
    struct RecordType2 {
        recordfield_internal_id_1: ScalarType3,
    }
    struct RecordType3 {
        surname: ScalarType1,
        forename: ScalarType0,
    }
    struct RecordType4 {
        people: ScalarType5,
    }
    struct RecordType5 {
        group: ScalarType6,
        age_bracket: ScalarType7,
    }
    pub struct RecordType6 {
        pub brackets: ScalarType8,
    }
    type ScalarType0 = String;
    type ScalarType1 = String;
    type ScalarType2 = u8;
    type ScalarType3 = RecordType0;
    type ScalarType4 = TableRef0;
    type ScalarType5 = ();
    type ScalarType6 = ScalarType5;
    type ScalarType7 = u8;
    type ScalarType8 = ();
    /// Reference to the table
    pub struct TableRef0 {}
    /// this is a function
    pub fn customer_age_brackets() -> RecordType6 {
        let closures = (
            move |age: ScalarType2| {
                (
                    move |RecordType4 { people }| {
                        let result: RecordType5 = {
                            {
                                let group: ScalarType6 = people;
                                let age_bracket: ScalarType7 = age;
                                RecordType5 { group, age_bracket }
                            }
                        };
                        result
                    },
                )
            },
            move |RecordType5 { group, age_bracket }| {
                let result: bool = { age_bracket > 16 };
                result
            },
        );
        todo!()
    }
}
