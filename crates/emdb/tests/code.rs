mod my_db {
    use emdb::dependencies::minister::Physical;
    pub mod tables {
        pub mod users {
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
                        pub name: String,
                        pub premium: bool,
                        pub credits: i32,
                    }
                    #[derive(Clone)]
                    pub struct Mut {}
                    pub struct ImmUnpack<'imm> {
                        pub name: &'imm String,
                        pub premium: &'imm bool,
                        pub credits: &'imm i32,
                    }
                    pub fn imm_unpack<'imm>(
                        Imm { name, premium, credits }: &'imm Imm,
                    ) -> ImmUnpack<'imm> {
                        ImmUnpack {
                            name,
                            premium,
                            credits,
                        }
                    }
                }
            }
            pub mod borrows {
                pub struct Borrows<'brw> {
                    pub credits: &'brw i32,
                    pub premium: &'brw bool,
                    pub name: &'brw String,
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
                        credits: &primary.imm_data.credits,
                        premium: &primary.imm_data.premium,
                        name: &primary.imm_data.name,
                    })
                }
            }
            pub mod get {
                pub struct Get<'db> {
                    pub name: &'db String,
                    pub premium: &'db bool,
                    pub credits: &'db i32,
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
                        credits: primary.imm_data.credits,
                        premium: primary.imm_data.premium,
                        name: primary.imm_data.name,
                    })
                }
            }
            pub mod updates {}
            impl<'imm> Window<'imm> {}
            pub mod insert {
                pub struct Insert {
                    pub credits: i32,
                    pub premium: bool,
                    pub name: String,
                }
                #[derive(Debug)]
                pub enum Error {
                    prem_credits,
                }
            }
            impl<'imm> Window<'imm> {
                pub fn insert(
                    &mut self,
                    insert_val: insert::Insert,
                ) -> Result<Key, insert::Error> {
                    if !predicates::prem_credits(borrows::Borrows {
                        credits: &insert_val.credits,
                        premium: &insert_val.premium,
                        name: &insert_val.name,
                    }) {
                        return Err(insert::Error::prem_credits);
                    }
                    let primary = (emdb::dependencies::pulpit::column::Data {
                        imm_data: column_types::primary::Imm {
                            name: insert_val.name,
                            premium: insert_val.premium,
                            credits: insert_val.credits,
                        },
                        mut_data: column_types::primary::Mut {},
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
            pub type Key = <emdb::dependencies::pulpit::column::PrimaryAppend<
                emdb::dependencies::pulpit::column::AssocBlocks<
                    column_types::primary::Imm,
                    column_types::primary::Mut,
                    1024usize,
                >,
            > as emdb::dependencies::pulpit::column::Keyable>::Key;
            mod predicates {
                pub fn prem_credits(
                    super::borrows::Borrows {
                        credits,
                        premium,
                        name,
                    }: super::borrows::Borrows,
                ) -> bool {
                    *premium || *credits > 0
                }
            }
            struct Uniques {}
            impl Uniques {
                fn new(size_hint: usize) -> Self {
                    Self {}
                }
            }
            struct ColumnHolder {
                primary: emdb::dependencies::pulpit::column::PrimaryAppend<
                    emdb::dependencies::pulpit::column::AssocBlocks<
                        column_types::primary::Imm,
                        column_types::primary::Mut,
                        1024usize,
                    >,
                >,
            }
            impl ColumnHolder {
                fn new(size_hint: usize) -> Self {
                    Self {
                        primary: emdb::dependencies::pulpit::column::PrimaryAppend::new(
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
                primary: <emdb::dependencies::pulpit::column::PrimaryAppend<
                    emdb::dependencies::pulpit::column::AssocBlocks<
                        column_types::primary::Imm,
                        column_types::primary::Mut,
                        1024usize,
                    >,
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
        pub mod total_premium_credits {
            #[derive(Debug)]
            pub enum Error {
                Error1,
            }
        }
    }
    struct RecordTypeAlias0<'db, 'qy> {
        credits: &'db i32,
        name: &'db String,
        premium: &'db bool,
        __internal_phantomdata: std::marker::PhantomData<(&'db (), &'qy ())>,
    }
    struct RecordTypeAlias1<'db, 'qy> {
        __internal_0: tables::users::Key,
        __internal_phantomdata: std::marker::PhantomData<(&'db (), &'qy ())>,
    }
    struct RecordTypeAlias2<'db, 'qy> {
        __internal_1: RecordTypeAlias0<'db, 'qy>,
        __internal_0: tables::users::Key,
        __internal_phantomdata: std::marker::PhantomData<(&'db (), &'qy ())>,
    }
    struct RecordTypeAlias3<'db, 'qy> {
        credits: i64,
        __internal_phantomdata: std::marker::PhantomData<(&'db (), &'qy ())>,
    }
    pub struct RecordTypeAlias4<'db, 'qy> {
        pub sum: i64,
        __internal_phantomdata: std::marker::PhantomData<(&'db (), &'qy ())>,
    }
    pub struct Datastore {
        users: tables::users::Table,
    }
    impl Datastore {
        pub fn new() -> Self {
            Self {
                users: tables::users::Table::new(1024),
            }
        }
        pub fn db(&mut self) -> Database<'_> {
            Database {
                users: self.users.window(),
            }
        }
    }
    pub struct Database<'db> {
        users: tables::users::Window<'db>,
    }
    impl<'db> Database<'db> {
        pub fn total_premium_credits<'qy>(
            &'qy self,
        ) -> Result<RecordTypeAlias4, queries::total_premium_credits::Error> {
            let (
                operator_closure_value_0,
                operator_closure_value_1,
                operator_closure_value_2,
                operator_closure_value_3,
                operator_closure_value_4,
                operator_closure_value_5,
                operator_closure_value_6,
            ) = (
                (),
                (),
                (),
                |
                    RecordTypeAlias0 {
                        credits: credits,
                        name: name,
                        premium: premium,
                        __internal_phantomdata: _,
                    }: &RecordTypeAlias0<'db, 'qy>,
                | -> bool { *premium },
                |
                    RecordTypeAlias0 {
                        credits: credits,
                        name: name,
                        premium: premium,
                        __internal_phantomdata: _,
                    }|
                {
                    RecordTypeAlias3 {
                        credits: credits as i64,
                        __internal_phantomdata: std::marker::PhantomData,
                    }
                },
                (
                    RecordTypeAlias4 {
                        sum: 0,
                        __internal_phantomdata: std::marker::PhantomData,
                    },
                    |
                        RecordTypeAlias4 {
                            sum: sum,
                            __internal_phantomdata: _,
                        }: &mut RecordTypeAlias4<'db, 'qy>,
                        RecordTypeAlias3 { credits: credits, __internal_phantomdata: _ }|
                    {
                        *sum = { sum + credits };
                    },
                ),
                (),
            );
            Ok({
                let dataflow_value_0: <emdb::dependencies::minister::Basic as emdb::dependencies::minister::Physical>::Stream<
                    RecordTypeAlias1<'db, 'qy>,
                > = {
                    let stream_values = emdb::dependencies::minister::Basic::consume_stream(
                        self.users.scan().collect::<Vec<_>>().into_iter(),
                    );
                    emdb::dependencies::minister::Basic::map(
                        stream_values,
                        |value| RecordTypeAlias1 {
                            __internal_0: value,
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
                            match self.users.get(dataflow_value_0.__internal_0) {
                                Ok(get_value) => {
                                    Ok(RecordTypeAlias2 {
                                        __internal_1: RecordTypeAlias0 {
                                            credits: get_value.credits,
                                            name: get_value.name,
                                            premium: get_value.premium,
                                            __internal_phantomdata: std::marker::PhantomData,
                                        },
                                        __internal_0: dataflow_value_0.__internal_0,
                                        __internal_phantomdata: std::marker::PhantomData,
                                    })
                                }
                                Err(_) => {
                                    return Err(queries::total_premium_credits::Error::Error1);
                                }
                            }
                        },
                    );
                    match emdb::dependencies::minister::Basic::error_stream(result) {
                        Ok(val) => val,
                        Err(err) => {
                            return Err(queries::total_premium_credits::Error::Error1);
                        }
                    }
                };
                let dataflow_value_2: <emdb::dependencies::minister::Basic as emdb::dependencies::minister::Physical>::Stream<
                    RecordTypeAlias0<'db, 'qy>,
                > = emdb::dependencies::minister::Basic::map(
                    dataflow_value_1,
                    |dataflow_value_1| dataflow_value_1.__internal_1,
                );
                let dataflow_value_3: <emdb::dependencies::minister::Basic as emdb::dependencies::minister::Physical>::Stream<
                    RecordTypeAlias0<'db, 'qy>,
                > = emdb::dependencies::minister::Basic::filter(
                    dataflow_value_2,
                    operator_closure_value_3,
                );
                let dataflow_value_4: <emdb::dependencies::minister::Basic as emdb::dependencies::minister::Physical>::Stream<
                    RecordTypeAlias3<'db, 'qy>,
                > = emdb::dependencies::minister::Basic::map(
                    dataflow_value_3,
                    operator_closure_value_4,
                );
                let dataflow_value_5: <emdb::dependencies::minister::Basic as emdb::dependencies::minister::Physical>::Single<
                    RecordTypeAlias4<'db, 'qy>,
                > = {
                    let (init, update) = operator_closure_value_5;
                    emdb::dependencies::minister::Basic::fold(
                        dataflow_value_4,
                        init,
                        update,
                    )
                };
                let return_value_6: <emdb::dependencies::minister::Basic as emdb::dependencies::minister::Physical>::Single<
                    RecordTypeAlias4<'db, 'qy>,
                > = dataflow_value_5;
                return_value_6
            })
        }
    }
}
