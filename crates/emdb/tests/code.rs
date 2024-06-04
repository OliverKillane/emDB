mod my_db {
    #![allow(non_shorthand_field_patterns)]
    use emdb::dependencies::minister::Physical;
    pub mod tables {
        pub mod family_bonus {
            #![allow(unused, non_camel_case_types)]
            use emdb::dependencies::pulpit::column::{
                PrimaryWindow, PrimaryWindowApp, PrimaryWindowPull, PrimaryWindowHide,
                AssocWindow, AssocWindowPull, Column,
            };
            #[derive(Debug)]
            pub struct KeyError;
            mod column_types {
                //! Column types to be used for storage in each column.
                pub mod primary {
                    #[derive(Clone)]
                    pub struct Imm {
                        pub surname: String,
                    }
                    #[derive(Clone)]
                    pub struct Mut {
                        pub bonus: i32,
                    }
                    pub struct ImmUnpack<'imm> {
                        pub surname: &'imm String,
                    }
                    pub fn imm_unpack<'imm>(
                        Imm { surname }: &'imm Imm,
                    ) -> ImmUnpack<'imm> {
                        ImmUnpack { surname }
                    }
                }
            }
            pub mod borrows {
                pub struct Borrows<'brw> {
                    pub bonus: &'brw i32,
                    pub surname: &'brw String,
                }
            }
            impl<'imm> Window<'imm> {
                pub fn borrow<'brw>(
                    &'brw self,
                    key: Key,
                ) -> Result<borrows::Borrows<'brw>, KeyError> {
                    let emdb::dependencies::pulpit::column::Entry {
                        index,
                        data: primary,
                    } = match self.columns.primary.brw(key) {
                        Ok(entry) => entry,
                        Err(_) => return Err(KeyError),
                    };
                    Ok(borrows::Borrows {
                        bonus: &primary.mut_data.bonus,
                        surname: &primary.imm_data.surname,
                    })
                }
            }
            pub mod get {
                pub struct Get<'db> {
                    pub bonus: i32,
                    pub surname: &'db String,
                }
            }
            impl<'db> Window<'db> {
                pub fn get(&self, key: Key) -> Result<get::Get<'db>, KeyError> {
                    let emdb::dependencies::pulpit::column::Entry {
                        index,
                        data: primary,
                    } = match self.columns.primary.get(key) {
                        Ok(entry) => entry,
                        Err(_) => return Err(KeyError),
                    };
                    let primary = primary.convert_imm(column_types::primary::imm_unpack);
                    Ok(get::Get {
                        bonus: primary.mut_data.bonus,
                        surname: primary.imm_data.surname,
                    })
                }
            }
            pub mod updates {
                pub mod pulpit_access_7 {
                    #[derive(Debug)]
                    pub enum UpdateError {
                        KeyError,
                    }
                    pub struct Update {
                        pub bonus: i32,
                    }
                }
            }
            impl<'imm> Window<'imm> {
                pub fn pulpit_access_7(
                    &mut self,
                    update: updates::pulpit_access_7::Update,
                    key: Key,
                ) -> Result<(), updates::pulpit_access_7::UpdateError> {
                    let emdb::dependencies::pulpit::column::Entry {
                        index,
                        data: primary,
                    } = match self.columns.primary.brw_mut(key) {
                        Ok(entry) => entry,
                        Err(_) => {
                            return Err(updates::pulpit_access_7::UpdateError::KeyError);
                        }
                    };
                    let mut update = update;
                    std::mem::swap(&mut primary.mut_data.bonus, &mut update.bonus);
                    if !self.transactions.rollback {
                        self.transactions
                            .log
                            .push(
                                transactions::LogItem::Update(
                                    key,
                                    transactions::Updates::pulpit_access_7(update),
                                ),
                            );
                    }
                    Ok(())
                }
            }
            pub mod insert {
                pub struct Insert {
                    pub bonus: i32,
                    pub surname: String,
                }
                #[derive(Debug)]
                pub enum Error {
                    unique_surnames_cons,
                }
            }
            impl<'imm> Window<'imm> {
                pub fn insert(
                    &mut self,
                    insert_val: insert::Insert,
                ) -> Result<Key, insert::Error> {
                    let unique_surnames_cons = match self
                        .uniques
                        .surname
                        .lookup(&insert_val.surname)
                    {
                        Ok(_) => return Err(insert::Error::unique_surnames_cons),
                        Err(_) => insert_val.surname.clone(),
                    };
                    let primary = (emdb::dependencies::pulpit::column::Data {
                        imm_data: column_types::primary::Imm {
                            surname: insert_val.surname,
                        },
                        mut_data: column_types::primary::Mut {
                            bonus: insert_val.bonus,
                        },
                    });
                    let key = self.columns.primary.append(primary);
                    self.uniques.surname.insert(unique_surnames_cons, key).unwrap();
                    if !self.transactions.rollback {
                        self.transactions.log.push(transactions::LogItem::Append);
                    }
                    Ok(key)
                }
            }
            pub mod unique {
                #[derive(Debug)]
                pub struct NotFound;
            }
            impl<'imm> Window<'imm> {
                pub fn unique_surnames_cons(
                    &self,
                    value: &String,
                ) -> Result<Key, unique::NotFound> {
                    match self.uniques.surname.lookup(value) {
                        Ok(k) => Ok(k),
                        Err(_) => Err(unique::NotFound),
                    }
                }
            }
            mod transactions {
                pub enum Updates {
                    pulpit_access_7(super::updates::pulpit_access_7::Update),
                }
                pub enum LogItem {
                    Update(super::Key, Updates),
                    Append,
                }
                pub struct Data {
                    pub log: Vec<LogItem>,
                    pub rollback: bool,
                }
                impl Data {
                    pub fn new() -> Self {
                        Self {
                            log: Vec::new(),
                            rollback: false,
                        }
                    }
                }
            }
            impl<'imm> Window<'imm> {
                /// Commit all current changes
                /// - Clears the rollback log
                pub fn commit(&mut self) {
                    debug_assert!(! self.transactions.rollback);
                    self.transactions.log.clear()
                }
                /// Undo the transactions applied since the last commit
                /// - Requires re-applying all updates, deleting inserts and undoing deletes
                ///   (deletes' keys are actually just hidden until commit or abort)
                pub fn abort(&mut self) {
                    self.transactions.rollback = true;
                    while let Some(entry) = self.transactions.log.pop() {
                        match entry {
                            transactions::LogItem::Append => {
                                unsafe {
                                    self.columns.primary.unppend();
                                }
                            }
                            transactions::LogItem::Update(key, update) => {
                                match update {
                                    transactions::Updates::pulpit_access_7(update) => {
                                        self.pulpit_access_7(update, key).unwrap();
                                    }
                                }
                            }
                        }
                    }
                    self.transactions.rollback = false;
                }
            }
            impl<'imm> Window<'imm> {
                pub fn count(&self) -> usize {
                    self.columns.primary.count()
                }
            }
            impl<'db> Window<'db> {
                pub fn scan(&self) -> impl Iterator<Item = Key> + '_ {
                    self.columns.primary.scan()
                }
            }
            /// The key for accessing rows (delete, update, get)
            pub type Key = <emdb::dependencies::pulpit::column::AssocBlocks<
                column_types::primary::Imm,
                column_types::primary::Mut,
                1024usize,
            > as emdb::dependencies::pulpit::column::Keyable>::Key;
            mod predicates {}
            struct Uniques {
                surname: emdb::dependencies::pulpit::access::Unique<String, Key>,
            }
            impl Uniques {
                fn new(size_hint: usize) -> Self {
                    Self {
                        surname: emdb::dependencies::pulpit::access::Unique::new(
                            size_hint,
                        ),
                    }
                }
            }
            struct ColumnHolder {
                primary: emdb::dependencies::pulpit::column::AssocBlocks<
                    column_types::primary::Imm,
                    column_types::primary::Mut,
                    1024usize,
                >,
            }
            impl ColumnHolder {
                fn new(size_hint: usize) -> Self {
                    Self {
                        primary: emdb::dependencies::pulpit::column::AssocBlocks::new(
                            size_hint,
                        ),
                    }
                }
                fn window(&mut self) -> WindowHolder<'_> {
                    WindowHolder {
                        primary: self.primary.window(),
                    }
                }
            }
            struct WindowHolder<'imm> {
                primary: <emdb::dependencies::pulpit::column::AssocBlocks<
                    column_types::primary::Imm,
                    column_types::primary::Mut,
                    1024usize,
                > as emdb::dependencies::pulpit::column::Column>::WindowKind<'imm>,
            }
            pub struct Table {
                columns: ColumnHolder,
                uniques: Uniques,
                transactions: transactions::Data,
            }
            impl Table {
                pub fn new(size_hint: usize) -> Self {
                    Self {
                        columns: ColumnHolder::new(size_hint),
                        uniques: Uniques::new(size_hint),
                        transactions: transactions::Data::new(),
                    }
                }
                pub fn window(&mut self) -> Window<'_> {
                    Window {
                        columns: self.columns.window(),
                        uniques: &mut self.uniques,
                        transactions: &mut self.transactions,
                    }
                }
            }
            pub struct Window<'imm> {
                columns: WindowHolder<'imm>,
                uniques: &'imm mut Uniques,
                transactions: &'imm mut transactions::Data,
            }
        }
        pub mod customers {
            #![allow(unused, non_camel_case_types)]
            use emdb::dependencies::pulpit::column::{
                PrimaryWindow, PrimaryWindowApp, PrimaryWindowPull, PrimaryWindowHide,
                AssocWindow, AssocWindowPull, Column,
            };
            #[derive(Debug)]
            pub struct KeyError;
            mod column_types {
                //! Column types to be used for storage in each column.
                pub mod primary {
                    #[derive(Clone)]
                    pub struct Imm {
                        pub forename: String,
                        pub age: u8,
                        pub surname: String,
                    }
                    #[derive(Clone)]
                    pub struct Mut {
                        pub bonus_points: i32,
                    }
                    pub struct ImmUnpack<'imm> {
                        pub forename: &'imm String,
                        pub age: &'imm u8,
                        pub surname: &'imm String,
                    }
                    pub fn imm_unpack<'imm>(
                        Imm { forename, age, surname }: &'imm Imm,
                    ) -> ImmUnpack<'imm> {
                        ImmUnpack {
                            forename,
                            age,
                            surname,
                        }
                    }
                }
            }
            pub mod borrows {
                pub struct Borrows<'brw> {
                    pub age: &'brw u8,
                    pub surname: &'brw String,
                    pub bonus_points: &'brw i32,
                    pub forename: &'brw String,
                }
            }
            impl<'imm> Window<'imm> {
                pub fn borrow<'brw>(
                    &'brw self,
                    key: Key,
                ) -> Result<borrows::Borrows<'brw>, KeyError> {
                    let emdb::dependencies::pulpit::column::Entry {
                        index,
                        data: primary,
                    } = match self.columns.primary.brw(key) {
                        Ok(entry) => entry,
                        Err(_) => return Err(KeyError),
                    };
                    Ok(borrows::Borrows {
                        age: &primary.imm_data.age,
                        surname: &primary.imm_data.surname,
                        bonus_points: &primary.mut_data.bonus_points,
                        forename: &primary.imm_data.forename,
                    })
                }
            }
            pub mod get {
                pub struct Get<'db> {
                    pub bonus_points: i32,
                    pub forename: &'db String,
                    pub age: &'db u8,
                    pub surname: &'db String,
                }
            }
            impl<'db> Window<'db> {
                pub fn get(&self, key: Key) -> Result<get::Get<'db>, KeyError> {
                    let emdb::dependencies::pulpit::column::Entry {
                        index,
                        data: primary,
                    } = match self.columns.primary.get(key) {
                        Ok(entry) => entry,
                        Err(_) => return Err(KeyError),
                    };
                    let primary = primary.convert_imm(column_types::primary::imm_unpack);
                    Ok(get::Get {
                        age: primary.imm_data.age,
                        surname: primary.imm_data.surname,
                        bonus_points: primary.mut_data.bonus_points,
                        forename: primary.imm_data.forename,
                    })
                }
            }
            pub mod updates {
                pub mod pulpit_access_2 {
                    #[derive(Debug)]
                    pub enum UpdateError {
                        KeyError,
                        sensible_ages,
                    }
                    pub struct Update {
                        pub bonus_points: i32,
                    }
                }
            }
            impl<'imm> Window<'imm> {
                pub fn pulpit_access_2(
                    &mut self,
                    update: updates::pulpit_access_2::Update,
                    key: Key,
                ) -> Result<(), updates::pulpit_access_2::UpdateError> {
                    let emdb::dependencies::pulpit::column::Entry {
                        index,
                        data: primary,
                    } = match self.columns.primary.brw_mut(key) {
                        Ok(entry) => entry,
                        Err(_) => {
                            return Err(updates::pulpit_access_2::UpdateError::KeyError);
                        }
                    };
                    if !predicates::sensible_ages(borrows::Borrows {
                        forename: &primary.imm_data.forename,
                        age: &primary.imm_data.age,
                        surname: &primary.imm_data.surname,
                        bonus_points: &update.bonus_points,
                    }) {
                        return Err(updates::pulpit_access_2::UpdateError::sensible_ages);
                    }
                    let mut update = update;
                    std::mem::swap(
                        &mut primary.mut_data.bonus_points,
                        &mut update.bonus_points,
                    );
                    if !self.transactions.rollback {
                        self.transactions
                            .log
                            .push(
                                transactions::LogItem::Update(
                                    key,
                                    transactions::Updates::pulpit_access_2(update),
                                ),
                            );
                    }
                    Ok(())
                }
            }
            pub mod insert {
                pub struct Insert {
                    pub age: u8,
                    pub surname: String,
                    pub bonus_points: i32,
                    pub forename: String,
                }
                #[derive(Debug)]
                pub enum Error {
                    sensible_ages,
                }
            }
            impl<'imm> Window<'imm> {
                pub fn insert(
                    &mut self,
                    insert_val: insert::Insert,
                ) -> Result<Key, insert::Error> {
                    if !predicates::sensible_ages(borrows::Borrows {
                        age: &insert_val.age,
                        surname: &insert_val.surname,
                        bonus_points: &insert_val.bonus_points,
                        forename: &insert_val.forename,
                    }) {
                        return Err(insert::Error::sensible_ages);
                    }
                    let primary = (emdb::dependencies::pulpit::column::Data {
                        imm_data: column_types::primary::Imm {
                            forename: insert_val.forename,
                            age: insert_val.age,
                            surname: insert_val.surname,
                        },
                        mut_data: column_types::primary::Mut {
                            bonus_points: insert_val.bonus_points,
                        },
                    });
                    let key = self.columns.primary.append(primary);
                    if !self.transactions.rollback {
                        self.transactions.log.push(transactions::LogItem::Append);
                    }
                    Ok(key)
                }
            }
            pub mod unique {
                #[derive(Debug)]
                pub struct NotFound;
            }
            impl<'imm> Window<'imm> {}
            mod transactions {
                pub enum Updates {
                    pulpit_access_2(super::updates::pulpit_access_2::Update),
                }
                pub enum LogItem {
                    Update(super::Key, Updates),
                    Append,
                }
                pub struct Data {
                    pub log: Vec<LogItem>,
                    pub rollback: bool,
                }
                impl Data {
                    pub fn new() -> Self {
                        Self {
                            log: Vec::new(),
                            rollback: false,
                        }
                    }
                }
            }
            impl<'imm> Window<'imm> {
                /// Commit all current changes
                /// - Clears the rollback log
                pub fn commit(&mut self) {
                    debug_assert!(! self.transactions.rollback);
                    self.transactions.log.clear()
                }
                /// Undo the transactions applied since the last commit
                /// - Requires re-applying all updates, deleting inserts and undoing deletes
                ///   (deletes' keys are actually just hidden until commit or abort)
                pub fn abort(&mut self) {
                    self.transactions.rollback = true;
                    while let Some(entry) = self.transactions.log.pop() {
                        match entry {
                            transactions::LogItem::Append => {
                                unsafe {
                                    self.columns.primary.unppend();
                                }
                            }
                            transactions::LogItem::Update(key, update) => {
                                match update {
                                    transactions::Updates::pulpit_access_2(update) => {
                                        self.pulpit_access_2(update, key).unwrap();
                                    }
                                }
                            }
                        }
                    }
                    self.transactions.rollback = false;
                }
            }
            impl<'imm> Window<'imm> {
                pub fn count(&self) -> usize {
                    self.columns.primary.count()
                }
            }
            impl<'db> Window<'db> {
                pub fn scan(&self) -> impl Iterator<Item = Key> + '_ {
                    self.columns.primary.scan()
                }
            }
            /// The key for accessing rows (delete, update, get)
            pub type Key = <emdb::dependencies::pulpit::column::AssocBlocks<
                column_types::primary::Imm,
                column_types::primary::Mut,
                1024usize,
            > as emdb::dependencies::pulpit::column::Keyable>::Key;
            mod predicates {
                pub fn sensible_ages(
                    super::borrows::Borrows {
                        age,
                        surname,
                        bonus_points,
                        forename,
                    }: super::borrows::Borrows,
                ) -> bool {
                    *age < 255
                }
            }
            struct Uniques {}
            impl Uniques {
                fn new(size_hint: usize) -> Self {
                    Self {}
                }
            }
            struct ColumnHolder {
                primary: emdb::dependencies::pulpit::column::AssocBlocks<
                    column_types::primary::Imm,
                    column_types::primary::Mut,
                    1024usize,
                >,
            }
            impl ColumnHolder {
                fn new(size_hint: usize) -> Self {
                    Self {
                        primary: emdb::dependencies::pulpit::column::AssocBlocks::new(
                            size_hint,
                        ),
                    }
                }
                fn window(&mut self) -> WindowHolder<'_> {
                    WindowHolder {
                        primary: self.primary.window(),
                    }
                }
            }
            struct WindowHolder<'imm> {
                primary: <emdb::dependencies::pulpit::column::AssocBlocks<
                    column_types::primary::Imm,
                    column_types::primary::Mut,
                    1024usize,
                > as emdb::dependencies::pulpit::column::Column>::WindowKind<'imm>,
            }
            pub struct Table {
                columns: ColumnHolder,
                uniques: Uniques,
                transactions: transactions::Data,
            }
            impl Table {
                pub fn new(size_hint: usize) -> Self {
                    Self {
                        columns: ColumnHolder::new(size_hint),
                        uniques: Uniques::new(size_hint),
                        transactions: transactions::Data::new(),
                    }
                }
                pub fn window(&mut self) -> Window<'_> {
                    Window {
                        columns: self.columns.window(),
                        uniques: &mut self.uniques,
                        transactions: &mut self.transactions,
                    }
                }
            }
            pub struct Window<'imm> {
                columns: WindowHolder<'imm>,
                uniques: &'imm mut Uniques,
                transactions: &'imm mut transactions::Data,
            }
        }
    }
    pub mod queries {
        pub mod customer_age_brackets {
            #[derive(Debug)]
            pub enum Error {
                Error5(super::super::tables::family_bonus::unique::NotFound),
                Error6,
                Error2(
                    super::super::tables::customers::updates::pulpit_access_2::UpdateError,
                ),
                Error7(
                    super::super::tables::family_bonus::updates::pulpit_access_7::UpdateError,
                ),
                Error1,
            }
        }
    }
    #[derive(Clone)]
    struct RecordTypeAlias0<'db, 'qy> {
        ref_cust: tables::customers::Key,
        __internal_phantomdata: std::marker::PhantomData<(&'db (), &'qy ())>,
    }
    #[derive(Clone)]
    struct RecordTypeAlias1<'db, 'qy> {
        forename: &'db String,
        age: &'db u8,
        surname: &'db String,
        bonus_points: i32,
        __internal_phantomdata: std::marker::PhantomData<(&'db (), &'qy ())>,
    }
    #[derive(Clone)]
    struct RecordTypeAlias2<'db, 'qy> {
        ref_cust: tables::customers::Key,
        person: RecordTypeAlias1<'db, 'qy>,
        __internal_phantomdata: std::marker::PhantomData<(&'db (), &'qy ())>,
    }
    #[derive(Clone)]
    struct RecordTypeAlias3<'db, 'qy> {
        bonus_points: i32,
        __internal_phantomdata: std::marker::PhantomData<(&'db (), &'qy ())>,
    }
    #[derive(Clone)]
    struct RecordTypeAlias4<'db, 'qy> {
        surname: String,
        __internal_phantomdata: std::marker::PhantomData<(&'db (), &'qy ())>,
    }
    #[derive(Clone)]
    struct RecordTypeAlias5<'db, 'qy> {
        surname: String,
        family_ref: tables::family_bonus::Key,
        __internal_phantomdata: std::marker::PhantomData<(&'db (), &'qy ())>,
    }
    #[derive(Clone)]
    struct RecordTypeAlias6<'db, 'qy> {
        surname: &'db String,
        bonus: i32,
        __internal_phantomdata: std::marker::PhantomData<(&'db (), &'qy ())>,
    }
    #[derive(Clone)]
    struct RecordTypeAlias7<'db, 'qy> {
        surname: String,
        family_ref: tables::family_bonus::Key,
        family: RecordTypeAlias6<'db, 'qy>,
        __internal_phantomdata: std::marker::PhantomData<(&'db (), &'qy ())>,
    }
    #[derive(Clone)]
    struct RecordTypeAlias8<'db, 'qy> {
        bonus: i32,
        __internal_phantomdata: std::marker::PhantomData<(&'db (), &'qy ())>,
    }
    #[derive(Clone)]
    struct RecordTypeAlias9<'db, 'qy> {
        __internal_phantomdata: std::marker::PhantomData<(&'db (), &'qy ())>,
    }
    pub struct Datastore {
        customers: tables::customers::Table,
        family_bonus: tables::family_bonus::Table,
    }
    impl Datastore {
        pub fn new() -> Self {
            Self {
                customers: tables::customers::Table::new(1024),
                family_bonus: tables::family_bonus::Table::new(1024),
            }
        }
        pub fn db(&mut self) -> Database<'_> {
            Database {
                customers: self.customers.window(),
                family_bonus: self.family_bonus.window(),
            }
        }
    }
    pub struct Database<'db> {
        customers: tables::customers::Window<'db>,
        family_bonus: tables::family_bonus::Window<'db>,
    }
    impl<'db> Database<'db> {
        pub fn customer_age_brackets<'qy>(
            &'qy mut self,
        ) -> Result<(), queries::customer_age_brackets::Error> {
            match (|| {
                let (
                    operator_closure_value_0,
                    operator_closure_value_1,
                    operator_closure_value_2,
                    operator_closure_value_3,
                ) = (
                    (),
                    (),
                    |RecordTypeAlias2 { ref_cust, person, .. }| {
                        (
                            RecordTypeAlias3 {
                                bonus_points: person.bonus_points + 1,
                                __internal_phantomdata: std::marker::PhantomData,
                            },
                            RecordTypeAlias2 {
                                ref_cust,
                                person,
                                __internal_phantomdata: std::marker::PhantomData,
                            },
                        )
                    },
                    |
                        ref_cust: tables::customers::Key,
                        person: RecordTypeAlias1<'db, 'qy>|
                    {
                        (
                            RecordTypeAlias4 {
                                surname: person.surname.clone(),
                                __internal_phantomdata: std::marker::PhantomData,
                            },
                            (),
                            (),
                            |RecordTypeAlias7 { surname, family_ref, family, .. }| {
                                (
                                    RecordTypeAlias8 {
                                        bonus: family.bonus + 1,
                                        __internal_phantomdata: std::marker::PhantomData,
                                    },
                                    RecordTypeAlias7 {
                                        surname,
                                        family_ref,
                                        family,
                                        __internal_phantomdata: std::marker::PhantomData,
                                    },
                                )
                            },
                            RecordTypeAlias9 {
                                __internal_phantomdata: std::marker::PhantomData,
                            },
                            (),
                        )
                    },
                );
                {
                    let dataflow_value_0: <emdb::dependencies::minister::Basic as emdb::dependencies::minister::Physical>::Stream<
                        RecordTypeAlias0<'db, 'qy>,
                    > = {
                        let stream_values = emdb::dependencies::minister::Basic::consume_stream(
                            self.customers.scan().collect::<Vec<_>>().into_iter(),
                        );
                        emdb::dependencies::minister::Basic::map(
                            stream_values,
                            |value| RecordTypeAlias0 {
                                ref_cust: value,
                                __internal_phantomdata: std::marker::PhantomData,
                            },
                        )
                    };
                    let dataflow_value_1: <emdb::dependencies::minister::Basic as emdb::dependencies::minister::Physical>::Stream<
                        RecordTypeAlias2<'db, 'qy>,
                    > = {
                        let result = emdb::dependencies::minister::Basic::map(
                            dataflow_value_0,
                            |dataflow_value_0| {
                                match self.customers.get(dataflow_value_0.ref_cust) {
                                    Ok(get_value) => {
                                        Ok(RecordTypeAlias2 {
                                            person: RecordTypeAlias1 {
                                                forename: get_value.forename,
                                                age: get_value.age,
                                                surname: get_value.surname,
                                                bonus_points: get_value.bonus_points,
                                                __internal_phantomdata: std::marker::PhantomData,
                                            },
                                            ref_cust: dataflow_value_0.ref_cust,
                                            __internal_phantomdata: std::marker::PhantomData,
                                        })
                                    }
                                    Err(_) => {
                                        return Err(queries::customer_age_brackets::Error::Error1);
                                    }
                                }
                            },
                        );
                        match emdb::dependencies::minister::Basic::error_stream(result) {
                            Ok(val) => val,
                            Err(err) => {
                                return Err(queries::customer_age_brackets::Error::Error1);
                            }
                        }
                    };
                    let dataflow_value_2: <emdb::dependencies::minister::Basic as emdb::dependencies::minister::Physical>::Stream<
                        RecordTypeAlias2<'db, 'qy>,
                    > = {
                        let results = emdb::dependencies::minister::Basic::map_seq(
                            dataflow_value_1,
                            |dataflow_value_1| {
                                let (update_struct, continue_struct) = operator_closure_value_2
                                    .clone()(dataflow_value_1);
                                match self
                                    .customers
                                    .pulpit_access_2(
                                        tables::customers::updates::pulpit_access_2::Update {
                                            bonus_points: update_struct.bonus_points,
                                        },
                                        continue_struct.ref_cust,
                                    )
                                {
                                    Ok(()) => Ok(continue_struct),
                                    Err(err) => {
                                        Err(queries::customer_age_brackets::Error::Error2(err))
                                    }
                                }
                            },
                        );
                        emdb::dependencies::minister::Basic::error_stream(results)?
                    };
                    let dataflow_value_3: <emdb::dependencies::minister::Basic as emdb::dependencies::minister::Physical>::Stream<
                        RecordTypeAlias9<'db, 'qy>,
                    > = {
                        let results = emdb::dependencies::minister::Basic::map_seq(
                            dataflow_value_2,
                            |lifted| {
                                let (
                                    operator_closure_value_4,
                                    operator_closure_value_5,
                                    operator_closure_value_6,
                                    operator_closure_value_7,
                                    operator_closure_value_9,
                                    operator_closure_value_10,
                                ) = (operator_closure_value_3)(
                                    lifted.ref_cust,
                                    lifted.person,
                                );
                                {
                                    let dataflow_value_4: <emdb::dependencies::minister::Basic as emdb::dependencies::minister::Physical>::Single<
                                        RecordTypeAlias4<'db, 'qy>,
                                    > = emdb::dependencies::minister::Basic::consume_single(
                                        operator_closure_value_4,
                                    );
                                    let dataflow_value_5: <emdb::dependencies::minister::Basic as emdb::dependencies::minister::Physical>::Single<
                                        RecordTypeAlias5<'db, 'qy>,
                                    > = {
                                        let result = emdb::dependencies::minister::Basic::map_single(
                                            dataflow_value_4,
                                            |dataflow_value_4| {
                                                let data = self
                                                    .family_bonus
                                                    .unique_surnames_cons(&dataflow_value_4.surname)?;
                                                Ok(RecordTypeAlias5 {
                                                    family_ref: data,
                                                    surname: dataflow_value_4.surname,
                                                    __internal_phantomdata: std::marker::PhantomData,
                                                })
                                            },
                                        );
                                        match emdb::dependencies::minister::Basic::error_single(
                                            result,
                                        ) {
                                            Ok(val) => val,
                                            Err(err) => {
                                                return Err(
                                                    queries::customer_age_brackets::Error::Error5(err),
                                                );
                                            }
                                        }
                                    };
                                    let dataflow_value_6: <emdb::dependencies::minister::Basic as emdb::dependencies::minister::Physical>::Single<
                                        RecordTypeAlias7<'db, 'qy>,
                                    > = {
                                        let result = emdb::dependencies::minister::Basic::map_single(
                                            dataflow_value_5,
                                            |dataflow_value_5| {
                                                match self.family_bonus.get(dataflow_value_5.family_ref) {
                                                    Ok(get_value) => {
                                                        Ok(RecordTypeAlias7 {
                                                            family: RecordTypeAlias6 {
                                                                surname: get_value.surname,
                                                                bonus: get_value.bonus,
                                                                __internal_phantomdata: std::marker::PhantomData,
                                                            },
                                                            surname: dataflow_value_5.surname,
                                                            family_ref: dataflow_value_5.family_ref,
                                                            __internal_phantomdata: std::marker::PhantomData,
                                                        })
                                                    }
                                                    Err(_) => {
                                                        return Err(queries::customer_age_brackets::Error::Error6);
                                                    }
                                                }
                                            },
                                        );
                                        match emdb::dependencies::minister::Basic::error_single(
                                            result,
                                        ) {
                                            Ok(val) => val,
                                            Err(err) => {
                                                return Err(queries::customer_age_brackets::Error::Error6);
                                            }
                                        }
                                    };
                                    let dataflow_value_7: <emdb::dependencies::minister::Basic as emdb::dependencies::minister::Physical>::Single<
                                        RecordTypeAlias7<'db, 'qy>,
                                    > = {
                                        let results = emdb::dependencies::minister::Basic::map_single(
                                            dataflow_value_6,
                                            |dataflow_value_6| {
                                                let (update_struct, continue_struct) = operator_closure_value_7
                                                    .clone()(dataflow_value_6);
                                                match self
                                                    .family_bonus
                                                    .pulpit_access_7(
                                                        tables::family_bonus::updates::pulpit_access_7::Update {
                                                            bonus: update_struct.bonus,
                                                        },
                                                        continue_struct.family_ref,
                                                    )
                                                {
                                                    Ok(()) => Ok(continue_struct),
                                                    Err(err) => {
                                                        Err(queries::customer_age_brackets::Error::Error7(err))
                                                    }
                                                }
                                            },
                                        );
                                        emdb::dependencies::minister::Basic::error_single(results)?
                                    };
                                    let dataflow_value_8: <emdb::dependencies::minister::Basic as emdb::dependencies::minister::Physical>::Single<
                                        RecordTypeAlias9<'db, 'qy>,
                                    > = emdb::dependencies::minister::Basic::consume_single(
                                        operator_closure_value_9,
                                    );
                                    let return_value_10: <emdb::dependencies::minister::Basic as emdb::dependencies::minister::Physical>::Single<
                                        RecordTypeAlias9<'db, 'qy>,
                                    > = dataflow_value_8;
                                    Ok(return_value_10)
                                }
                            },
                        );
                        emdb::dependencies::minister::Basic::error_stream(results)?
                    };
                    Ok(())
                }
            })() {
                Ok(result) => {
                    {
                        self.customers.commit();
                        self.family_bonus.commit();
                    }
                    Ok(result)
                }
                Err(e) => {
                    {
                        self.customers.abort();
                        self.family_bonus.abort();
                    }
                    Err(e)
                }
            }
        }
    }
}
