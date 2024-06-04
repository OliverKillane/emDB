mod my_db {
    #![allow(non_shorthand_field_patterns)]
    use emdb::dependencies::minister::Physical;
    pub mod tables {
        pub mod other {
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
                    pub struct Imm {}
                    #[derive(Clone)]
                    pub struct Mut {}
                    pub struct ImmUnpack<'imm> {
                        pub phantom: std::marker::PhantomData<&'imm ()>,
                    }
                    pub fn imm_unpack<'imm>(_: &'imm Imm) -> ImmUnpack<'imm> {
                        ImmUnpack {
                            phantom: std::marker::PhantomData,
                        }
                    }
                }
            }
            pub mod borrows {
                pub struct Borrows<'brw> {
                    pub phantom: std::marker::PhantomData<&'brw ()>,
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
                        phantom: std::marker::PhantomData,
                    })
                }
            }
            pub mod get {
                pub struct Get<'db> {}
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
                    Ok(get::Get {})
                }
            }
            pub mod updates {}
            impl<'imm> Window<'imm> {}
            pub mod insert {
                pub struct Insert {}
                #[derive(Debug)]
                pub enum Error {
                    check2,
                }
            }
            impl<'imm> Window<'imm> {
                pub fn insert(
                    &mut self,
                    insert_val: insert::Insert,
                ) -> Result<Key, insert::Error> {
                    if !predicates::check2(borrows::Borrows {}) {
                        return Err(insert::Error::check2);
                    }
                    let primary = (emdb::dependencies::pulpit::column::Data {
                        imm_data: column_types::primary::Imm {},
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
            pub type Key = <emdb::dependencies::pulpit::column::AssocBlocks<
                column_types::primary::Imm,
                column_types::primary::Mut,
                1024usize,
            > as emdb::dependencies::pulpit::column::Keyable>::Key;
            mod predicates {
                pub fn check2(
                    super::borrows::Borrows {}: super::borrows::Borrows,
                ) -> bool {
                    1 + 1 == 2
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
        pub mod simple {
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
                        pub c: (u32, i32),
                        pub a: i32,
                    }
                    #[derive(Clone)]
                    pub struct Mut {
                        pub b: String,
                    }
                    pub struct ImmUnpack<'imm> {
                        pub c: &'imm (u32, i32),
                        pub a: &'imm i32,
                    }
                    pub fn imm_unpack<'imm>(Imm { c, a }: &'imm Imm) -> ImmUnpack<'imm> {
                        ImmUnpack { c, a }
                    }
                }
            }
            pub mod borrows {
                pub struct Borrows<'brw> {
                    pub b: &'brw String,
                    pub c: &'brw (u32, i32),
                    pub a: &'brw i32,
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
                        b: &primary.mut_data.b,
                        c: &primary.imm_data.c,
                        a: &primary.imm_data.a,
                    })
                }
            }
            pub mod get {
                pub struct Get<'db> {
                    pub b: String,
                    pub c: &'db (u32, i32),
                    pub a: &'db i32,
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
                        b: primary.mut_data.b,
                        c: primary.imm_data.c,
                        a: primary.imm_data.a,
                    })
                }
            }
            pub mod updates {
                pub mod pulpit_access_4 {
                    #[derive(Debug)]
                    pub enum UpdateError {
                        KeyError,
                        c_predicate,
                        b_length,
                    }
                    pub struct Update {
                        pub b: String,
                    }
                }
            }
            impl<'imm> Window<'imm> {
                pub fn pulpit_access_4(
                    &mut self,
                    update: updates::pulpit_access_4::Update,
                    key: Key,
                ) -> Result<(), updates::pulpit_access_4::UpdateError> {
                    let emdb::dependencies::pulpit::column::Entry {
                        index,
                        data: primary,
                    } = match self.columns.primary.brw_mut(key) {
                        Ok(entry) => entry,
                        Err(_) => {
                            return Err(updates::pulpit_access_4::UpdateError::KeyError);
                        }
                    };
                    if !predicates::c_predicate(borrows::Borrows {
                        c: &primary.imm_data.c,
                        a: &primary.imm_data.a,
                        b: &update.b,
                    }) {
                        return Err(updates::pulpit_access_4::UpdateError::c_predicate);
                    }
                    if !predicates::b_length(borrows::Borrows {
                        c: &primary.imm_data.c,
                        a: &primary.imm_data.a,
                        b: &update.b,
                    }) {
                        return Err(updates::pulpit_access_4::UpdateError::b_length);
                    }
                    let mut update = update;
                    std::mem::swap(&mut primary.mut_data.b, &mut update.b);
                    if !self.transactions.rollback {
                        self.transactions
                            .log
                            .push(
                                transactions::LogItem::Update(
                                    key,
                                    transactions::Updates::pulpit_access_4(update),
                                ),
                            );
                    }
                    Ok(())
                }
            }
            pub mod insert {
                pub struct Insert {
                    pub b: String,
                    pub c: (u32, i32),
                    pub a: i32,
                }
                #[derive(Debug)]
                pub enum Error {
                    simple_un,
                    c_predicate,
                    b_length,
                }
            }
            impl<'imm> Window<'imm> {
                pub fn insert(
                    &mut self,
                    insert_val: insert::Insert,
                ) -> Result<Key, insert::Error> {
                    if !predicates::c_predicate(borrows::Borrows {
                        b: &insert_val.b,
                        c: &insert_val.c,
                        a: &insert_val.a,
                    }) {
                        return Err(insert::Error::c_predicate);
                    }
                    if !predicates::b_length(borrows::Borrows {
                        b: &insert_val.b,
                        c: &insert_val.c,
                        a: &insert_val.a,
                    }) {
                        return Err(insert::Error::b_length);
                    }
                    let simple_un = match self.uniques.a.lookup(&insert_val.a) {
                        Ok(_) => return Err(insert::Error::simple_un),
                        Err(_) => insert_val.a.clone(),
                    };
                    let primary = (emdb::dependencies::pulpit::column::Data {
                        imm_data: column_types::primary::Imm {
                            c: insert_val.c,
                            a: insert_val.a,
                        },
                        mut_data: column_types::primary::Mut {
                            b: insert_val.b,
                        },
                    });
                    let (key, action) = self.columns.primary.insert(primary);
                    match action {
                        emdb::dependencies::pulpit::column::InsertAction::Place(
                            index,
                        ) => unsafe {}
                        emdb::dependencies::pulpit::column::InsertAction::Append => {}
                    }
                    self.uniques.a.insert(simple_un, key).unwrap();
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
                pub fn simple_un(&self, value: &i32) -> Result<Key, unique::NotFound> {
                    match self.uniques.a.lookup(value) {
                        Ok(k) => Ok(k),
                        Err(_) => Err(unique::NotFound),
                    }
                }
            }
            mod transactions {
                pub enum Updates {
                    pulpit_access_4(super::updates::pulpit_access_4::Update),
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
                                    transactions::Updates::pulpit_access_4(update) => {
                                        self.pulpit_access_4(update, key).unwrap();
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
                                self.uniques.a.pull(&primary.imm_data.a).unwrap();
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
                    self.uniques.a.insert(brw_data.a.clone(), key).unwrap();
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
                pub fn c_predicate(
                    super::borrows::Borrows { b, c, a }: super::borrows::Borrows,
                ) -> bool {
                    c.0 > c.1
                }
                pub fn b_length(
                    super::borrows::Borrows { b, c, a }: super::borrows::Borrows,
                ) -> bool {
                    b.len() < 10
                }
            }
            struct Uniques {
                a: emdb::dependencies::pulpit::access::Unique<i32, Key>,
            }
            impl Uniques {
                fn new(size_hint: usize) -> Self {
                    Self {
                        a: emdb::dependencies::pulpit::access::Unique::new(size_hint),
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
        pub mod insert {
            #[derive(Debug)]
            pub enum Error {
                Error1(super::super::tables::simple::insert::Error),
            }
        }
        pub mod update_b {
            #[derive(Debug)]
            pub enum Error {
                Error4(
                    super::super::tables::simple::updates::pulpit_access_4::UpdateError,
                ),
            }
        }
        mod single_maths {}
        pub mod remove_all {
            #[derive(Debug)]
            pub enum Error {
                Error12(super::super::tables::simple::KeyError),
            }
        }
    }
    struct RecordTypeAlias0<'db, 'qy> {
        c: (u32, i32),
        b: String,
        a: i32,
        __internal_phantomdata: std::marker::PhantomData<(&'db (), &'qy ())>,
    }
    struct RecordTypeAlias1<'db, 'qy> {
        b: String,
        c: (u32, i32),
        a: i32,
        __internal_phantomdata: std::marker::PhantomData<(&'db (), &'qy ())>,
    }
    pub struct RecordTypeAlias2<'db, 'qy> {
        pub it: tables::simple::Key,
        __internal_phantomdata: std::marker::PhantomData<(&'db (), &'qy ())>,
    }
    pub struct RecordTypeAlias3<'db, 'qy> {
        pub simple_ref: tables::simple::Key,
        __internal_phantomdata: std::marker::PhantomData<(&'db (), &'qy ())>,
    }
    struct RecordTypeAlias4<'db, 'qy> {
        b: String,
        __internal_phantomdata: std::marker::PhantomData<(&'db (), &'qy ())>,
    }
    pub struct RecordTypeAlias5<'db, 'qy> {
        pub it: Vec<RecordTypeAlias3<'db, 'qy>>,
        __internal_phantomdata: std::marker::PhantomData<(&'db (), &'qy ())>,
    }
    struct RecordTypeAlias6<'db, 'qy> {
        a: i32,
        b: i32,
        __internal_phantomdata: std::marker::PhantomData<(&'db (), &'qy ())>,
    }
    struct RecordTypeAlias7<'db, 'qy> {
        c: i32,
        __internal_phantomdata: std::marker::PhantomData<(&'db (), &'qy ())>,
    }
    pub struct RecordTypeAlias8<'db, 'qy> {
        pub z: i32,
        __internal_phantomdata: std::marker::PhantomData<(&'db (), &'qy ())>,
    }
    struct RecordTypeAlias9<'db, 'qy> {
        simple_ref: tables::simple::Key,
        __internal_phantomdata: std::marker::PhantomData<(&'db (), &'qy ())>,
    }
    pub struct Datastore {
        simple: tables::simple::Table,
        other: tables::other::Table,
    }
    impl Datastore {
        pub fn new() -> Self {
            Self {
                simple: tables::simple::Table::new(1024),
                other: tables::other::Table::new(1024),
            }
        }
        pub fn db(&mut self) -> Database<'_> {
            Database {
                simple: self.simple.window(),
                other: self.other.window(),
            }
        }
    }
    pub struct Database<'db> {
        simple: tables::simple::Window<'db>,
        other: tables::other::Window<'db>,
    }
    impl<'db> Database<'db> {
        pub fn insert<'qy>(
            &'qy mut self,
            a_initial: i32,
        ) -> Result<RecordTypeAlias2, queries::insert::Error> {
            match (|| {
                let (
                    operator_closure_value_0,
                    operator_closure_value_1,
                    operator_closure_value_2,
                ) = (
                    RecordTypeAlias0 {
                        a: a_initial,
                        b: "hello".to_string(),
                        c: (0, 0),
                        __internal_phantomdata: std::marker::PhantomData,
                    },
                    (),
                    (),
                );
                Ok({
                    let dataflow_value_0: <emdb::dependencies::minister::Basic as emdb::dependencies::minister::Physical>::Single<
                        RecordTypeAlias0<'db, 'qy>,
                    > = emdb::dependencies::minister::Basic::consume_single(
                        operator_closure_value_0,
                    );
                    let dataflow_value_1: <emdb::dependencies::minister::Basic as emdb::dependencies::minister::Physical>::Single<
                        RecordTypeAlias2<'db, 'qy>,
                    > = {
                        let result = emdb::dependencies::minister::Basic::map_single(
                            dataflow_value_0,
                            |dataflow_value_0| {
                                Ok(RecordTypeAlias2 {
                                    it: self
                                        .simple
                                        .insert(tables::simple::insert::Insert {
                                            c: dataflow_value_0.c,
                                            b: dataflow_value_0.b,
                                            a: dataflow_value_0.a,
                                        })?,
                                    __internal_phantomdata: std::marker::PhantomData,
                                })
                            },
                        );
                        match emdb::dependencies::minister::Basic::error_single(result) {
                            Ok(val) => val,
                            Err(err) => return Err(queries::insert::Error::Error1(err)),
                        }
                    };
                    let return_value_2: <emdb::dependencies::minister::Basic as emdb::dependencies::minister::Physical>::Single<
                        RecordTypeAlias2<'db, 'qy>,
                    > = dataflow_value_1;
                    return_value_2
                })
            })() {
                Ok(result) => {
                    {
                        self.simple.commit();
                    }
                    Ok(result)
                }
                Err(e) => {
                    {
                        self.simple.abort();
                    }
                    Err(e)
                }
            }
        }
        pub fn update_b<'qy>(
            &'qy mut self,
            new_b: String,
        ) -> Result<RecordTypeAlias5, queries::update_b::Error> {
            match (|| {
                let (
                    operator_closure_value_3,
                    operator_closure_value_4,
                    operator_closure_value_5,
                    operator_closure_value_6,
                ) = (
                    (),
                    |RecordTypeAlias3 { simple_ref, .. }| {
                        (
                            RecordTypeAlias4 {
                                b: new_b,
                                __internal_phantomdata: std::marker::PhantomData,
                            },
                            RecordTypeAlias3 {
                                simple_ref,
                                __internal_phantomdata: std::marker::PhantomData,
                            },
                        )
                    },
                    (),
                    (),
                );
                Ok({
                    let dataflow_value_2: <emdb::dependencies::minister::Basic as emdb::dependencies::minister::Physical>::Stream<
                        RecordTypeAlias3<'db, 'qy>,
                    > = {
                        let stream_values = emdb::dependencies::minister::Basic::consume_stream(
                            self.simple.scan().collect::<Vec<_>>().into_iter(),
                        );
                        emdb::dependencies::minister::Basic::map(
                            stream_values,
                            |value| RecordTypeAlias3 {
                                simple_ref: value,
                                __internal_phantomdata: std::marker::PhantomData,
                            },
                        )
                    };
                    let dataflow_value_3: <emdb::dependencies::minister::Basic as emdb::dependencies::minister::Physical>::Stream<
                        RecordTypeAlias3<'db, 'qy>,
                    > = {
                        let results = emdb::dependencies::minister::Basic::map_seq(
                            dataflow_value_2,
                            |dataflow_value_2| {
                                let (update_struct, continue_struct) = operator_closure_value_4
                                    .clone()(dataflow_value_2);
                                match self
                                    .simple
                                    .pulpit_access_4(
                                        tables::simple::updates::pulpit_access_4::Update {
                                            b: update_struct.b,
                                        },
                                        continue_struct.simple_ref,
                                    )
                                {
                                    Ok(()) => Ok(continue_struct),
                                    Err(err) => Err(queries::update_b::Error::Error4(err)),
                                }
                            },
                        );
                        emdb::dependencies::minister::Basic::error_stream(results)?
                    };
                    let dataflow_value_4: <emdb::dependencies::minister::Basic as emdb::dependencies::minister::Physical>::Single<
                        RecordTypeAlias5<'db, 'qy>,
                    > = emdb::dependencies::minister::Basic::consume_single(RecordTypeAlias5 {
                        it: emdb::dependencies::minister::Basic::export_stream(
                                dataflow_value_3,
                            )
                            .collect::<Vec<_>>(),
                        __internal_phantomdata: std::marker::PhantomData,
                    });
                    let return_value_6: <emdb::dependencies::minister::Basic as emdb::dependencies::minister::Physical>::Single<
                        RecordTypeAlias5<'db, 'qy>,
                    > = dataflow_value_4;
                    return_value_6
                })
            })() {
                Ok(result) => {
                    {
                        self.simple.commit();
                    }
                    Ok(result)
                }
                Err(e) => {
                    {
                        self.simple.abort();
                    }
                    Err(e)
                }
            }
        }
        pub fn single_maths<'qy>(&'qy self) -> RecordTypeAlias8 {
            let (
                operator_closure_value_7,
                operator_closure_value_8,
                operator_closure_value_9,
                operator_closure_value_10,
            ) = (
                RecordTypeAlias6 {
                    a: 0,
                    b: 2,
                    __internal_phantomdata: std::marker::PhantomData,
                },
                |RecordTypeAlias6 { a: a, b: b, __internal_phantomdata: _ }| {
                    RecordTypeAlias7 {
                        c: a + b,
                        __internal_phantomdata: std::marker::PhantomData,
                    }
                },
                |RecordTypeAlias7 { c: c, __internal_phantomdata: _ }| {
                    RecordTypeAlias8 {
                        z: c * c,
                        __internal_phantomdata: std::marker::PhantomData,
                    }
                },
                (),
            );
            {
                let dataflow_value_5: <emdb::dependencies::minister::Basic as emdb::dependencies::minister::Physical>::Single<
                    RecordTypeAlias6<'db, 'qy>,
                > = emdb::dependencies::minister::Basic::consume_single(
                    operator_closure_value_7,
                );
                let dataflow_value_6: <emdb::dependencies::minister::Basic as emdb::dependencies::minister::Physical>::Single<
                    RecordTypeAlias7<'db, 'qy>,
                > = emdb::dependencies::minister::Basic::map_single(
                    dataflow_value_5,
                    operator_closure_value_8,
                );
                let dataflow_value_7: <emdb::dependencies::minister::Basic as emdb::dependencies::minister::Physical>::Single<
                    RecordTypeAlias8<'db, 'qy>,
                > = emdb::dependencies::minister::Basic::map_single(
                    dataflow_value_6,
                    operator_closure_value_9,
                );
                let return_value_10: <emdb::dependencies::minister::Basic as emdb::dependencies::minister::Physical>::Single<
                    RecordTypeAlias8<'db, 'qy>,
                > = dataflow_value_7;
                return_value_10
            }
        }
        pub fn remove_all<'qy>(&'qy mut self) -> Result<(), queries::remove_all::Error> {
            match (|| {
                let (operator_closure_value_11, operator_closure_value_12) = ((), ());
                Ok({
                    let dataflow_value_8: <emdb::dependencies::minister::Basic as emdb::dependencies::minister::Physical>::Stream<
                        RecordTypeAlias9<'db, 'qy>,
                    > = {
                        let stream_values = emdb::dependencies::minister::Basic::consume_stream(
                            self.simple.scan().collect::<Vec<_>>().into_iter(),
                        );
                        emdb::dependencies::minister::Basic::map(
                            stream_values,
                            |value| RecordTypeAlias9 {
                                simple_ref: value,
                                __internal_phantomdata: std::marker::PhantomData,
                            },
                        )
                    };
                    let dataflow_value_9: <emdb::dependencies::minister::Basic as emdb::dependencies::minister::Physical>::Stream<
                        RecordTypeAlias9<'db, 'qy>,
                    > = {
                        let result = emdb::dependencies::minister::Basic::map_seq(
                            dataflow_value_8,
                            |dataflow_value_8| {
                                match self.simple.delete(dataflow_value_8.simple_ref) {
                                    Ok(()) => Ok(dataflow_value_8),
                                    Err(err) => Err(queries::remove_all::Error::Error12(err)),
                                }
                            },
                        );
                        emdb::dependencies::minister::Basic::error_stream(result)?
                    };
                    ()
                })
            })() {
                Ok(result) => {
                    {
                        self.simple.commit();
                    }
                    Ok(result)
                }
                Err(e) => {
                    {
                        self.simple.abort();
                    }
                    Err(e)
                }
            }
        }
    }
}
