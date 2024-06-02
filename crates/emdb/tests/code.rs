mod debug_code {
    use emdb::dependencies::minister::Physical;
    pub mod tables {
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
                        pub id: usize,
                        pub surname: String,
                        pub forename: String,
                    }
                    #[derive(Clone)]
                    pub struct Mut {
                        pub age: u8,
                    }
                    pub struct ImmUnpack<'imm> {
                        pub id: &'imm usize,
                        pub surname: &'imm String,
                        pub forename: &'imm String,
                    }
                    pub fn imm_unpack<'imm>(
                        Imm { id, surname, forename }: &'imm Imm,
                    ) -> ImmUnpack<'imm> {
                        ImmUnpack { id, surname, forename }
                    }
                }
            }
            pub mod borrows {
                pub struct Borrows<'brw> {
                    pub surname: &'brw String,
                    pub forename: &'brw String,
                    pub age: &'brw u8,
                    pub id: &'brw usize,
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
                        surname: &primary.imm_data.surname,
                        forename: &primary.imm_data.forename,
                        age: &primary.mut_data.age,
                        id: &primary.imm_data.id,
                    })
                }
            }
            pub mod get {
                pub struct Get<'db> {
                    pub age: u8,
                    pub id: &'db usize,
                    pub surname: &'db String,
                    pub forename: &'db String,
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
                        surname: primary.imm_data.surname,
                        forename: primary.imm_data.forename,
                        age: primary.mut_data.age,
                        id: primary.imm_data.id,
                    })
                }
            }
            pub mod updates {
                pub mod pulpit_access_8 {
                    #[derive(Debug)]
                    pub enum UpdateError {
                        KeyError,
                        sensible_ages,
                    }
                    pub struct Update {
                        pub age: u8,
                    }
                }
            }
            impl<'imm> Window<'imm> {
                pub fn pulpit_access_8(
                    &mut self,
                    update: updates::pulpit_access_8::Update,
                    key: Key,
                ) -> Result<(), updates::pulpit_access_8::UpdateError> {
                    let emdb::dependencies::pulpit::column::Entry {
                        index,
                        data: primary,
                    } = match self.columns.primary.brw_mut(key) {
                        Ok(entry) => entry,
                        Err(_) => {
                            return Err(updates::pulpit_access_8::UpdateError::KeyError);
                        }
                    };
                    if !predicates::sensible_ages(borrows::Borrows {
                        id: &primary.imm_data.id,
                        surname: &primary.imm_data.surname,
                        forename: &primary.imm_data.forename,
                        age: &update.age,
                    }) {
                        return Err(updates::pulpit_access_8::UpdateError::sensible_ages);
                    }
                    let mut update = update;
                    std::mem::swap(&mut primary.mut_data.age, &mut update.age);
                    if !self.transactions.rollback {
                        self.transactions
                            .log
                            .push(
                                transactions::LogItem::Update(
                                    key,
                                    transactions::Updates::pulpit_access_8(update),
                                ),
                            );
                    }
                    Ok(())
                }
            }
            pub mod insert {
                pub struct Insert {
                    pub surname: String,
                    pub forename: String,
                    pub age: u8,
                    pub id: usize,
                }
                #[derive(Debug)]
                pub enum Error {
                    unique_id,
                    sensible_ages,
                }
            }
            impl<'imm> Window<'imm> {
                pub fn insert(
                    &mut self,
                    insert_val: insert::Insert,
                ) -> Result<Key, insert::Error> {
                    if !predicates::sensible_ages(borrows::Borrows {
                        surname: &insert_val.surname,
                        forename: &insert_val.forename,
                        age: &insert_val.age,
                        id: &insert_val.id,
                    }) {
                        return Err(insert::Error::sensible_ages);
                    }
                    let unique_id = match self.uniques.id.lookup(&insert_val.id) {
                        Ok(_) => return Err(insert::Error::unique_id),
                        Err(_) => insert_val.id.clone(),
                    };
                    let primary = (emdb::dependencies::pulpit::column::Data {
                        imm_data: column_types::primary::Imm {
                            id: insert_val.id,
                            surname: insert_val.surname,
                            forename: insert_val.forename,
                        },
                        mut_data: column_types::primary::Mut {
                            age: insert_val.age,
                        },
                    });
                    let (key, action) = self.columns.primary.insert(primary);
                    match action {
                        emdb::dependencies::pulpit::column::InsertAction::Place(
                            index,
                        ) => unsafe {}
                        emdb::dependencies::pulpit::column::InsertAction::Append => {}
                    }
                    self.uniques.id.insert(unique_id, key).unwrap();
                    if !self.transactions.rollback {
                        self.transactions.log.push(transactions::LogItem::Insert(key));
                    }
                    Ok(key)
                }
            }
            pub mod unique {
                #[derive(Debug)]
                pub struct NotFound;
            }
            impl<'imm> Window<'imm> {
                pub fn unique_id(&self, value: &usize) -> Result<Key, unique::NotFound> {
                    match self.uniques.id.lookup(value) {
                        Ok(k) => Ok(k),
                        Err(_) => Err(unique::NotFound),
                    }
                }
            }
            mod transactions {
                pub enum Updates {
                    pulpit_access_8(super::updates::pulpit_access_8::Update),
                }
                pub enum LogItem {
                    Update(super::Key, Updates),
                    Insert(super::Key),
                    Delete(super::Key),
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
                /// - Requires concretely applying deletions (which until commit
                ///   or abort simply hide keys from the table)
                pub fn commit(&mut self) {
                    debug_assert!(! self.transactions.rollback);
                    while let Some(entry) = self.transactions.log.pop() {
                        match entry {
                            transactions::LogItem::Delete(key) => {
                                self.restore_hidden(key);
                            }
                            _ => {}
                        }
                    }
                }
                /// Undo the transactions applied since the last commit
                /// - Requires re-applying all updates, deleting inserts and undoing deletes
                ///   (deletes' keys are actually just hidden until commit or abort)
                pub fn abort(&mut self) {
                    self.transactions.rollback = true;
                    while let Some(entry) = self.transactions.log.pop() {
                        match entry {
                            transactions::LogItem::Delete(key) => {
                                self.delete_hidden(key);
                            }
                            transactions::LogItem::Insert(key) => {
                                self.reverse_insert(key);
                            }
                            transactions::LogItem::Update(key, update) => {
                                match update {
                                    transactions::Updates::pulpit_access_8(update) => {
                                        self.pulpit_access_8(update, key).unwrap();
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
            impl<'imm> Window<'imm> {
                fn reverse_insert(&mut self, key: Key) {
                    debug_assert!(self.transactions.rollback);
                    {
                        match self.columns.primary.pull(key) {
                            Ok(
                                emdb::dependencies::pulpit::column::Entry {
                                    index: index,
                                    data: primary,
                                },
                            ) => {
                                self.uniques.id.pull(&primary.imm_data.id).unwrap();
                                Ok(())
                            }
                            Err(_) => Err(KeyError),
                        }
                    }
                        .unwrap()
                }
                fn delete_hidden(&mut self, key: Key) {
                    debug_assert!(self.transactions.rollback);
                    let emdb::dependencies::pulpit::column::Entry {
                        index: index,
                        data,
                    } = self.columns.primary.pull(key).unwrap();
                    unsafe {}
                }
                fn restore_hidden(&mut self, key: Key) {
                    debug_assert!(self.transactions.rollback);
                    self.columns.primary.reveal(key).unwrap();
                    let brw_data = self.borrow(key).unwrap();
                    self.uniques.id.insert(brw_data.id.clone(), key).unwrap();
                }
                pub fn delete(&mut self, key: Key) -> Result<(), KeyError> {
                    match self.columns.primary.hide(key) {
                        Ok(()) => {}
                        Err(_) => return Err(KeyError),
                    }
                    if !self.transactions.rollback {
                        self.transactions.log.push(transactions::LogItem::Delete(key));
                    }
                    Ok(())
                }
            }
            /// The key for accessing rows (delete, update, get)
            pub type Key = <emdb::dependencies::pulpit::column::PrimaryRetain<
                column_types::primary::Imm,
                column_types::primary::Mut,
                1024usize,
            > as emdb::dependencies::pulpit::column::Keyable>::Key;
            mod predicates {
                pub fn sensible_ages(
                    super::borrows::Borrows {
                        surname,
                        forename,
                        age,
                        id,
                    }: super::borrows::Borrows,
                ) -> bool {
                    *age < 120
                }
            }
            struct Uniques {
                id: emdb::dependencies::pulpit::access::Unique<usize, Key>,
            }
            impl Uniques {
                fn new(size_hint: usize) -> Self {
                    Self {
                        id: emdb::dependencies::pulpit::access::Unique::new(size_hint),
                    }
                }
            }
            struct ColumnHolder {
                primary: emdb::dependencies::pulpit::column::PrimaryRetain<
                    column_types::primary::Imm,
                    column_types::primary::Mut,
                    1024usize,
                >,
            }
            impl ColumnHolder {
                fn new(size_hint: usize) -> Self {
                    Self {
                        primary: emdb::dependencies::pulpit::column::PrimaryRetain::new(
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
                primary: <emdb::dependencies::pulpit::column::PrimaryRetain<
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
        pub mod get_unique_customer {
            #[derive(Debug)]
            pub enum Error {
                Error1(super::super::tables::customers::unique::NotFound),
            }
        }
        mod drop_all {}
        pub mod update_name {
            #[derive(Debug)]
            pub enum Error {
                Error8(
                    super::super::tables::customers::updates::pulpit_access_8::UpdateError,
                ),
                Error7,
            }
        }
        pub mod insert_name {
            #[derive(Debug)]
            pub enum Error {
                Error11(super::super::tables::customers::insert::Error),
            }
        }
    }
    struct RecordTypeAlias0<'db, 'qy> {
        name: &'qy str,
        ident: usize,
        __internal_phantomdata: std::marker::PhantomData<(&'db (), &'qy ())>,
    }
    struct RecordTypeAlias1<'db, 'qy> {
        name: &'qy str,
        cust_ref: tables::customers::Key,
        ident: usize,
        __internal_phantomdata: std::marker::PhantomData<(&'db (), &'qy ())>,
    }
    struct RecordTypeAlias2<'db, 'qy> {
        cust: tables::customers::Key,
        __internal_phantomdata: std::marker::PhantomData<(&'db (), &'qy ())>,
    }
    struct RecordTypeAlias3<'db, 'qy> {
        cust_key: tables::customers::Key,
        __internal_phantomdata: std::marker::PhantomData<(&'db (), &'qy ())>,
    }
    struct RecordTypeAlias4<'db, 'qy> {
        surname: &'db String,
        id: &'db usize,
        forename: &'db String,
        age: u8,
        __internal_phantomdata: std::marker::PhantomData<(&'db (), &'qy ())>,
    }
    struct RecordTypeAlias5<'db, 'qy> {
        cust_key: tables::customers::Key,
        cust_data: RecordTypeAlias4<'db, 'qy>,
        __internal_phantomdata: std::marker::PhantomData<(&'db (), &'qy ())>,
    }
    struct RecordTypeAlias6<'db, 'qy> {
        age: u8,
        __internal_phantomdata: std::marker::PhantomData<(&'db (), &'qy ())>,
    }
    struct RecordTypeAlias7<'db, 'qy> {
        id: usize,
        age: u8,
        surname: String,
        forename: String,
        __internal_phantomdata: std::marker::PhantomData<(&'db (), &'qy ())>,
    }
    struct RecordTypeAlias8<'db, 'qy> {
        forename: String,
        id: usize,
        surname: String,
        age: u8,
        __internal_phantomdata: std::marker::PhantomData<(&'db (), &'qy ())>,
    }
    struct RecordTypeAlias9<'db, 'qy> {
        cust_ref: tables::customers::Key,
        __internal_phantomdata: std::marker::PhantomData<(&'db (), &'qy ())>,
    }
    pub struct Datastore {
        customers: tables::customers::Table,
    }
    impl Datastore {
        pub fn new() -> Self {
            Self {
                customers: tables::customers::Table::new(1024),
            }
        }
        pub fn db(&mut self) -> Database<'_> {
            Database {
                customers: self.customers.window(),
            }
        }
    }
    pub struct Database<'db> {
        customers: tables::customers::Window<'db>,
    }
    impl<'db> Database<'db> {
        pub fn get_unique_customer<'qy>(
            &'qy self,
            name: &'qy str,
            id: usize,
        ) -> Result<(), queries::get_unique_customer::Error> {
            let (operator_closure_value_0, operator_closure_value_1) = (
                RecordTypeAlias0 {
                    ident: id,
                    name: name,
                    __internal_phantomdata: std::marker::PhantomData,
                },
                (),
            );
            Ok({
                let dataflow_value_0: <emdb::dependencies::minister::Basic as emdb::dependencies::minister::Physical>::Single<
                    RecordTypeAlias0<'db, 'qy>,
                > = emdb::dependencies::minister::Basic::consume_single(
                    operator_closure_value_0,
                );
                let dataflow_value_1: <emdb::dependencies::minister::Basic as emdb::dependencies::minister::Physical>::Single<
                    RecordTypeAlias1<'db, 'qy>,
                > = {
                    let result = emdb::dependencies::minister::Basic::map_single(
                        dataflow_value_0,
                        |dataflow_value_0| {
                            let data = self
                                .customers
                                .unique_id(&dataflow_value_0.ident)?;
                            Ok(RecordTypeAlias1 {
                                cust_ref: data,
                                name: dataflow_value_0.name,
                                ident: dataflow_value_0.ident,
                                __internal_phantomdata: std::marker::PhantomData,
                            })
                        },
                    );
                    match emdb::dependencies::minister::Basic::error_single(result) {
                        Ok(val) => val,
                        Err(err) => {
                            return Err(queries::get_unique_customer::Error::Error1(err));
                        }
                    }
                };
                ()
            })
        }
        pub fn drop_all<'qy>(&'qy self) -> () {
            let (operator_closure_value_3, operator_closure_value_4) = ((), ());
            {
                let dataflow_value_2: <emdb::dependencies::minister::Basic as emdb::dependencies::minister::Physical>::Stream<
                    RecordTypeAlias2<'db, 'qy>,
                > = {
                    let stream_values = emdb::dependencies::minister::Basic::consume_stream(
                        self.customers.scan().collect::<Vec<_>>().into_iter(),
                    );
                    emdb::dependencies::minister::Basic::map(
                        stream_values,
                        |value| RecordTypeAlias2 {
                            cust: value,
                            __internal_phantomdata: std::marker::PhantomData,
                        },
                    )
                };
                let _ = ();
                ()
            }
        }
        pub fn update_name<'qy>(
            &'qy mut self,
        ) -> Result<(), queries::update_name::Error> {
            match (|| {
                let (
                    operator_closure_value_6,
                    operator_closure_value_7,
                    operator_closure_value_8,
                ) = (
                    (),
                    (),
                    |RecordTypeAlias5 { cust_key, cust_data, .. }| {
                        (
                            RecordTypeAlias6 {
                                age: cust_data.age + 1,
                                __internal_phantomdata: std::marker::PhantomData,
                            },
                            RecordTypeAlias5 {
                                cust_key,
                                cust_data,
                                __internal_phantomdata: std::marker::PhantomData,
                            },
                        )
                    },
                );
                Ok({
                    let dataflow_value_4: <emdb::dependencies::minister::Basic as emdb::dependencies::minister::Physical>::Stream<
                        RecordTypeAlias3<'db, 'qy>,
                    > = {
                        let stream_values = emdb::dependencies::minister::Basic::consume_stream(
                            self.customers.scan().collect::<Vec<_>>().into_iter(),
                        );
                        emdb::dependencies::minister::Basic::map(
                            stream_values,
                            |value| RecordTypeAlias3 {
                                cust_key: value,
                                __internal_phantomdata: std::marker::PhantomData,
                            },
                        )
                    };
                    let dataflow_value_5: <emdb::dependencies::minister::Basic as emdb::dependencies::minister::Physical>::Stream<
                        RecordTypeAlias5<'db, 'qy>,
                    > = {
                        let result = emdb::dependencies::minister::Basic::map(
                            dataflow_value_4,
                            |dataflow_value_4| {
                                match self.customers.get(dataflow_value_4.cust_key) {
                                    Ok(get_value) => {
                                        Ok(RecordTypeAlias5 {
                                            cust_data: RecordTypeAlias4 {
                                                surname: get_value.surname,
                                                id: get_value.id,
                                                forename: get_value.forename,
                                                age: get_value.age,
                                                __internal_phantomdata: std::marker::PhantomData,
                                            },
                                            cust_key: dataflow_value_4.cust_key,
                                            __internal_phantomdata: std::marker::PhantomData,
                                        })
                                    }
                                    Err(_) => return Err(queries::update_name::Error::Error7),
                                }
                            },
                        );
                        match emdb::dependencies::minister::Basic::error_stream(result) {
                            Ok(val) => val,
                            Err(err) => return Err(queries::update_name::Error::Error7),
                        }
                    };
                    let dataflow_value_6: <emdb::dependencies::minister::Basic as emdb::dependencies::minister::Physical>::Stream<
                        RecordTypeAlias5<'db, 'qy>,
                    > = {
                        let results = emdb::dependencies::minister::Basic::map_seq(
                            dataflow_value_5,
                            |dataflow_value_5| {
                                let (update_struct, continue_struct) = operator_closure_value_8(
                                    dataflow_value_5,
                                );
                                match self
                                    .customers
                                    .pulpit_access_8(
                                        tables::customers::updates::pulpit_access_8::Update {
                                            age: update_struct.age,
                                        },
                                        continue_struct.cust_key,
                                    )
                                {
                                    Ok(()) => Ok(continue_struct),
                                    Err(err) => Err(queries::update_name::Error::Error8(err)),
                                }
                            },
                        );
                        emdb::dependencies::minister::Basic::error_stream(results)?
                    };
                    ()
                })
            })() {
                Ok(result) => {
                    {
                        self.customers.commit();
                    }
                    Ok(result)
                }
                Err(e) => {
                    {
                        self.customers.abort();
                    }
                    Err(e)
                }
            }
        }
        pub fn insert_name<'qy>(
            &'qy mut self,
            fname: String,
            age: u8,
            id: usize,
        ) -> Result<(), queries::insert_name::Error> {
            match (|| {
                let (operator_closure_value_10, operator_closure_value_11) = (
                    RecordTypeAlias7 {
                        id: id,
                        age: age,
                        forename: fname,
                        surname: String::from("Smith"),
                        __internal_phantomdata: std::marker::PhantomData,
                    },
                    (),
                );
                Ok({
                    let dataflow_value_7: <emdb::dependencies::minister::Basic as emdb::dependencies::minister::Physical>::Single<
                        RecordTypeAlias7<'db, 'qy>,
                    > = emdb::dependencies::minister::Basic::consume_single(
                        operator_closure_value_10,
                    );
                    let dataflow_value_8: <emdb::dependencies::minister::Basic as emdb::dependencies::minister::Physical>::Single<
                        RecordTypeAlias9<'db, 'qy>,
                    > = {
                        let result = emdb::dependencies::minister::Basic::map_single(
                            dataflow_value_7,
                            |dataflow_value_7| {
                                Ok(RecordTypeAlias9 {
                                    cust_ref: self
                                        .customers
                                        .insert(tables::customers::insert::Insert {
                                            id: dataflow_value_7.id,
                                            age: dataflow_value_7.age,
                                            surname: dataflow_value_7.surname,
                                            forename: dataflow_value_7.forename,
                                        })?,
                                    __internal_phantomdata: std::marker::PhantomData,
                                })
                            },
                        );
                        match emdb::dependencies::minister::Basic::error_single(result) {
                            Ok(val) => val,
                            Err(err) => {
                                return Err(queries::insert_name::Error::Error11(err));
                            }
                        }
                    };
                    ()
                })
            })() {
                Ok(result) => {
                    {
                        self.customers.commit();
                    }
                    Ok(result)
                }
                Err(e) => {
                    {
                        self.customers.abort();
                    }
                    Err(e)
                }
            }
        }
    }
}

#[test]
fn foo() {
    let mut ds = debug_code::Datastore::new();
    let mut db = ds.db();
    db.insert_name("John".to_string(), 1, 20).unwrap();
    // db.get_unique_customer("John", 19).unwrap();
    // db.insert_name("John".to_string(), 1, 20).unwrap();
    // db.insert_name("John".to_string(), 1, 20).unwrap();
}