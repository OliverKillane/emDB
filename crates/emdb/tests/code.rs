mod my_db {
    #![allow(non_shorthand_field_patterns)]
    #![allow(unused_variables)]
    #![allow(dead_code)]
    use emdb::dependencies::minister::parallel::ParallelOps;
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
                    #[inline(always)]
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
                pub struct pulpit_access_6<'db> {
                    pub bonus: i32,
                    pub surname: &'db String,
                    pub phantom: std::marker::PhantomData<&'db ()>,
                }
                pub struct pulpit_access_19<'db> {
                    pub bonus: i32,
                    pub surname: &'db String,
                    pub phantom: std::marker::PhantomData<&'db ()>,
                }
            }
            impl<'db> Window<'db> {
                pub fn pulpit_access_6(
                    &self,
                    key: Key,
                ) -> Result<get::pulpit_access_6<'db>, KeyError> {
                    let emdb::dependencies::pulpit::column::Entry {
                        index,
                        data: primary,
                    } = match self.columns.primary.get(key) {
                        Ok(entry) => entry,
                        Err(_) => return Err(KeyError),
                    };
                    let primary = primary.convert_imm(column_types::primary::imm_unpack);
                    Ok(get::pulpit_access_6 {
                        bonus: primary.mut_data.bonus.clone(),
                        surname: primary.imm_data.surname,
                        phantom: std::marker::PhantomData,
                    })
                }
                pub fn pulpit_access_19(
                    &self,
                    key: Key,
                ) -> Result<get::pulpit_access_19<'db>, KeyError> {
                    let emdb::dependencies::pulpit::column::Entry {
                        index,
                        data: primary,
                    } = match self.columns.primary.get(key) {
                        Ok(entry) => entry,
                        Err(_) => return Err(KeyError),
                    };
                    let primary = primary.convert_imm(column_types::primary::imm_unpack);
                    Ok(get::pulpit_access_19 {
                        bonus: primary.mut_data.bonus.clone(),
                        surname: primary.imm_data.surname,
                        phantom: std::marker::PhantomData,
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
                pub fn borrow_indices(&self) -> impl Iterator<Item = Key> + '_ {
                    self.columns.primary.scan_brw()
                }
                pub fn get_indices(&self) -> impl Iterator<Item = Key> + '_ {
                    self.columns.primary.scan_get()
                }
            }
            /// The key for accessing rows (delete, update, get)
            pub type Key = <emdb::dependencies::pulpit::column::AssocBlocks<
                column_types::primary::Imm,
                column_types::primary::Mut,
                4096usize,
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
                    4096usize,
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
                    4096usize,
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
                        pub age: u8,
                        pub surname: String,
                        pub forename: String,
                    }
                    #[derive(Clone)]
                    pub struct Mut {
                        pub bonus_points: i32,
                    }
                    pub struct ImmUnpack<'imm> {
                        pub age: &'imm u8,
                        pub surname: &'imm String,
                        pub forename: &'imm String,
                    }
                    #[inline(always)]
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
                    pub forename: &'brw String,
                    pub bonus_points: &'brw i32,
                    pub age: &'brw u8,
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
                        bonus_points: &primary.mut_data.bonus_points,
                        age: &primary.imm_data.age,
                    })
                }
            }
            pub mod get {
                pub struct pulpit_access_1<'db> {
                    pub surname: &'db String,
                    pub forename: &'db String,
                    pub bonus_points: i32,
                    pub age: &'db u8,
                    pub phantom: std::marker::PhantomData<&'db ()>,
                }
            }
            impl<'db> Window<'db> {
                pub fn pulpit_access_1(
                    &self,
                    key: Key,
                ) -> Result<get::pulpit_access_1<'db>, KeyError> {
                    let emdb::dependencies::pulpit::column::Entry {
                        index,
                        data: primary,
                    } = match self.columns.primary.get(key) {
                        Ok(entry) => entry,
                        Err(_) => return Err(KeyError),
                    };
                    let primary = primary.convert_imm(column_types::primary::imm_unpack);
                    Ok(get::pulpit_access_1 {
                        surname: primary.imm_data.surname,
                        forename: primary.imm_data.forename,
                        bonus_points: primary.mut_data.bonus_points.clone(),
                        age: primary.imm_data.age,
                        phantom: std::marker::PhantomData,
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
                        age: &primary.imm_data.age,
                        surname: &primary.imm_data.surname,
                        forename: &primary.imm_data.forename,
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
                    pub surname: String,
                    pub forename: String,
                    pub bonus_points: i32,
                    pub age: u8,
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
                        forename: &insert_val.forename,
                        bonus_points: &insert_val.bonus_points,
                        age: &insert_val.age,
                    }) {
                        return Err(insert::Error::sensible_ages);
                    }
                    let primary = (emdb::dependencies::pulpit::column::Data {
                        imm_data: column_types::primary::Imm {
                            age: insert_val.age,
                            surname: insert_val.surname,
                            forename: insert_val.forename,
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
                pub fn borrow_indices(&self) -> impl Iterator<Item = Key> + '_ {
                    self.columns.primary.scan_brw()
                }
                pub fn get_indices(&self) -> impl Iterator<Item = Key> + '_ {
                    self.columns.primary.scan_get()
                }
            }
            /// The key for accessing rows (delete, update, get)
            pub type Key = <emdb::dependencies::pulpit::column::AssocBlocks<
                column_types::primary::Imm,
                column_types::primary::Mut,
                4096usize,
            > as emdb::dependencies::pulpit::column::Keyable>::Key;
            mod predicates {
                #[inline(always)]
                pub fn sensible_ages(
                    super::borrows::Borrows {
                        surname,
                        forename,
                        bonus_points,
                        age,
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
                    4096usize,
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
                    4096usize,
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
                Error6,
                Error2(
                    super::super::tables::customers::updates::pulpit_access_2::UpdateError,
                ),
                Error5(super::super::tables::family_bonus::unique::NotFound),
                Error1,
                Error7(
                    super::super::tables::family_bonus::updates::pulpit_access_7::UpdateError,
                ),
            }
        }
        pub mod add_customer {
            #[derive(Debug)]
            pub enum Error {
                Error13(super::super::tables::customers::insert::Error),
            }
        }
        pub mod add_family {
            #[derive(Debug)]
            pub enum Error {
                Error16(super::super::tables::family_bonus::insert::Error),
            }
        }
        pub mod get_family {
            #[derive(Debug)]
            pub enum Error {
                Error19,
            }
        }
    }
    #[derive(Clone)]
    struct Record0<'db, 'qy> {
        ref_cust: tables::customers::Key,
        __internal_phantomdata: std::marker::PhantomData<(&'db (), &'qy ())>,
    }
    #[derive(Clone)]
    struct Record1<'db, 'qy> {
        surname: &'db String,
        forename: &'db String,
        bonus_points: i32,
        age: &'db u8,
        __internal_phantomdata: std::marker::PhantomData<(&'db (), &'qy ())>,
    }
    #[derive(Clone)]
    struct Record2<'db, 'qy> {
        ref_cust: tables::customers::Key,
        person: Record1<'db, 'qy>,
        __internal_phantomdata: std::marker::PhantomData<(&'db (), &'qy ())>,
    }
    #[derive(Clone)]
    struct Record3<'db, 'qy> {
        bonus_points: i32,
        __internal_phantomdata: std::marker::PhantomData<(&'db (), &'qy ())>,
    }
    #[derive(Clone)]
    struct Record4<'db, 'qy> {
        surname: String,
        __internal_phantomdata: std::marker::PhantomData<(&'db (), &'qy ())>,
    }
    #[derive(Clone)]
    struct Record5<'db, 'qy> {
        family_ref: tables::family_bonus::Key,
        surname: String,
        __internal_phantomdata: std::marker::PhantomData<(&'db (), &'qy ())>,
    }
    #[derive(Clone)]
    struct Record6<'db, 'qy> {
        bonus: i32,
        surname: &'db String,
        __internal_phantomdata: std::marker::PhantomData<(&'db (), &'qy ())>,
    }
    #[derive(Clone)]
    struct Record7<'db, 'qy> {
        family: Record6<'db, 'qy>,
        family_ref: tables::family_bonus::Key,
        surname: String,
        __internal_phantomdata: std::marker::PhantomData<(&'db (), &'qy ())>,
    }
    #[derive(Clone)]
    struct Record8<'db, 'qy> {
        bonus: i32,
        __internal_phantomdata: std::marker::PhantomData<(&'db (), &'qy ())>,
    }
    #[derive(Clone)]
    struct Record9<'db, 'qy> {
        __internal_phantomdata: std::marker::PhantomData<(&'db (), &'qy ())>,
    }
    #[derive(Clone)]
    struct Record10<'db, 'qy> {
        age: u8,
        bonus_points: i32,
        forename: String,
        surname: String,
        __internal_phantomdata: std::marker::PhantomData<(&'db (), &'qy ())>,
    }
    #[derive(Clone)]
    struct Record11<'db, 'qy> {
        surname: String,
        bonus_points: i32,
        forename: String,
        age: u8,
        __internal_phantomdata: std::marker::PhantomData<(&'db (), &'qy ())>,
    }
    #[derive(Clone)]
    pub struct Record12<'db, 'qy> {
        pub name: tables::customers::Key,
        __internal_phantomdata: std::marker::PhantomData<(&'db (), &'qy ())>,
    }
    #[derive(Clone)]
    struct Record13<'db, 'qy> {
        surname: String,
        bonus: i32,
        __internal_phantomdata: std::marker::PhantomData<(&'db (), &'qy ())>,
    }
    #[derive(Clone)]
    struct Record14<'db, 'qy> {
        bonus: i32,
        surname: String,
        __internal_phantomdata: std::marker::PhantomData<(&'db (), &'qy ())>,
    }
    #[derive(Clone)]
    pub struct Record15<'db, 'qy> {
        pub name: tables::family_bonus::Key,
        __internal_phantomdata: std::marker::PhantomData<(&'db (), &'qy ())>,
    }
    #[derive(Clone)]
    struct Record16<'db, 'qy> {
        family: tables::family_bonus::Key,
        __internal_phantomdata: std::marker::PhantomData<(&'db (), &'qy ())>,
    }
    #[derive(Clone)]
    pub struct Record17<'db, 'qy> {
        pub bonus: i32,
        pub surname: &'db String,
        __internal_phantomdata: std::marker::PhantomData<(&'db (), &'qy ())>,
    }
    #[derive(Clone)]
    pub struct Record18<'db, 'qy> {
        pub family: tables::family_bonus::Key,
        pub family_val: Record17<'db, 'qy>,
        __internal_phantomdata: std::marker::PhantomData<(&'db (), &'qy ())>,
    }
    pub struct Datastore {
        customers: tables::customers::Table,
        family_bonus: tables::family_bonus::Table,
        __internal_stats: Stats,
    }
    impl Datastore {
        pub fn new() -> Self {
            Self {
                customers: tables::customers::Table::new(1024),
                family_bonus: tables::family_bonus::Table::new(1024),
                __internal_stats: Stats::default(),
            }
        }
        pub fn db(&mut self) -> Database<'_> {
            Database {
                customers: self.customers.window(),
                family_bonus: self.family_bonus.window(),
                __internal_stats: &self.__internal_stats,
            }
        }
    }
    pub struct Database<'db> {
        customers: tables::customers::Window<'db>,
        family_bonus: tables::family_bonus::Window<'db>,
        __internal_stats: &'db Stats,
    }
    impl<'db> Database<'db> {
        pub fn customer_age_brackets<'qy>(
            &'qy mut self,
        ) -> Result<(), queries::customer_age_brackets::Error> {
            match (|__internal_self: &mut Self| {
                let (operator_closure_value_2, operator_closure_value_3) = (
                    |Record2 { ref_cust, person, .. }| {
                        (
                            Record3 {
                                bonus_points: person.bonus_points + 1,
                                __internal_phantomdata: std::marker::PhantomData,
                            },
                            Record2 {
                                ref_cust,
                                person,
                                __internal_phantomdata: std::marker::PhantomData,
                            },
                        )
                    },
                    |
                        __internal_self: &mut Self,
                        ref_cust: tables::customers::Key,
                        person: Record1<'db, 'qy>|
                    {
                        let (
                            operator_closure_value_4,
                            operator_closure_value_7,
                            operator_closure_value_9,
                        ) = (
                            Record4 {
                                surname: person.surname.clone(),
                                __internal_phantomdata: std::marker::PhantomData,
                            },
                            |Record7 { family, family_ref, surname, .. }| {
                                (
                                    Record8 {
                                        bonus: family.bonus + 1,
                                        __internal_phantomdata: std::marker::PhantomData,
                                    },
                                    Record7 {
                                        family,
                                        family_ref,
                                        surname,
                                        __internal_phantomdata: std::marker::PhantomData,
                                    },
                                )
                            },
                            Record9 {
                                __internal_phantomdata: std::marker::PhantomData,
                            },
                        );
                        let dataflow_value_4 = emdb::dependencies::minister::parallel::Parallel::consume_single(
                            operator_closure_value_4,
                        );
                        let dataflow_value_5 = {
                            let result = emdb::dependencies::minister::parallel::Parallel::consume_single(
                                emdb::dependencies::minister::parallel::Parallel::export_single(
                                    emdb::dependencies::minister::parallel::Parallel::map_single(
                                        dataflow_value_4,
                                        |dataflow_value_4| {
                                            let data = __internal_self
                                                .family_bonus
                                                .unique_surnames_cons(&dataflow_value_4.surname)?;
                                            Ok(Record5 {
                                                family_ref: data,
                                                surname: dataflow_value_4.surname,
                                                __internal_phantomdata: std::marker::PhantomData,
                                            })
                                        },
                                        &__internal_self.__internal_stats.stat_3,
                                    ),
                                ),
                            );
                            match emdb::dependencies::minister::parallel::Parallel::error_single(
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
                        let dataflow_value_6 = {
                            let result = emdb::dependencies::minister::parallel::Parallel::map_single(
                                dataflow_value_5,
                                |dataflow_value_5| {
                                    match __internal_self
                                        .family_bonus
                                        .pulpit_access_6(dataflow_value_5.family_ref)
                                    {
                                        Ok(get_value) => {
                                            Ok(Record7 {
                                                family: Record6 {
                                                    bonus: get_value.bonus,
                                                    surname: get_value.surname,
                                                    __internal_phantomdata: std::marker::PhantomData,
                                                },
                                                family_ref: dataflow_value_5.family_ref,
                                                surname: dataflow_value_5.surname,
                                                __internal_phantomdata: std::marker::PhantomData,
                                            })
                                        }
                                        Err(_) => {
                                            return Err(queries::customer_age_brackets::Error::Error6);
                                        }
                                    }
                                },
                                &__internal_self.__internal_stats.stat_4,
                            );
                            match emdb::dependencies::minister::parallel::Parallel::error_single(
                                result,
                            ) {
                                Ok(val) => {
                                    emdb::dependencies::minister::parallel::Parallel::consume_single(
                                        emdb::dependencies::minister::parallel::Parallel::export_single(
                                            val,
                                        ),
                                    )
                                }
                                Err(err) => {
                                    return Err(queries::customer_age_brackets::Error::Error6);
                                }
                            }
                        };
                        let dataflow_value_7 = {
                            let results = emdb::dependencies::minister::parallel::Parallel::map_single(
                                dataflow_value_6,
                                |dataflow_value_6| {
                                    let (update_struct, continue_struct) = operator_closure_value_7
                                        .clone()(dataflow_value_6);
                                    match __internal_self
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
                                &__internal_self.__internal_stats.stat_5,
                            );
                            emdb::dependencies::minister::parallel::Parallel::consume_single(
                                emdb::dependencies::minister::parallel::Parallel::export_single(
                                    emdb::dependencies::minister::parallel::Parallel::error_single(
                                        results,
                                    )?,
                                ),
                            )
                        };
                        let dataflow_value_8 = emdb::dependencies::minister::parallel::Parallel::consume_single(
                            operator_closure_value_9,
                        );
                        let return_value_10 = dataflow_value_8;
                        Ok(return_value_10)
                    },
                );
                let dataflow_value_0 = {
                    let stream_values = emdb::dependencies::minister::parallel::Parallel::consume_stream(
                        __internal_self.customers.get_indices(),
                    );
                    emdb::dependencies::minister::parallel::Parallel::map(
                        stream_values,
                        |value| Record0 {
                            ref_cust: value,
                            __internal_phantomdata: std::marker::PhantomData,
                        },
                        &__internal_self.__internal_stats.stat_0,
                    )
                };
                let dataflow_value_1 = {
                    let result = emdb::dependencies::minister::parallel::Parallel::map(
                        dataflow_value_0,
                        |dataflow_value_0| {
                            match __internal_self
                                .customers
                                .pulpit_access_1(dataflow_value_0.ref_cust)
                            {
                                Ok(get_value) => {
                                    Ok(Record2 {
                                        person: Record1 {
                                            surname: get_value.surname,
                                            forename: get_value.forename,
                                            bonus_points: get_value.bonus_points,
                                            age: get_value.age,
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
                        &__internal_self.__internal_stats.stat_1,
                    );
                    match emdb::dependencies::minister::parallel::Parallel::error_stream(
                        result,
                    ) {
                        Ok(val) => {
                            emdb::dependencies::minister::parallel::Parallel::consume_buffer(
                                emdb::dependencies::minister::parallel::Parallel::export_buffer(
                                    val,
                                ),
                            )
                        }
                        Err(err) => {
                            return Err(queries::customer_age_brackets::Error::Error1);
                        }
                    }
                };
                let dataflow_value_2 = {
                    let results = emdb::dependencies::minister::parallel::Parallel::map_seq(
                        dataflow_value_1,
                        |dataflow_value_1| {
                            let (update_struct, continue_struct) = operator_closure_value_2
                                .clone()(dataflow_value_1);
                            match __internal_self
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
                        &__internal_self.__internal_stats.stat_2,
                    );
                    emdb::dependencies::minister::parallel::Parallel::consume_buffer(
                        emdb::dependencies::minister::parallel::Parallel::export_buffer(
                            emdb::dependencies::minister::parallel::Parallel::error_stream(
                                results,
                            )?,
                        ),
                    )
                };
                let dataflow_value_3 = {
                    let results = emdb::dependencies::minister::parallel::Parallel::map_seq(
                        dataflow_value_2,
                        |lifted| {
                            (operator_closure_value_3)(
                                __internal_self,
                                lifted.ref_cust,
                                lifted.person,
                            )
                        },
                        &__internal_self.__internal_stats.stat_6,
                    );
                    emdb::dependencies::minister::parallel::Parallel::error_stream(
                        results,
                    )?
                };
                Ok(())
            })(self) {
                Ok(result) => {
                    {
                        self.customers.commit();
                        self.family_bonus.commit();
                    }
                    Ok(
                        emdb::dependencies::minister::parallel::Parallel::export_single(
                            result,
                        ),
                    )
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
        pub fn add_customer<'qy>(
            &'qy mut self,
            forename: String,
            surname: String,
            age: u8,
        ) -> Result<Record12, queries::add_customer::Error> {
            match (|
                __internal_self: &mut Self,
                forename: String,
                surname: String,
                age: u8|
            {
                let (operator_closure_value_12) = (Record10 {
                    forename: forename,
                    surname: surname,
                    age: age,
                    bonus_points: 0,
                    __internal_phantomdata: std::marker::PhantomData,
                });
                let dataflow_value_9 = emdb::dependencies::minister::parallel::Parallel::consume_single(
                    operator_closure_value_12,
                );
                let dataflow_value_10 = emdb::dependencies::minister::parallel::Parallel::consume_single(
                    emdb::dependencies::minister::parallel::Parallel::export_single({
                        let result = emdb::dependencies::minister::parallel::Parallel::map_single(
                            dataflow_value_9,
                            |dataflow_value_9| {
                                Ok(Record12 {
                                    name: __internal_self
                                        .customers
                                        .insert(tables::customers::insert::Insert {
                                            age: dataflow_value_9.age,
                                            bonus_points: dataflow_value_9.bonus_points,
                                            forename: dataflow_value_9.forename,
                                            surname: dataflow_value_9.surname,
                                        })?,
                                    __internal_phantomdata: std::marker::PhantomData,
                                })
                            },
                            &__internal_self.__internal_stats.stat_7,
                        );
                        match emdb::dependencies::minister::parallel::Parallel::error_single(
                            result,
                        ) {
                            Ok(val) => val,
                            Err(err) => {
                                return Err(queries::add_customer::Error::Error13(err));
                            }
                        }
                    }),
                );
                let return_value_14 = dataflow_value_10;
                Ok(return_value_14)
            })(self, forename, surname, age) {
                Ok(result) => {
                    {
                        self.customers.commit();
                    }
                    Ok(
                        emdb::dependencies::minister::parallel::Parallel::export_single(
                            result,
                        ),
                    )
                }
                Err(e) => {
                    {
                        self.customers.abort();
                    }
                    Err(e)
                }
            }
        }
        pub fn add_family<'qy>(
            &'qy mut self,
            surname: String,
        ) -> Result<Record15, queries::add_family::Error> {
            match (|__internal_self: &mut Self, surname: String| {
                let (operator_closure_value_15) = (Record13 {
                    surname: surname,
                    bonus: 0,
                    __internal_phantomdata: std::marker::PhantomData,
                });
                let dataflow_value_11 = emdb::dependencies::minister::parallel::Parallel::consume_single(
                    operator_closure_value_15,
                );
                let dataflow_value_12 = emdb::dependencies::minister::parallel::Parallel::consume_single(
                    emdb::dependencies::minister::parallel::Parallel::export_single({
                        let result = emdb::dependencies::minister::parallel::Parallel::map_single(
                            dataflow_value_11,
                            |dataflow_value_11| {
                                Ok(Record15 {
                                    name: __internal_self
                                        .family_bonus
                                        .insert(tables::family_bonus::insert::Insert {
                                            surname: dataflow_value_11.surname,
                                            bonus: dataflow_value_11.bonus,
                                        })?,
                                    __internal_phantomdata: std::marker::PhantomData,
                                })
                            },
                            &__internal_self.__internal_stats.stat_8,
                        );
                        match emdb::dependencies::minister::parallel::Parallel::error_single(
                            result,
                        ) {
                            Ok(val) => val,
                            Err(err) => {
                                return Err(queries::add_family::Error::Error16(err));
                            }
                        }
                    }),
                );
                let return_value_17 = dataflow_value_12;
                Ok(return_value_17)
            })(self, surname) {
                Ok(result) => {
                    {
                        self.family_bonus.commit();
                    }
                    Ok(
                        emdb::dependencies::minister::parallel::Parallel::export_single(
                            result,
                        ),
                    )
                }
                Err(e) => {
                    {
                        self.family_bonus.abort();
                    }
                    Err(e)
                }
            }
        }
        pub fn get_family<'qy>(
            &'qy self,
            family: tables::family_bonus::Key,
        ) -> Result<Record18, queries::get_family::Error> {
            (|__internal_self: &Self, family: tables::family_bonus::Key| {
                let (operator_closure_value_18) = (Record16 {
                    family: family,
                    __internal_phantomdata: std::marker::PhantomData,
                });
                let dataflow_value_13 = emdb::dependencies::minister::parallel::Parallel::consume_single(
                    operator_closure_value_18,
                );
                let dataflow_value_14 = {
                    let result = emdb::dependencies::minister::parallel::Parallel::map_single(
                        dataflow_value_13,
                        |dataflow_value_13| {
                            match __internal_self
                                .family_bonus
                                .pulpit_access_19(dataflow_value_13.family)
                            {
                                Ok(get_value) => {
                                    Ok(Record18 {
                                        family_val: Record17 {
                                            bonus: get_value.bonus,
                                            surname: get_value.surname,
                                            __internal_phantomdata: std::marker::PhantomData,
                                        },
                                        family: dataflow_value_13.family,
                                        __internal_phantomdata: std::marker::PhantomData,
                                    })
                                }
                                Err(_) => return Err(queries::get_family::Error::Error19),
                            }
                        },
                        &__internal_self.__internal_stats.stat_9,
                    );
                    match emdb::dependencies::minister::parallel::Parallel::error_single(
                        result,
                    ) {
                        Ok(val) => {
                            emdb::dependencies::minister::parallel::Parallel::consume_single(
                                emdb::dependencies::minister::parallel::Parallel::export_single(
                                    val,
                                ),
                            )
                        }
                        Err(err) => return Err(queries::get_family::Error::Error19),
                    }
                };
                let return_value_20 = dataflow_value_14;
                Ok(return_value_20)
            })(self, family)
                .map(emdb::dependencies::minister::parallel::Parallel::export_single)
        }
    }
    #[derive(Default)]
    struct Stats {
        stat_0: <emdb::dependencies::minister::parallel::Parallel as emdb::dependencies::minister::parallel::ParallelOps>::MapStats,
        stat_1: <emdb::dependencies::minister::parallel::Parallel as emdb::dependencies::minister::parallel::ParallelOps>::MapStats,
        stat_2: <emdb::dependencies::minister::parallel::Parallel as emdb::dependencies::minister::parallel::ParallelOps>::MapSeqStats,
        stat_3: <emdb::dependencies::minister::parallel::Parallel as emdb::dependencies::minister::parallel::ParallelOps>::MapSingleStats,
        stat_4: <emdb::dependencies::minister::parallel::Parallel as emdb::dependencies::minister::parallel::ParallelOps>::MapSingleStats,
        stat_5: <emdb::dependencies::minister::parallel::Parallel as emdb::dependencies::minister::parallel::ParallelOps>::MapSingleStats,
        stat_6: <emdb::dependencies::minister::parallel::Parallel as emdb::dependencies::minister::parallel::ParallelOps>::MapSeqStats,
        stat_7: <emdb::dependencies::minister::parallel::Parallel as emdb::dependencies::minister::parallel::ParallelOps>::MapSeqStats,
        stat_8: <emdb::dependencies::minister::parallel::Parallel as emdb::dependencies::minister::parallel::ParallelOps>::MapSeqStats,
        stat_9: <emdb::dependencies::minister::parallel::Parallel as emdb::dependencies::minister::parallel::ParallelOps>::MapSingleStats,
    }
}
