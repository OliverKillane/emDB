mod debug_code {
    mod tables {
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
                        pub age: u8,
                        pub surname: String,
                        pub forename: String,
                    }
                    #[derive(Clone)]
                    pub struct Mut {}
                    pub struct ImmUnpack<'imm> {
                        pub age: &'imm u8,
                        pub surname: &'imm String,
                        pub forename: &'imm String,
                    }
                    pub fn imm_unpack<'imm>(
                        Imm { age, surname, forename }: &'imm Imm,
                    ) -> ImmUnpack<'imm> {
                        ImmUnpack {
                            age,
                            surname,
                            forename,
                        }
                    }
                }
            }
            pub mod borrows {
                pub struct Borrows<'brw> {
                    pub surname: &'brw String,
                    pub age: &'brw u8,
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
                        surname: &primary.imm_data.surname,
                        age: &primary.imm_data.age,
                        forename: &primary.imm_data.forename,
                    })
                }
            }
            pub mod get {
                pub struct Get<'db> {
                    pub age: &'db u8,
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
                        age: primary.imm_data.age,
                        forename: primary.imm_data.forename,
                    })
                }
            }
            pub mod updates {}
            impl<'imm> Window<'imm> {}
            pub mod insert {
                pub struct Insert {
                    pub surname: String,
                    pub age: u8,
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
                        surname: &insert_val.surname,
                        age: &insert_val.age,
                        forename: &insert_val.forename,
                    }) {
                        return Err(insert::Error::sensible_ages);
                    }
                    let primary = (emdb::dependencies::pulpit::column::Data {
                        imm_data: column_types::primary::Imm {
                            age: insert_val.age,
                            surname: insert_val.surname,
                            forename: insert_val.forename,
                        },
                        mut_data: column_types::primary::Mut {},
                    });
                    let (key, action) = self.columns.primary.insert(primary);
                    match action {
                        emdb::dependencies::pulpit::column::InsertAction::Place(
                            index,
                        ) => unsafe {}
                        emdb::dependencies::pulpit::column::InsertAction::Append => {}
                    }
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
            impl<'imm> Window<'imm> {}
            mod transactions {
                pub enum Updates {}
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
                            transactions::LogItem::Update(key, update) => match update {}
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
                            ) => Ok(()),
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
                        age,
                        forename,
                    }: super::borrows::Borrows,
                ) -> bool {
                    *age < 120
                }
            }
            struct Uniques {}
            impl Uniques {
                fn new(size_hint: usize) -> Self {
                    Self {}
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
        mod customer_age_brackets {}
        mod foo {}
    }
    struct RecordTypeAlias0<'db, 'qy> {
        forename: &'db String,
        surname: &'db String,
        age: &'db u8,
        __internal_phantomdata: std::marker::PhantomData<(&'db (), &'qy ())>,
    }
    struct RecordTypeAlias1<'db, 'qy> {
        __internal_0: tables::customers::Key,
        __internal_phantomdata: std::marker::PhantomData<(&'db (), &'qy ())>,
    }
    struct RecordTypeAlias2<'db, 'qy> {
        __internal_1: RecordTypeAlias0<'db, 'qy>,
        __internal_phantomdata: std::marker::PhantomData<(&'db (), &'qy ())>,
    }
    pub struct RecordTypeAlias3<'db, 'qy> {
        pub forename: &'db String,
        pub surname: &'db String,
        __internal_phantomdata: std::marker::PhantomData<(&'db (), &'qy ())>,
    }
    struct RecordTypeAlias4<'db, 'qy> {
        people: Vec<RecordTypeAlias3<'db, 'qy>>,
        __internal_phantomdata: std::marker::PhantomData<(&'db (), &'qy ())>,
    }
    pub struct RecordTypeAlias5<'db, 'qy> {
        pub group: Vec<RecordTypeAlias3<'db, 'qy>>,
        pub age_bracket: u8,
        __internal_phantomdata: std::marker::PhantomData<(&'db (), &'qy ())>,
    }
    pub struct RecordTypeAlias6<'db, 'qy> {
        pub brackets: Vec<RecordTypeAlias5<'db, 'qy>>,
        __internal_phantomdata: std::marker::PhantomData<(&'db (), &'qy ())>,
    }
    struct RecordTypeAlias7<'db, 'qy> {
        key: tables::customers::Key,
        __internal_phantomdata: std::marker::PhantomData<(&'db (), &'qy ())>,
    }
    pub struct Database {
        customers: tables::customers::Table,
    }
    impl Database {
        pub fn new() -> Self {
            Self {
                customers: tables::customers::Table::new(1024),
            }
        }
        pub fn db(&mut self) -> Window<'_> {
            Window {
                customers: self.customers.window(),
            }
        }
    }
    pub struct Window<'db> {
        customers: tables::customers::Window<'db>,
    }
    impl<'db> Window<'db> {
        pub fn customer_age_brackets<'qy>(&'qy self) -> RecordTypeAlias6 {
            let (
                operator_closure_value_0,
                operator_closure_value_1,
                operator_closure_value_2,
                operator_closure_value_3,
                operator_closure_value_7,
                operator_closure_value_8,
                operator_closure_value_9,
            ) = (todo!(), todo!(), todo!(), todo!(), todo!(), todo!(), todo!());
            {
                let return_value_0 = todo!();
                let return_value_1 = todo!();
                let return_value_2 = todo!();
                let return_value_3 = todo!();
                let return_value_7 = todo!();
                let return_value_8 = todo!();
                let return_value_9 = todo!();
                return_value_9
            }
        }
        pub fn foo<'qy>(&'qy self, k: tables::customers::Key) -> () {
            let (operator_closure_value_10, operator_closure_value_11) = (
                todo!(),
                todo!(),
            );
            {
                let return_value_10 = todo!();
                let return_value_11 = todo!();
                ()
            }
        }
    }
}
