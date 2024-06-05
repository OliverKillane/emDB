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
                        pub bar: (&'static str, bool),
                        pub foo: String,
                        pub bing: usize,
                    }
                    #[derive(Clone)]
                    pub struct Mut {}
                    pub struct ImmUnpack<'imm> {
                        pub bar: &'imm (&'static str, bool),
                        pub foo: &'imm String,
                        pub bing: &'imm usize,
                    }
                    pub fn imm_unpack<'imm>(
                        Imm { bar, foo, bing }: &'imm Imm,
                    ) -> ImmUnpack<'imm> {
                        ImmUnpack { bar, foo, bing }
                    }
                }
            }
            pub mod borrows {
                pub struct Borrows<'brw> {
                    pub bing: &'brw usize,
                    pub foo: &'brw String,
                    pub bar: &'brw (&'static str, bool),
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
                        bing: &primary.imm_data.bing,
                        foo: &primary.imm_data.foo,
                        bar: &primary.imm_data.bar,
                    })
                }
            }
            pub mod get {
                pub struct Get<'db> {
                    pub bar: &'db (&'static str, bool),
                    pub foo: &'db String,
                    pub bing: &'db usize,
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
                        bing: primary.imm_data.bing,
                        foo: primary.imm_data.foo,
                        bar: primary.imm_data.bar,
                    })
                }
            }
            pub mod updates {}
            impl<'imm> Window<'imm> {}
            pub mod insert {
                pub struct Insert {
                    pub bing: usize,
                    pub foo: String,
                    pub bar: (&'static str, bool),
                }
            }
            impl<'imm> Window<'imm> {
                pub fn insert(&mut self, insert_val: insert::Insert) -> Key {
                    let primary = (emdb::dependencies::pulpit::column::Data {
                        imm_data: column_types::primary::Imm {
                            bar: insert_val.bar,
                            foo: insert_val.foo,
                            bing: insert_val.bing,
                        },
                        mut_data: column_types::primary::Mut {},
                    });
                    let key = self.columns.primary.append(primary);
                    if !self.transactions.rollback {
                        self.transactions.log.push(transactions::LogItem::Append);
                    }
                    key
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
        mod new_data {}
        mod all_bings {}
    }
    #[derive(Clone)]
    struct Record0<'db, 'qy> {
        foo: String,
        bing: usize,
        bar: (&'static str, bool),
        __internal_phantomdata: std::marker::PhantomData<(&'db (), &'qy ())>,
    }
    #[derive(Clone)]
    struct Record1<'db, 'qy> {
        bar: (&'static str, bool),
        foo: String,
        bing: usize,
        __internal_phantomdata: std::marker::PhantomData<(&'db (), &'qy ())>,
    }
    #[derive(Clone)]
    pub struct Record2<'db, 'qy> {
        pub new_key: tables::data::Key,
        __internal_phantomdata: std::marker::PhantomData<(&'db (), &'qy ())>,
    }
    #[derive(Clone)]
    struct Record3<'db, 'qy> {
        bar: &'db (&'static str, bool),
        bing: &'db usize,
        foo: &'db String,
        __internal_phantomdata: std::marker::PhantomData<(&'db (), &'qy ())>,
    }
    #[derive(Clone)]
    struct Record4<'db, 'qy> {
        __internal_0: tables::data::Key,
        __internal_phantomdata: std::marker::PhantomData<(&'db (), &'qy ())>,
    }
    #[derive(Clone)]
    struct Record5<'db, 'qy> {
        __internal_1: Record3<'db, 'qy>,
        __internal_0: tables::data::Key,
        __internal_phantomdata: std::marker::PhantomData<(&'db (), &'qy ())>,
    }
    #[derive(Clone)]
    pub struct Record6<'db, 'qy> {
        pub bing_val: usize,
        __internal_phantomdata: std::marker::PhantomData<(&'db (), &'qy ())>,
    }
    #[derive(Clone)]
    pub struct Record7<'db, 'qy> {
        pub values: Vec<Record6<'db, 'qy>>,
        __internal_phantomdata: std::marker::PhantomData<(&'db (), &'qy ())>,
    }
    pub struct Datastore {
        data: tables::data::Table,
    }
    impl Datastore {
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
        pub fn new_data<'qy>(
            &'qy mut self,
            foo: &str,
            bing: usize,
            bar_0: bool,
        ) -> Record2 {
            let (
                operator_closure_value_0,
                operator_closure_value_1,
                operator_closure_value_2,
            ) = (
                Record0 {
                    foo: String::from(foo),
                    bing: bing,
                    bar: (if bar_0 { "bar" } else { "baz" }, bar_0),
                    __internal_phantomdata: std::marker::PhantomData,
                },
                (),
                (),
            );
            let result = {
                let dataflow_value_0: <emdb::dependencies::minister::Basic as emdb::dependencies::minister::Physical>::Single<
                    Record0<'db, 'qy>,
                > = emdb::dependencies::minister::Basic::consume_single(
                    operator_closure_value_0,
                );
                let dataflow_value_1: <emdb::dependencies::minister::Basic as emdb::dependencies::minister::Physical>::Single<
                    Record2<'db, 'qy>,
                > = emdb::dependencies::minister::Basic::map_single(
                    dataflow_value_0,
                    |dataflow_value_0| {
                        Record2 {
                            new_key: self
                                .data
                                .insert(tables::data::insert::Insert {
                                    foo: dataflow_value_0.foo,
                                    bing: dataflow_value_0.bing,
                                    bar: dataflow_value_0.bar,
                                }),
                            __internal_phantomdata: std::marker::PhantomData,
                        }
                    },
                );
                let return_value_2: <emdb::dependencies::minister::Basic as emdb::dependencies::minister::Physical>::Single<
                    Record2<'db, 'qy>,
                > = dataflow_value_1;
                return_value_2
            };
            {
                self.data.commit();
            }
            result
        }
        pub fn all_bings<'qy>(&'qy self) -> Record7 {
            let (
                operator_closure_value_3,
                operator_closure_value_4,
                operator_closure_value_5,
                operator_closure_value_6,
                operator_closure_value_7,
                operator_closure_value_8,
            ) = (
                (),
                (),
                (),
                |Record3 { bar: bar, bing: bing, foo: foo, __internal_phantomdata: _ }| {
                    Record6 {
                        bing_val: *bing,
                        __internal_phantomdata: std::marker::PhantomData,
                    }
                },
                (),
                (),
            );
            {
                let dataflow_value_2: <emdb::dependencies::minister::Basic as emdb::dependencies::minister::Physical>::Stream<
                    Record4<'db, 'qy>,
                > = {
                    let stream_values = emdb::dependencies::minister::Basic::consume_stream(
                        self.data.scan().collect::<Vec<_>>().into_iter(),
                    );
                    emdb::dependencies::minister::Basic::map(
                        stream_values,
                        |value| Record4 {
                            __internal_0: value,
                            __internal_phantomdata: std::marker::PhantomData,
                        },
                    )
                };
                let dataflow_value_3: <emdb::dependencies::minister::Basic as emdb::dependencies::minister::Physical>::Stream<
                    Record5<'db, 'qy>,
                > = {
                    emdb::dependencies::minister::Basic::map(
                        dataflow_value_2,
                        |dataflow_value_2| {
                            match self.data.get(dataflow_value_2.__internal_0) {
                                Ok(get_value) => {
                                    Record5 {
                                        __internal_1: Record3 {
                                            bar: get_value.bar,
                                            bing: get_value.bing,
                                            foo: get_value.foo,
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
                    Record3<'db, 'qy>,
                > = emdb::dependencies::minister::Basic::map(
                    dataflow_value_3,
                    |dataflow_value_3| dataflow_value_3.__internal_1,
                );
                let dataflow_value_5: <emdb::dependencies::minister::Basic as emdb::dependencies::minister::Physical>::Stream<
                    Record6<'db, 'qy>,
                > = emdb::dependencies::minister::Basic::map(
                    dataflow_value_4,
                    operator_closure_value_6,
                );
                let dataflow_value_6: <emdb::dependencies::minister::Basic as emdb::dependencies::minister::Physical>::Single<
                    Record7<'db, 'qy>,
                > = emdb::dependencies::minister::Basic::consume_single(Record7 {
                    values: emdb::dependencies::minister::Basic::export_stream(
                            dataflow_value_5,
                        )
                        .collect::<Vec<_>>(),
                    __internal_phantomdata: std::marker::PhantomData,
                });
                let return_value_8: <emdb::dependencies::minister::Basic as emdb::dependencies::minister::Physical>::Single<
                    Record7<'db, 'qy>,
                > = dataflow_value_6;
                return_value_8
            }
        }
    }
}
