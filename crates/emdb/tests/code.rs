mod my_db {
    #![allow(non_shorthand_field_patterns)]
    use emdb::dependencies::minister::Physical;
    pub mod tables {
        pub mod data {
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
                        pub value: i32,
                    }
                    #[derive(Clone)]
                    pub struct Mut {}
                    pub struct ImmUnpack<'imm> {
                        pub value: &'imm i32,
                    }
                    pub fn imm_unpack<'imm>(
                        Imm { value }: &'imm Imm,
                    ) -> ImmUnpack<'imm> {
                        ImmUnpack { value }
                    }
                }
            }
            pub mod borrows {
                pub struct Borrows<'brw> {
                    pub value: &'brw i32,
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
                        value: &primary.imm_data.value,
                    })
                }
            }
            pub mod get {
                pub struct Get<'db> {
                    pub value: &'db i32,
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
                        value: primary.imm_data.value,
                    })
                }
            }
            pub mod updates {}
            impl<'imm> Window<'imm> {}
            pub mod insert {
                pub struct Insert {
                    pub value: i32,
                }
                #[derive(Debug)]
                pub enum Error {
                    unique_values,
                }
            }
            impl<'imm> Window<'imm> {
                pub fn insert(
                    &mut self,
                    insert_val: insert::Insert,
                ) -> Result<Key, insert::Error> {
                    let unique_values = match self
                        .uniques
                        .value
                        .lookup(&insert_val.value)
                    {
                        Ok(_) => return Err(insert::Error::unique_values),
                        Err(_) => insert_val.value.clone(),
                    };
                    let primary = (emdb::dependencies::pulpit::column::Data {
                        imm_data: column_types::primary::Imm {
                            value: insert_val.value,
                        },
                        mut_data: column_types::primary::Mut {},
                    });
                    let key = self.columns.primary.append(primary);
                    self.uniques.value.insert(unique_values, key).unwrap();
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
                pub fn unique_values(
                    &self,
                    value: &i32,
                ) -> Result<Key, unique::NotFound> {
                    match self.uniques.value.lookup(value) {
                        Ok(k) => Ok(k),
                        Err(_) => Err(unique::NotFound),
                    }
                }
            }
            mod transactions {
                pub enum Updates {}
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
            /// The key for accessing rows (delete, update, get)
            pub type Key = <emdb::dependencies::pulpit::column::AssocBlocks<
                column_types::primary::Imm,
                column_types::primary::Mut,
                1024usize,
            > as emdb::dependencies::pulpit::column::Keyable>::Key;
            mod predicates {}
            struct Uniques {
                value: emdb::dependencies::pulpit::access::Unique<i32, Key>,
            }
            impl Uniques {
                fn new(size_hint: usize) -> Self {
                    Self {
                        value: emdb::dependencies::pulpit::access::Unique::new(size_hint),
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
    }
    pub mod queries {
        mod filter_values {}
    }
    #[derive(Clone)]
    struct Record0<'db, 'qy> {
        other_math: i32,
        __internal_phantomdata: std::marker::PhantomData<(&'db (), &'qy ())>,
    }
    #[derive(Clone)]
    struct Record1<'db, 'qy> {
        value: &'db i32,
        __internal_phantomdata: std::marker::PhantomData<(&'db (), &'qy ())>,
    }
    #[derive(Clone)]
    struct Record2<'db, 'qy> {
        __internal_0: tables::data::Key,
        __internal_phantomdata: std::marker::PhantomData<(&'db (), &'qy ())>,
    }
    #[derive(Clone)]
    struct Record3<'db, 'qy> {
        __internal_0: tables::data::Key,
        __internal_1: Record1<'db, 'qy>,
        __internal_phantomdata: std::marker::PhantomData<(&'db (), &'qy ())>,
    }
    #[derive(Clone)]
    struct Record4<'db, 'qy> {
        filtered: Vec<Record1<'db, 'qy>>,
        __internal_phantomdata: std::marker::PhantomData<(&'db (), &'qy ())>,
    }
    pub struct EmDBDebug {
        data: tables::data::Table,
    }
    impl EmDBDebug {
        pub fn new() -> Self {
            Self {
                data: tables::data::Table::new(1024),
            }
        }
        pub fn db(&mut self) -> Database<'_> {
            Database {
                data: self.data.window(),
            }
        }
    }
    pub struct Database<'db> {
        data: tables::data::Window<'db>,
    }
    impl<'db> Database<'db> {
        pub fn filter_values<'qy>(&'qy self, math: i32) -> () {
            let (operator_closure_value_0, operator_closure_value_1) = (
                Record0 {
                    other_math: 7,
                    __internal_phantomdata: std::marker::PhantomData,
                },
                |other_math: i32| {
                    (
                        (),
                        (),
                        (),
                        |
                            Record1 {
                                value: value,
                                __internal_phantomdata: _,
                            }: &Record1<'db, 'qy>,
                        | -> bool { **value > other_math },
                        (),
                        (),
                    )
                },
            );
            {
                let dataflow_value_0: <emdb::dependencies::minister::Basic as emdb::dependencies::minister::Physical>::Single<
                    Record0<'db, 'qy>,
                > = emdb::dependencies::minister::Basic::consume_single(
                    operator_closure_value_0,
                );
                let dataflow_value_1: <emdb::dependencies::minister::Basic as emdb::dependencies::minister::Physical>::Single<
                    Record4<'db, 'qy>,
                > = {
                    let results = emdb::dependencies::minister::Basic::map_single(
                        dataflow_value_0,
                        |lifted| {
                            let (
                                operator_closure_value_2,
                                operator_closure_value_3,
                                operator_closure_value_4,
                                operator_closure_value_5,
                                operator_closure_value_6,
                                operator_closure_value_7,
                            ) = (operator_closure_value_1)(lifted.other_math);
                            {
                                let dataflow_value_2: <emdb::dependencies::minister::Basic as emdb::dependencies::minister::Physical>::Stream<
                                    Record2<'db, 'qy>,
                                > = {
                                    let stream_values = emdb::dependencies::minister::Basic::consume_stream(
                                        self.data.scan().collect::<Vec<_>>().into_iter(),
                                    );
                                    emdb::dependencies::minister::Basic::map(
                                        stream_values,
                                        |value| Record2 {
                                            __internal_0: value,
                                            __internal_phantomdata: std::marker::PhantomData,
                                        },
                                    )
                                };
                                let dataflow_value_3: <emdb::dependencies::minister::Basic as emdb::dependencies::minister::Physical>::Stream<
                                    Record3<'db, 'qy>,
                                > = {
                                    emdb::dependencies::minister::Basic::map(
                                        dataflow_value_2,
                                        |dataflow_value_2| {
                                            match self.data.get(dataflow_value_2.__internal_0) {
                                                Ok(get_value) => {
                                                    Record3 {
                                                        __internal_1: Record1 {
                                                            value: get_value.value,
                                                            __internal_phantomdata: std::marker::PhantomData,
                                                        },
                                                        __internal_0: dataflow_value_2.__internal_0,
                                                        __internal_phantomdata: std::marker::PhantomData,
                                                    }
                                                }
                                                Err(_) => {
                                                    unreachable!(
                                                        "This is an unchecked dereference (used internally - e.g. generated by a use"
                                                    )
                                                }
                                            }
                                        },
                                    )
                                };
                                let dataflow_value_4: <emdb::dependencies::minister::Basic as emdb::dependencies::minister::Physical>::Stream<
                                    Record1<'db, 'qy>,
                                > = emdb::dependencies::minister::Basic::map(
                                    dataflow_value_3,
                                    |dataflow_value_3| dataflow_value_3.__internal_1,
                                );
                                let dataflow_value_5: <emdb::dependencies::minister::Basic as emdb::dependencies::minister::Physical>::Stream<
                                    Record1<'db, 'qy>,
                                > = emdb::dependencies::minister::Basic::filter(
                                    dataflow_value_4,
                                    operator_closure_value_5,
                                );
                                let dataflow_value_6: <emdb::dependencies::minister::Basic as emdb::dependencies::minister::Physical>::Single<
                                    Record4<'db, 'qy>,
                                > = emdb::dependencies::minister::Basic::consume_single(Record4 {
                                    filtered: emdb::dependencies::minister::Basic::export_stream(
                                            dataflow_value_5,
                                        )
                                        .collect::<Vec<_>>(),
                                    __internal_phantomdata: std::marker::PhantomData,
                                });
                                let return_value_7: <emdb::dependencies::minister::Basic as emdb::dependencies::minister::Physical>::Single<
                                    Record4<'db, 'qy>,
                                > = dataflow_value_6;
                                return_value_7
                            }
                        },
                    );
                    results
                };
                ()
            }
        }
    }
}
