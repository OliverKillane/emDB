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
                pub b: &'imm usize,
            }
            pub fn imm_unpack<'imm>(Imm { b }: &'imm Imm) -> ImmUnpack<'imm> {
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
                pub d: char,
            }
            pub fn imm_unpack(Imm { d }: Imm) -> ImmUnpack {
                ImmUnpack { d }
            }
        }
    }
    pub mod borrows {
        pub struct Borrows<'brw> {
            pub a: &'brw i32,
            pub b: &'brw usize,
            pub e: &'brw String,
            pub d: &'brw char,
            pub c: &'brw Option<String>,
        }
    }
    pub trait Borrow {
        /// Gets an immutable borrow of all fields.
        fn borrow<'brw>(
            &'brw self,
            key: Key,
        ) -> Result<borrows::Borrows<'brw>, KeyError>;
    }
    impl<'imm> Borrow for Window<'imm> {
        fn borrow<'brw>(
            &'brw self,
            key: Key,
        ) -> Result<borrows::Borrows<'brw>, KeyError> {
            let pulpit::column::Entry { index, data: primary } = match self
                .columns
                .primary
                .brw(key)
            {
                Ok(entry) => entry,
                Err(e) => return Err(KeyError),
            };
            let assoc_0 = unsafe { self.columns.assoc_0.brw(index) };
            Ok(borrows::Borrows {
                a: &primary.mut_data.a,
                b: &primary.imm_data.b,
                e: &assoc_0.mut_data.e,
                d: &assoc_0.imm_data.d,
                c: &primary.mut_data.c,
            })
        }
    }
    pub mod get {
        pub struct Get<'imm> {
            pub a: i32,
            pub c: Option<String>,
            pub b: &'imm usize,
            pub e: String,
            pub d: char,
        }
    }
    pub trait Get<'imm> {
        fn get(&self, key: Key) -> Result<get::Get<'imm>, KeyError>;
    }
    impl<'imm> Get<'imm> for Window<'imm> {
        fn get(&self, key: Key) -> Result<get::Get<'imm>, KeyError> {
            let pulpit::column::Entry { index, data: primary } = match self
                .columns
                .primary
                .get(key)
            {
                Ok(entry) => entry,
                Err(e) => return Err(KeyError),
            };
            let primary = primary.convert_imm(column_types::primary::imm_unpack);
            let assoc_0 = unsafe { self.columns.assoc_0.get(index) }
                .convert_imm(column_types::assoc_0::imm_unpack);
            Ok(get::Get {
                a: primary.mut_data.a,
                b: primary.imm_data.b,
                e: assoc_0.mut_data.e,
                d: assoc_0.imm_data.d,
                c: primary.mut_data.c,
            })
        }
    }
    pub mod updates {
        pub mod update_ace {
            #[derive(Debug)]
            pub enum UpdateError {
                KeyError,
                a_unique,
                e_unique,
                check_b,
                check_e_len,
            }
            pub struct Update {
                pub a: i32,
                pub c: Option<String>,
                pub e: String,
            }
        }
        pub mod update_a {
            #[derive(Debug)]
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
            if !predicates::check_b(borrows::Borrows {
                b: &primary.imm_data.b,
                a: &update.a,
                c: &update.c,
                d: &assoc_0.imm_data.d,
                e: &update.e,
            }) {
                return Err(updates::update_ace::UpdateError::check_b);
            }
            if !predicates::check_e_len(borrows::Borrows {
                b: &primary.imm_data.b,
                a: &update.a,
                c: &update.c,
                d: &assoc_0.imm_data.d,
                e: &update.e,
            }) {
                return Err(updates::update_ace::UpdateError::check_e_len);
            }
            let a_unique = match self
                .uniques
                .a
                .replace(&update.a, &primary.mut_data.a, key)
            {
                Ok(old_val) => old_val,
                Err(_) => return Err(updates::update_ace::UpdateError::a_unique),
            };
            let e_unique = match self
                .uniques
                .e
                .replace(&update.e, &assoc_0.mut_data.e, key)
            {
                Ok(old_val) => old_val,
                Err(_) => {
                    self.uniques.a.undo_replace(a_unique, &update.a, key);
                    return Err(updates::update_ace::UpdateError::e_unique);
                }
            };
            let mut update = update;
            std::mem::swap(&mut primary.mut_data.a, &mut update.a);
            std::mem::swap(&mut primary.mut_data.c, &mut update.c);
            std::mem::swap(&mut assoc_0.mut_data.e, &mut update.e);
            if !self.transactions.rollback {
                self.transactions
                    .log
                    .push(
                        transactions::LogItem::Update(
                            key,
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
            if !predicates::check_b(borrows::Borrows {
                b: &primary.imm_data.b,
                a: &update.a,
                c: &primary.mut_data.c,
                d: &assoc_0.imm_data.d,
                e: &assoc_0.mut_data.e,
            }) {
                return Err(updates::update_a::UpdateError::check_b);
            }
            if !predicates::check_e_len(borrows::Borrows {
                b: &primary.imm_data.b,
                a: &update.a,
                c: &primary.mut_data.c,
                d: &assoc_0.imm_data.d,
                e: &assoc_0.mut_data.e,
            }) {
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
                            key,
                            transactions::Updates::update_a(update),
                        ),
                    );
            }
            Ok(())
        }
    }
    pub mod insert {
        /// TODO
        pub struct Insert {
            pub a: i32,
            pub b: usize,
            pub e: String,
            pub d: char,
            pub c: Option<String>,
        }
        /// TODO
        #[derive(Debug)]
        pub enum Error {
            a_unique,
            e_unique,
            check_b,
            check_e_len,
        }
    }
    pub trait Insert {
        fn insert(&mut self, insert_val: insert::Insert) -> Result<Key, insert::Error>;
    }
    impl<'imm> Insert for Window<'imm> {
        fn insert(&mut self, insert_val: insert::Insert) -> Result<Key, insert::Error> {
            if !predicates::check_b(borrows::Borrows {
                a: &insert_val.a,
                b: &insert_val.b,
                e: &insert_val.e,
                d: &insert_val.d,
                c: &insert_val.c,
            }) {
                return Err(insert::Error::check_b);
            }
            if !predicates::check_e_len(borrows::Borrows {
                a: &insert_val.a,
                b: &insert_val.b,
                e: &insert_val.e,
                d: &insert_val.d,
                c: &insert_val.c,
            }) {
                return Err(insert::Error::check_e_len);
            }
            let a_unique = match self.uniques.a.lookup(&insert_val.a) {
                Ok(_) => return Err(insert::Error::a_unique),
                Err(_) => insert_val.a.clone(),
            };
            let e_unique = match self.uniques.e.lookup(&insert_val.e) {
                Ok(_) => return Err(insert::Error::e_unique),
                Err(_) => insert_val.e.clone(),
            };
            let primary = (pulpit::column::Data {
                imm_data: column_types::primary::Imm {
                    b: insert_val.b,
                },
                mut_data: column_types::primary::Mut {
                    a: insert_val.a,
                    c: insert_val.c,
                },
            });
            let assoc_0 = (pulpit::column::Data {
                imm_data: column_types::assoc_0::Imm {
                    d: insert_val.d,
                },
                mut_data: column_types::assoc_0::Mut {
                    e: insert_val.e,
                },
            });
            let (key, action) = self.columns.primary.insert(primary);
            match action {
                pulpit::column::InsertAction::Place(index) => {
                    unsafe {
                        self.columns.assoc_0.place(index, assoc_0);
                    }
                }
                pulpit::column::InsertAction::Append => {
                    self.columns.assoc_0.append(assoc_0);
                }
            }
            self.uniques.a.insert(a_unique, key).unwrap();
            self.uniques.e.insert(e_unique, key).unwrap();
            if !self.transactions.rollback {
                self.transactions.log.push(transactions::LogItem::Insert(key));
            }
            Ok(key)
        }
    }
    mod transactions {
        ///TODO
        pub enum Updates {
            update_ace(super::updates::update_ace::Update),
            update_a(super::updates::update_a::Update),
        }
        /// TODO
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
    pub trait Transact {
        fn commit(&mut self);
        fn abort(&mut self);
    }
    impl<'imm> Transact for Window<'imm> {
        /// Commit all current changes
        /// - Requires concretely applying deletions (which until commit
        ///   or abort simply hide keys from the table)
        fn commit(&mut self) {
            debug_assert!(! self.transactions.rollback);
            while let Some(entry) = self.transactions.log.pop() {
                match entry {
                    transactions::LogItem::Delete(key) => {
                        let pulpit::column::Entry { index, data: _ } = self
                            .columns
                            .primary
                            .pull(key)
                            .unwrap();
                        unsafe {
                            self.columns.assoc_0.pull(index);
                        }
                    }
                    _ => {}
                }
            }
        }
        /// Undo the transactions applied since the last commit
        /// - Requires re-applying all updates, deleting inserts and undoing deletes
        ///   (deletes' keys are actually just hidden until commit or abort)
        fn abort(&mut self) {
            self.transactions.rollback = true;
            while let Some(entry) = self.transactions.log.pop() {
                match entry {
                    transactions::LogItem::Delete(key) => {
                        self.columns.primary.reveal(key).unwrap();
                    }
                    transactions::LogItem::Insert(key) => {
                        let pulpit::column::Entry { index, data: _ } = self
                            .columns
                            .primary
                            .pull(key)
                            .unwrap();
                        unsafe {
                            self.columns.assoc_0.pull(index);
                        }
                    }
                    transactions::LogItem::Update(key, update) => {
                        match update {
                            transactions::Updates::update_ace(update) => {
                                <Self as Update>::update_ace(self, update, key).unwrap();
                            }
                            transactions::Updates::update_a(update) => {
                                <Self as Update>::update_a(self, update, key).unwrap();
                            }
                        }
                    }
                }
            }
            self.transactions.rollback = false;
        }
    }
    mod delete {}
    pub trait Delete {
        fn delete(&mut self, key: Key) -> Result<(), KeyError>;
    }
    impl<'imm> Delete for Window<'imm> {
        fn delete(&mut self, key: Key) -> Result<(), KeyError> {
            match self.columns.primary.hide(key) {
                Ok(()) => {}
                Err(e) => return Err(KeyError),
            }
            if !self.transactions.rollback {
                self.transactions.log.push(transactions::LogItem::Delete(key));
            }
            Ok(())
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
            super::borrows::Borrows { a, b, e, d, c }: super::borrows::Borrows,
        ) -> bool {
            *b < 1045
        }
        pub fn check_e_len(
            super::borrows::Borrows { a, b, e, d, c }: super::borrows::Borrows,
        ) -> bool {
            e.len() > *b
        }
    }
    struct Uniques {
        a: pulpit::access::Unique<i32, Key>,
        e: pulpit::access::Unique<String, Key>,
    }
    impl Uniques {
        fn new(size_hint: usize) -> Self {
            Self {
                a: pulpit::access::Unique::new(size_hint),
                e: pulpit::access::Unique::new(size_hint),
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
