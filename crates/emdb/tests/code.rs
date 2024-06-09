#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LogLevel {
    Error,
    Warning,
    Info,
}

mod my_db {
    #![allow(non_shorthand_field_patterns)]
    use emdb::dependencies::minister::Physical;
    pub mod tables {
        pub mod logs {
            #![allow(unused, non_camel_case_types)]
            use emdb::dependencies::pulpit::column::{
                AssocWindow, AssocWindowPull, Column, PrimaryWindow, PrimaryWindowApp,
                PrimaryWindowHide, PrimaryWindowPull,
            };
            #[derive(Debug)]
            pub struct KeyError;
            mod column_types {
                //! Column types to be used for storage in each column.
                pub mod primary {
                    #[derive(Clone)]
                    pub struct Imm {
                        pub timestamp: u64,
                        pub comment: Option<String>,
                    }
                    #[derive(Clone)]
                    pub struct Mut {
                        pub level: crate::LogLevel,
                    }
                    pub struct ImmUnpack<'imm> {
                        pub timestamp: &'imm u64,
                        pub comment: &'imm Option<String>,
                    }
                    pub fn imm_unpack<'imm>(
                        Imm { timestamp, comment }: &'imm Imm,
                    ) -> ImmUnpack<'imm> {
                        ImmUnpack { timestamp, comment }
                    }
                }
            }
            pub mod borrows {
                pub struct Borrows<'brw> {
                    pub timestamp: &'brw u64,
                    pub level: &'brw crate::LogLevel,
                    pub comment: &'brw Option<String>,
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
                        timestamp: &primary.imm_data.timestamp,
                        level: &primary.mut_data.level,
                        comment: &primary.imm_data.comment,
                    })
                }
            }
            pub mod get {
                pub struct Get<'db> {
                    pub level: crate::LogLevel,
                    pub timestamp: &'db u64,
                    pub comment: &'db Option<String>,
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
                        timestamp: primary.imm_data.timestamp,
                        level: primary.mut_data.level,
                        comment: primary.imm_data.comment,
                    })
                }
            }
            pub mod updates {
                pub mod pulpit_access_23 {
                    #[derive(Debug)]
                    pub enum UpdateError {
                        KeyError,
                    }
                    pub struct Update {
                        pub level: crate::LogLevel,
                    }
                }
            }
            impl<'imm> Window<'imm> {
                pub fn pulpit_access_23(
                    &mut self,
                    update: updates::pulpit_access_23::Update,
                    key: Key,
                ) -> Result<(), updates::pulpit_access_23::UpdateError> {
                    let emdb::dependencies::pulpit::column::Entry {
                        index,
                        data: primary,
                    } = match self.columns.primary.brw_mut(key) {
                        Ok(entry) => entry,
                        Err(_) => {
                            return Err(updates::pulpit_access_23::UpdateError::KeyError);
                        }
                    };
                    let mut update = update;
                    std::mem::swap(&mut primary.mut_data.level, &mut update.level);
                    if !self.transactions.rollback {
                        self.transactions.log.push(transactions::LogItem::Update(
                            key,
                            transactions::Updates::pulpit_access_23(update),
                        ));
                    }
                    Ok(())
                }
            }
            pub mod insert {
                pub struct Insert {
                    pub timestamp: u64,
                    pub level: crate::LogLevel,
                    pub comment: Option<String>,
                }
            }
            impl<'imm> Window<'imm> {
                pub fn insert(&mut self, insert_val: insert::Insert) -> Key {
                    let primary = (emdb::dependencies::pulpit::column::Data {
                        imm_data: column_types::primary::Imm {
                            timestamp: insert_val.timestamp,
                            comment: insert_val.comment,
                        },
                        mut_data: column_types::primary::Mut {
                            level: insert_val.level,
                        },
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
                pub enum Updates {
                    pulpit_access_23(super::updates::pulpit_access_23::Update),
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
                    debug_assert!(!self.transactions.rollback);
                    self.transactions.log.clear()
                }
                /// Undo the transactions applied since the last commit
                /// - Requires re-applying all updates, deleting inserts and undoing deletes
                ///   (deletes' keys are actually just hidden until commit or abort)
                pub fn abort(&mut self) {
                    self.transactions.rollback = true;
                    while let Some(entry) = self.transactions.log.pop() {
                        match entry {
                            transactions::LogItem::Append => unsafe {
                                self.columns.primary.unppend();
                            },
                            transactions::LogItem::Update(key, update) => match update {
                                transactions::Updates::pulpit_access_23(update) => {
                                    self.pulpit_access_23(update, key).unwrap();
                                }
                            },
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
                        primary: emdb::dependencies::pulpit::column::AssocBlocks::new(size_hint),
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
        mod add_event {}
        mod get_errors_per_minute {}
        mod get_comment_summaries {}
        pub mod demote_error_logs {
            #[derive(Debug)]
            pub enum Error {
                Error22,
                Error23(super::super::tables::logs::updates::pulpit_access_23::UpdateError),
            }
        }
    }
    #[derive(Clone)]
    struct Record0<'db, 'qy> {
        comment: Option<String>,
        level: crate::LogLevel,
        timestamp: u64,
        __internal_phantomdata: std::marker::PhantomData<(&'db (), &'qy ())>,
    }
    #[derive(Clone)]
    struct Record1<'db, 'qy> {
        timestamp: u64,
        level: crate::LogLevel,
        comment: Option<String>,
        __internal_phantomdata: std::marker::PhantomData<(&'db (), &'qy ())>,
    }
    #[derive(Clone)]
    struct Record2<'db, 'qy> {
        log_id: tables::logs::Key,
        __internal_phantomdata: std::marker::PhantomData<(&'db (), &'qy ())>,
    }
    #[derive(Clone)]
    struct Record3<'db, 'qy> {
        timestamp: &'db u64,
        level: crate::LogLevel,
        comment: &'db Option<String>,
        __internal_phantomdata: std::marker::PhantomData<(&'db (), &'qy ())>,
    }
    #[derive(Clone)]
    struct Record4<'db, 'qy> {
        __internal_0: tables::logs::Key,
        __internal_phantomdata: std::marker::PhantomData<(&'db (), &'qy ())>,
    }
    #[derive(Clone)]
    struct Record5<'db, 'qy> {
        __internal_0: tables::logs::Key,
        __internal_1: Record3<'db, 'qy>,
        __internal_phantomdata: std::marker::PhantomData<(&'db (), &'qy ())>,
    }
    #[derive(Clone)]
    struct Record6<'db, 'qy> {
        min: u64,
        __internal_phantomdata: std::marker::PhantomData<(&'db (), &'qy ())>,
    }
    #[derive(Clone)]
    struct Record7<'db, 'qy> {
        __internal_phantomdata: std::marker::PhantomData<(&'db (), &'qy ())>,
    }
    #[derive(Clone)]
    struct Record8<'db, 'qy> {
        num_logs: usize,
        __internal_phantomdata: std::marker::PhantomData<(&'db (), &'qy ())>,
    }
    #[derive(Clone)]
    pub struct Record9<'db, 'qy> {
        pub min: u64,
        pub errors: usize,
        __internal_phantomdata: std::marker::PhantomData<(&'db (), &'qy ())>,
    }
    #[derive(Clone)]
    pub struct Record10<'db, 'qy> {
        pub errors: Vec<Record9<'db, 'qy>>,
        __internal_phantomdata: std::marker::PhantomData<(&'db (), &'qy ())>,
    }
    #[derive(Clone)]
    struct Record11<'db, 'qy> {
        timestamp: &'db u64,
        comment: &'db Option<String>,
        level: crate::LogLevel,
        __internal_phantomdata: std::marker::PhantomData<(&'db (), &'qy ())>,
    }
    #[derive(Clone)]
    struct Record12<'db, 'qy> {
        __internal_0: tables::logs::Key,
        __internal_phantomdata: std::marker::PhantomData<(&'db (), &'qy ())>,
    }
    #[derive(Clone)]
    struct Record13<'db, 'qy> {
        __internal_1: Record11<'db, 'qy>,
        __internal_0: tables::logs::Key,
        __internal_phantomdata: std::marker::PhantomData<(&'db (), &'qy ())>,
    }
    #[derive(Clone)]
    pub struct Record14<'db, 'qy> {
        pub comment: &'db str,
        pub length: usize,
        __internal_phantomdata: std::marker::PhantomData<(&'db (), &'qy ())>,
    }
    #[derive(Clone)]
    pub struct Record15<'db, 'qy> {
        pub comments: Vec<Record14<'db, 'qy>>,
        __internal_phantomdata: std::marker::PhantomData<(&'db (), &'qy ())>,
    }
    #[derive(Clone)]
    struct Record16<'db, 'qy> {
        log_ref: tables::logs::Key,
        __internal_phantomdata: std::marker::PhantomData<(&'db (), &'qy ())>,
    }
    #[derive(Clone)]
    struct Record17<'db, 'qy> {
        timestamp: &'db u64,
        comment: &'db Option<String>,
        level: crate::LogLevel,
        __internal_phantomdata: std::marker::PhantomData<(&'db (), &'qy ())>,
    }
    #[derive(Clone)]
    struct Record18<'db, 'qy> {
        log_ref: tables::logs::Key,
        log_data: Record17<'db, 'qy>,
        __internal_phantomdata: std::marker::PhantomData<(&'db (), &'qy ())>,
    }
    #[derive(Clone)]
    struct Record19<'db, 'qy> {
        level: crate::LogLevel,
        __internal_phantomdata: std::marker::PhantomData<(&'db (), &'qy ())>,
    }
    pub struct EmDBDebug {
        logs: tables::logs::Table,
    }
    impl EmDBDebug {
        pub fn new() -> Self {
            Self {
                logs: tables::logs::Table::new(1024),
            }
        }
        pub fn db(&mut self) -> Database<'_> {
            Database {
                logs: self.logs.window(),
            }
        }
    }
    pub struct Database<'db> {
        logs: tables::logs::Window<'db>,
    }
    impl<'db> Database<'db> {
        pub fn add_event<'qy>(
            &'qy mut self,
            timestamp: u64,
            comment: Option<String>,
            log_level: crate::LogLevel,
        ) -> () {
            let result = (|__internal_self: &mut Self,
                           timestamp: u64,
                           comment: Option<String>,
                           log_level: crate::LogLevel| {
                let (operator_closure_value_0) = (Record0 {
                    timestamp: timestamp,
                    comment: comment,
                    level: log_level,
                    __internal_phantomdata: std::marker::PhantomData,
                });
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
                            log_id: __internal_self
                                .logs
                                .insert(tables::logs::insert::Insert {
                                    comment: dataflow_value_0.comment,
                                    level: dataflow_value_0.level,
                                    timestamp: dataflow_value_0.timestamp,
                                }),
                            __internal_phantomdata: std::marker::PhantomData,
                        }
                    },
                );
                ()
            })(self, timestamp, comment, log_level);
            {
                self.logs.commit();
            }
            result
        }
        pub fn get_errors_per_minute<'qy>(&'qy self) -> Record10 {
            (|__internal_self: &Self| {
                let (
                    operator_closure_value_6,
                    operator_closure_value_7,
                    operator_closure_value_8,
                ) = (
                    |
                        Record3 {
                            timestamp: timestamp,
                            level: level,
                            comment: comment,
                            __internal_phantomdata: _,
                        }: &Record3<'db, 'qy>,
                    | -> bool { *level == crate::LogLevel::Error },
                    |
                        Record3 {
                            timestamp: timestamp,
                            level: level,
                            comment: comment,
                            __internal_phantomdata: _,
                        }|
                    {
                        Record6 {
                            min: timestamp % 60,
                            __internal_phantomdata: std::marker::PhantomData,
                        }
                    },
                    |
                        __internal_self: &Self,
                        min: u64,
                        dataflow_value_8: <emdb::dependencies::minister::Basic as emdb::dependencies::minister::Physical>::Stream<
                            Record7<'db, 'qy>,
                        >|
                    {
                        let (operator_closure_value_10) = (|
                            Record8 { num_logs: num_logs, __internal_phantomdata: _ }|
                        {
                            Record9 {
                                min: min,
                                errors: num_logs,
                                __internal_phantomdata: std::marker::PhantomData,
                            }
                        });
                        let dataflow_value_9: <emdb::dependencies::minister::Basic as emdb::dependencies::minister::Physical>::Single<
                            Record8<'db, 'qy>,
                        > = emdb::dependencies::minister::Basic::map_single(
                            emdb::dependencies::minister::Basic::count(dataflow_value_8),
                            |count| Record8 {
                                num_logs: count,
                                __internal_phantomdata: std::marker::PhantomData,
                            },
                        );
                        let dataflow_value_10: <emdb::dependencies::minister::Basic as emdb::dependencies::minister::Physical>::Single<
                            Record9<'db, 'qy>,
                        > = emdb::dependencies::minister::Basic::map_single(
                            dataflow_value_9,
                            operator_closure_value_10,
                        );
                        let return_value_11: <emdb::dependencies::minister::Basic as emdb::dependencies::minister::Physical>::Single<
                            Record9<'db, 'qy>,
                        > = dataflow_value_10;
                        return_value_11
                    },
                );
                let dataflow_value_2: <emdb::dependencies::minister::Basic as emdb::dependencies::minister::Physical>::Stream<
                    Record4<'db, 'qy>,
                > = {
                    let stream_values = emdb::dependencies::minister::Basic::consume_stream(
                        __internal_self.logs.scan().collect::<Vec<_>>().into_iter(),
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
                            match __internal_self.logs.get(dataflow_value_2.__internal_0)
                            {
                                Ok(get_value) => {
                                    Record5 {
                                        __internal_1: Record3 {
                                            timestamp: get_value.timestamp,
                                            level: get_value.level,
                                            comment: get_value.comment,
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
                    Record3<'db, 'qy>,
                > = emdb::dependencies::minister::Basic::filter(
                    dataflow_value_4,
                    operator_closure_value_6,
                );
                let dataflow_value_6: <emdb::dependencies::minister::Basic as emdb::dependencies::minister::Physical>::Stream<
                    Record6<'db, 'qy>,
                > = emdb::dependencies::minister::Basic::map(
                    dataflow_value_5,
                    operator_closure_value_7,
                );
                let dataflow_value_7: <emdb::dependencies::minister::Basic as emdb::dependencies::minister::Physical>::Stream<
                    Record9<'db, 'qy>,
                > = {
                    let split_vars = emdb::dependencies::minister::Basic::map(
                        dataflow_value_6,
                        |input| {
                            (
                                input.min,
                                Record7 {
                                    __internal_phantomdata: std::marker::PhantomData,
                                },
                            )
                        },
                    );
                    let grouped = emdb::dependencies::minister::Basic::group_by(
                        split_vars,
                    );
                    let results = emdb::dependencies::minister::Basic::map(
                        grouped,
                        |(grouping, inner_stream)| {
                            (operator_closure_value_8)(
                                __internal_self,
                                grouping,
                                inner_stream,
                            )
                        },
                    );
                    results
                };
                let dataflow_value_11: <emdb::dependencies::minister::Basic as emdb::dependencies::minister::Physical>::Single<
                    Record10<'db, 'qy>,
                > = emdb::dependencies::minister::Basic::consume_single(Record10 {
                    errors: emdb::dependencies::minister::Basic::export_stream(
                            dataflow_value_7,
                        )
                        .collect::<Vec<_>>(),
                    __internal_phantomdata: std::marker::PhantomData,
                });
                let return_value_13: <emdb::dependencies::minister::Basic as emdb::dependencies::minister::Physical>::Single<
                    Record10<'db, 'qy>,
                > = dataflow_value_11;
                return_value_13
            })(self)
        }
        pub fn get_comment_summaries<'qy>(&'qy self, time_start: u64, time_end: u64) -> Record15 {
            (|__internal_self: &Self, time_start: u64, time_end: u64| {
                let (operator_closure_value_17, operator_closure_value_18) = (
                    |Record11 {
                         timestamp: timestamp,
                         comment: comment,
                         level: level,
                         __internal_phantomdata: _,
                     }: &Record11<'db, 'qy>|
                     -> bool {
                        **timestamp >= time_start && **timestamp <= time_end && comment.is_some()
                    },
                    |Record11 {
                         timestamp: timestamp,
                         comment: comment,
                         level: level,
                         __internal_phantomdata: _,
                     }| {
                        Record14 {
                            comment: &comment.as_ref().unwrap()[..100],
                            length: comment.as_ref().unwrap().len(),
                            __internal_phantomdata: std::marker::PhantomData,
                        }
                    },
                );
                let dataflow_value_12: <emdb::dependencies::minister::Basic as emdb::dependencies::minister::Physical>::Stream<
                    Record12<'db, 'qy>,
                > = {
                    let stream_values = emdb::dependencies::minister::Basic::consume_stream(
                        __internal_self.logs.scan().collect::<Vec<_>>().into_iter(),
                    );
                    emdb::dependencies::minister::Basic::map(
                        stream_values,
                        |value| Record12 {
                            __internal_0: value,
                            __internal_phantomdata: std::marker::PhantomData,
                        },
                    )
                };
                let dataflow_value_13: <emdb::dependencies::minister::Basic as emdb::dependencies::minister::Physical>::Stream<
                    Record13<'db, 'qy>,
                > = {
                    emdb::dependencies::minister::Basic::map(
                        dataflow_value_12,
                        |dataflow_value_12| {
                            match __internal_self
                                .logs
                                .get(dataflow_value_12.__internal_0)
                            {
                                Ok(get_value) => {
                                    Record13 {
                                        __internal_1: Record11 {
                                            timestamp: get_value.timestamp,
                                            comment: get_value.comment,
                                            level: get_value.level,
                                            __internal_phantomdata: std::marker::PhantomData,
                                        },
                                        __internal_0: dataflow_value_12.__internal_0,
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
                let dataflow_value_14: <emdb::dependencies::minister::Basic as emdb::dependencies::minister::Physical>::Stream<
                    Record11<'db, 'qy>,
                > = emdb::dependencies::minister::Basic::map(
                    dataflow_value_13,
                    |dataflow_value_13| dataflow_value_13.__internal_1,
                );
                let dataflow_value_15: <emdb::dependencies::minister::Basic as emdb::dependencies::minister::Physical>::Stream<
                    Record11<'db, 'qy>,
                > = emdb::dependencies::minister::Basic::filter(
                    dataflow_value_14,
                    operator_closure_value_17,
                );
                let dataflow_value_16: <emdb::dependencies::minister::Basic as emdb::dependencies::minister::Physical>::Stream<
                    Record14<'db, 'qy>,
                > = emdb::dependencies::minister::Basic::map(
                    dataflow_value_15,
                    operator_closure_value_18,
                );
                let dataflow_value_17: <emdb::dependencies::minister::Basic as emdb::dependencies::minister::Physical>::Single<
                    Record15<'db, 'qy>,
                > = emdb::dependencies::minister::Basic::consume_single(Record15 {
                    comments: emdb::dependencies::minister::Basic::export_stream(
                            dataflow_value_16,
                        )
                        .collect::<Vec<_>>(),
                    __internal_phantomdata: std::marker::PhantomData,
                });
                let return_value_20: <emdb::dependencies::minister::Basic as emdb::dependencies::minister::Physical>::Single<
                    Record15<'db, 'qy>,
                > = dataflow_value_17;
                return_value_20
            })(self, time_start, time_end)
        }
        pub fn demote_error_logs<'qy>(
            &'qy mut self,
        ) -> Result<(), queries::demote_error_logs::Error> {
            match (|__internal_self: &mut Self| {
                let (operator_closure_value_23) =
                    (|Record18 {
                          log_ref, log_data, ..
                      }| {
                        (
                            Record19 {
                                level: (if crate::LogLevel::Error == log_data.level {
                                    crate::LogLevel::Warning
                                } else {
                                    log_data.level.clone()
                                }),
                                __internal_phantomdata: std::marker::PhantomData,
                            },
                            Record18 {
                                log_ref,
                                log_data,
                                __internal_phantomdata: std::marker::PhantomData,
                            },
                        )
                    });
                let dataflow_value_18: <emdb::dependencies::minister::Basic as emdb::dependencies::minister::Physical>::Stream<
                    Record16<'db, 'qy>,
                > = {
                    let stream_values = emdb::dependencies::minister::Basic::consume_stream(
                        __internal_self.logs.scan().collect::<Vec<_>>().into_iter(),
                    );
                    emdb::dependencies::minister::Basic::map(
                        stream_values,
                        |value| Record16 {
                            log_ref: value,
                            __internal_phantomdata: std::marker::PhantomData,
                        },
                    )
                };
                let dataflow_value_19: <emdb::dependencies::minister::Basic as emdb::dependencies::minister::Physical>::Stream<
                    Record18<'db, 'qy>,
                > = {
                    let result = emdb::dependencies::minister::Basic::map(
                        dataflow_value_18,
                        |dataflow_value_18| {
                            match __internal_self.logs.get(dataflow_value_18.log_ref) {
                                Ok(get_value) => {
                                    Ok(Record18 {
                                        log_data: Record17 {
                                            timestamp: get_value.timestamp,
                                            comment: get_value.comment,
                                            level: get_value.level,
                                            __internal_phantomdata: std::marker::PhantomData,
                                        },
                                        log_ref: dataflow_value_18.log_ref,
                                        __internal_phantomdata: std::marker::PhantomData,
                                    })
                                }
                                Err(_) => {
                                    return Err(queries::demote_error_logs::Error::Error22);
                                }
                            }
                        },
                    );
                    match emdb::dependencies::minister::Basic::error_stream(result) {
                        Ok(val) => val,
                        Err(err) => {
                            return Err(queries::demote_error_logs::Error::Error22);
                        }
                    }
                };
                let dataflow_value_20: <emdb::dependencies::minister::Basic as emdb::dependencies::minister::Physical>::Stream<
                    Record18<'db, 'qy>,
                > = {
                    let results = emdb::dependencies::minister::Basic::map_seq(
                        dataflow_value_19,
                        |dataflow_value_19| {
                            let (update_struct, continue_struct) = operator_closure_value_23
                                .clone()(dataflow_value_19);
                            match __internal_self
                                .logs
                                .pulpit_access_23(
                                    tables::logs::updates::pulpit_access_23::Update {
                                        level: update_struct.level,
                                    },
                                    continue_struct.log_ref,
                                )
                            {
                                Ok(()) => Ok(continue_struct),
                                Err(err) => {
                                    Err(queries::demote_error_logs::Error::Error23(err))
                                }
                            }
                        },
                    );
                    emdb::dependencies::minister::Basic::error_stream(results)?
                };
                Ok(())
            })(self)
            {
                Ok(result) => {
                    {
                        self.logs.commit();
                    }
                    Ok(emdb::dependencies::minister::Basic::export_single(result))
                }
                Err(e) => {
                    {
                        self.logs.abort();
                    }
                    Err(e)
                }
            }
        }
    }
}
