mod my_table {
    use pulpit::column::{
        PrimaryWindow, PrimaryWindowApp, PrimaryWindowPull, PrimaryWindowHide,
        AssocWindow, AssocWindowPull, Column,
    };
    pub struct KeyError;
    mod column_types {
        //! Column types to be used for storage in each column.
        pub mod primary {
            #[derive(Clone)]
            pub struct Imm {
                pub b: usize,
            }
            #[derive(Clone)]
            pub struct Mut {
                pub a: i32,
                pub c: Option<String>,
            }
            pub struct ImmUnpack<'imm> {
                b: &'imm usize,
            }
            fn imm_unpack<'imm>(Imm { b }: &'imm Imm) -> ImmUnpack<'imm> {
                ImmUnpack { b }
            }
        }
        pub mod assoc_0 {
            #[derive(Clone)]
            pub struct Imm {
                pub d: char,
            }
            #[derive(Clone)]
            pub struct Mut {
                pub e: String,
            }
            pub struct ImmUnpack {
                d: char,
            }
            fn imm_unpack(Imm { d }: Imm) -> ImmUnpack {
                ImmUnpack { d }
            }
        }
    }
    pub mod borrow {
        pub struct Borrow<'brw> {
            a: &'brw i32,
            e: &'brw String,
            c: &'brw Option<String>,
            b: &'brw usize,
            d: &'brw char,
        }
    }
    pub trait Borrow {
        fn borrow<'brw>(&'brw self, key: Key) -> Result<borrow::Borrow<'brw>, KeyError>;
    }
    impl<'imm> Borrow for Window<'imm> {
        fn borrow<'brw>(&'brw self, key: Key) -> Result<borrow::Borrow<'brw>, KeyError> {
            todo!()
        }
    }
    pub mod get {
        /// TODO
        pub struct Get<'imm> {}
    }
    pub trait Get<'imm> {
        fn get(&self, key: Key) -> Result<get::Get<'imm>, KeyError>;
    }
    impl<'imm> Get<'imm> for Window<'imm> {
        fn get(&self, key: Key) -> Result<get::Get<'imm>, KeyError> {
            todo!()
        }
    }
    pub mod updates {
        pub mod update_ace {
            pub enum UpdateError {
                KeyError,
                e_unique,
                a_unique,
                check_b,
                check_e_len,
            }
            pub struct Update {
                pub e: String,
                pub a: i32,
                pub c: Option<String>,
            }
        }
        pub mod update_a {
            pub enum UpdateError {
                KeyError,
                a_unique,
                check_b,
                check_e_len,
            }
            pub struct Update {
                pub a: i32,
            }
        }
    }
    pub trait Update: Sized {
        fn update_ace(
            &mut self,
            update: updates::update_ace::Update,
            key: Key,
        ) -> Result<(), updates::update_ace::UpdateError>;
        fn update_a(
            &mut self,
            update: updates::update_a::Update,
            key: Key,
        ) -> Result<(), updates::update_a::UpdateError>;
    }
    impl<'imm> Update for Window<'imm> {
        fn update_ace(
            &mut self,
            update: updates::update_ace::Update,
            key: Key,
        ) -> Result<(), updates::update_ace::UpdateError> {
            let pulpit::column::Entry { index, data: primary } = match self
                .columns
                .primary
                .brw_mut(key)
            {
                Ok(entry) => entry,
                Err(e) => return Err(updates::update_ace::UpdateError::KeyError),
            };
            let assoc_0 = unsafe { self.columns.assoc_0.brw_mut(index) };
            if !predicates::check_b(
                &primary.imm_data.b,
                &update.a,
                &update.c,
                &assoc_0.imm_data.d,
                &update.e,
            ) {
                return Err(updates::update_ace::UpdateError::check_b);
            }
            if !predicates::check_e_len(
                &primary.imm_data.b,
                &update.a,
                &update.c,
                &assoc_0.imm_data.d,
                &update.e,
            ) {
                return Err(updates::update_ace::UpdateError::check_e_len);
            }
            let e_unique = match self
                .uniques
                .e
                .replace(&update.e, &assoc_0.mut_data.e, key)
            {
                Ok(old_val) => old_val,
                Err(_) => return Err(updates::update_ace::UpdateError::e_unique),
            };
            let a_unique = match self
                .uniques
                .a
                .replace(&update.a, &primary.mut_data.a, key)
            {
                Ok(old_val) => old_val,
                Err(_) => {
                    self.uniques.e.undo_replace(e_unique, &update.e, key);
                    return Err(updates::update_ace::UpdateError::a_unique);
                }
            };
            let mut update = update;
            std::mem::swap(&mut assoc_0.mut_data.e, &mut update.e);
            std::mem::swap(&mut primary.mut_data.a, &mut update.a);
            std::mem::swap(&mut primary.mut_data.c, &mut update.c);
            if !self.transactions.rollback {
                self.transactions
                    .log
                    .push(
                        transactions::LogItem::Update(
                            transactions::Updates::update_ace(update),
                        ),
                    );
            }
            Ok(())
        }
        fn update_a(
            &mut self,
            update: updates::update_a::Update,
            key: Key,
        ) -> Result<(), updates::update_a::UpdateError> {
            let pulpit::column::Entry { index, data: primary } = match self
                .columns
                .primary
                .brw_mut(key)
            {
                Ok(entry) => entry,
                Err(e) => return Err(updates::update_a::UpdateError::KeyError),
            };
            let assoc_0 = unsafe { self.columns.assoc_0.brw_mut(index) };
            if !predicates::check_b(
                &primary.imm_data.b,
                &update.a,
                &primary.mut_data.c,
                &assoc_0.imm_data.d,
                &assoc_0.mut_data.e,
            ) {
                return Err(updates::update_a::UpdateError::check_b);
            }
            if !predicates::check_e_len(
                &primary.imm_data.b,
                &update.a,
                &primary.mut_data.c,
                &assoc_0.imm_data.d,
                &assoc_0.mut_data.e,
            ) {
                return Err(updates::update_a::UpdateError::check_e_len);
            }
            let a_unique = match self
                .uniques
                .a
                .replace(&update.a, &primary.mut_data.a, key)
            {
                Ok(old_val) => old_val,
                Err(_) => return Err(updates::update_a::UpdateError::a_unique),
            };
            let mut update = update;
            std::mem::swap(&mut primary.mut_data.a, &mut update.a);
            if !self.transactions.rollback {
                self.transactions
                    .log
                    .push(
                        transactions::LogItem::Update(
                            transactions::Updates::update_a(update),
                        ),
                    );
            }
            Ok(())
        }
    }
    pub mod insert {
        /// TODO
        pub struct Insert {}
        /// TODO
        pub enum Error {}
    }
    pub trait Insert {
        fn get(&self, insert: insert::Insert) -> Result<Key, insert::Error>;
    }
    impl<'imm> Insert for Window<'imm> {
        fn get(&self, insert: insert::Insert) -> Result<Key, insert::Error> {
            todo!()
        }
    }
    mod transactions {
        ///TODO
        pub enum Updates {}
        /// TODO
        pub enum LogItem {}
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
    pub trait Transact {
        fn commit(&mut self);
        fn abort(&mut self);
    }
    impl<'imm> Transact for Window<'imm> {
        fn commit(&mut self) {
            todo!()
        }
        fn abort(&mut self) {
            todo!()
        }
    }
    mod delete {}
    pub trait Delete {
        fn delete(&mut self, key: Key) -> Result<(), KeyError>;
    }
    impl<'imm> Delete for Window<'imm> {
        fn delete(&mut self, key: Key) -> Result<(), KeyError> {
            todo!()
        }
    }
    /// The key for accessing rows (delete, update, get)
    pub type Key = <pulpit::column::PrimaryRetain<
        column_types::primary::Imm,
        column_types::primary::Mut,
        1024usize,
    > as pulpit::column::Keyable>::Key;
    mod predicates {
        pub fn check_b(
            b: &usize,
            a: &i32,
            c: &Option<String>,
            d: &char,
            e: &String,
        ) -> bool {
            *b < 1045
        }
        pub fn check_e_len(
            b: &usize,
            a: &i32,
            c: &Option<String>,
            d: &char,
            e: &String,
        ) -> bool {
            e.len() > *b
        }
    }
    struct Uniques {
        e: pulpit::access::Unique<String, Key>,
        a: pulpit::access::Unique<i32, Key>,
    }
    impl Uniques {
        fn new(size_hint: usize) -> Self {
            Self {
                e: pulpit::access::Unique::new(size_hint),
                a: pulpit::access::Unique::new(size_hint),
            }
        }
    }
    struct ColumnHolder {
        assoc_0: pulpit::column::AssocVec<
            column_types::assoc_0::Imm,
            column_types::assoc_0::Mut,
        >,
        primary: pulpit::column::PrimaryRetain<
            column_types::primary::Imm,
            column_types::primary::Mut,
            1024usize,
        >,
    }
    impl ColumnHolder {
        fn new(size_hint: usize) -> Self {
            Self {
                assoc_0: pulpit::column::AssocVec::new(size_hint),
                primary: pulpit::column::PrimaryRetain::new(size_hint),
            }
        }
        fn window(&mut self) -> WindowHolder<'_> {
            WindowHolder {
                assoc_0: self.assoc_0.window(),
                primary: self.primary.window(),
            }
        }
    }
    struct WindowHolder<'imm> {
        assoc_0: <pulpit::column::AssocVec<
            column_types::assoc_0::Imm,
            column_types::assoc_0::Mut,
        > as pulpit::column::Column>::WindowKind<'imm>,
        primary: <pulpit::column::PrimaryRetain<
            column_types::primary::Imm,
            column_types::primary::Mut,
            1024usize,
        > as pulpit::column::Column>::WindowKind<'imm>,
    }
    pub struct Table {
        columns: ColumnHolder,
        uniques: Uniques,
        transactions: transactions::Data,
    }
    impl Table {
        fn new(size_hint: usize) -> Self {
            Self {
                columns: ColumnHolder::new(size_hint),
                uniques: Uniques::new(size_hint),
                transactions: transactions::Data::new(),
            }
        }
        fn window(&mut self) -> Window<'_> {
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
