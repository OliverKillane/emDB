mod my_table {
    #![allow(unused, non_camel_case_types)]
    use pulpit::column::{
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
            pub d: &'brw char,
            pub b: &'brw usize,
            pub a: &'brw i32,
            pub c: &'brw Option<String>,
            pub e: &'brw String,
        }
    }
    impl<'imm> Window<'imm> {
        pub fn borrow<'brw>(
            &'brw self,
            key: Key,
        ) -> Result<borrows::Borrows<'brw>, KeyError> {
            let pulpit::column::Entry { index, data: primary } = match self
                .columns
                .primary
                .brw(key)
            {
                Ok(entry) => entry,
                Err(_) => return Err(KeyError),
            };
            let assoc_0 = unsafe { self.columns.assoc_0.brw(index) };
            Ok(borrows::Borrows {
                d: &assoc_0.imm_data.d,
                b: &primary.imm_data.b,
                a: &primary.mut_data.a,
                c: &primary.mut_data.c,
                e: &assoc_0.mut_data.e,
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
    impl<'imm> Window<'imm> {
        pub fn get(&self, key: Key) -> Result<get::Get<'imm>, KeyError> {
            let pulpit::column::Entry { index, data: primary } = match self
                .columns
                .primary
                .get(key)
            {
                Ok(entry) => entry,
                Err(_) => return Err(KeyError),
            };
            let primary = primary.convert_imm(column_types::primary::imm_unpack);
            let assoc_0 = unsafe { self.columns.assoc_0.get(index) }
                .convert_imm(column_types::assoc_0::imm_unpack);
            Ok(get::Get {
                d: assoc_0.imm_data.d,
                b: primary.imm_data.b,
                a: primary.mut_data.a,
                c: primary.mut_data.c,
                e: assoc_0.mut_data.e,
            })
        }
    }
    pub mod updates {
        pub mod update_ace {
            #[derive(Debug)]
            pub enum UpdateError {
                KeyError,
                e_unique,
                a_unique,
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
    impl<'imm> Window<'imm> {
        pub fn update_ace(
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
                Err(_) => return Err(updates::update_ace::UpdateError::KeyError),
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
            *(&mut primary.mut_data.a) = update.a;
            *(&mut primary.mut_data.c) = update.c;
            *(&mut assoc_0.mut_data.e) = update.e;
            Ok(())
        }
        pub fn update_a(
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
                Err(_) => return Err(updates::update_a::UpdateError::KeyError),
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
            *(&mut primary.mut_data.a) = update.a;
            Ok(())
        }
    }
    pub mod insert {
        pub struct Insert {
            pub d: char,
            pub b: usize,
            pub a: i32,
            pub c: Option<String>,
            pub e: String,
        }
        #[derive(Debug)]
        pub enum Error {
            e_unique,
            a_unique,
            check_b,
            check_e_len,
        }
    }
    impl<'imm> Window<'imm> {
        pub fn insert(
            &mut self,
            insert_val: insert::Insert,
        ) -> Result<Key, insert::Error> {
            if !predicates::check_b(borrows::Borrows {
                d: &insert_val.d,
                b: &insert_val.b,
                a: &insert_val.a,
                c: &insert_val.c,
                e: &insert_val.e,
            }) {
                return Err(insert::Error::check_b);
            }
            if !predicates::check_e_len(borrows::Borrows {
                d: &insert_val.d,
                b: &insert_val.b,
                a: &insert_val.a,
                c: &insert_val.c,
                e: &insert_val.e,
            }) {
                return Err(insert::Error::check_e_len);
            }
            let e_unique = match self.uniques.e.lookup(&insert_val.e) {
                Ok(_) => return Err(insert::Error::e_unique),
                Err(_) => insert_val.e.clone(),
            };
            let a_unique = match self.uniques.a.lookup(&insert_val.a) {
                Ok(_) => return Err(insert::Error::a_unique),
                Err(_) => insert_val.a.clone(),
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
            let key = self.columns.primary.append(primary);
            self.columns.assoc_0.append(assoc_0);
            self.uniques.e.insert(e_unique, key).unwrap();
            self.uniques.a.insert(a_unique, key).unwrap();
            Ok(key)
        }
    }
    /// The key for accessing rows (delete, update, get)
    pub type Key = <pulpit::column::PrimaryAppend<
        pulpit::column::AssocBlocks<
            column_types::primary::Imm,
            column_types::primary::Mut,
            1024usize,
        >,
    > as pulpit::column::Keyable>::Key;
    mod predicates {
        pub fn check_b(
            super::borrows::Borrows { d, b, a, c, e }: super::borrows::Borrows,
        ) -> bool {
            *b < 1045
        }
        pub fn check_e_len(
            super::borrows::Borrows { d, b, a, c, e }: super::borrows::Borrows,
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
        primary: pulpit::column::PrimaryAppend<
            pulpit::column::AssocBlocks<
                column_types::primary::Imm,
                column_types::primary::Mut,
                1024usize,
            >,
        >,
    }
    impl ColumnHolder {
        fn new(size_hint: usize) -> Self {
            Self {
                assoc_0: pulpit::column::AssocVec::new(size_hint),
                primary: pulpit::column::PrimaryAppend::new(size_hint),
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
        primary: <pulpit::column::PrimaryAppend<
            pulpit::column::AssocBlocks<
                column_types::primary::Imm,
                column_types::primary::Mut,
                1024usize,
            >,
        > as pulpit::column::Column>::WindowKind<'imm>,
    }
    pub struct Table {
        columns: ColumnHolder,
        uniques: Uniques,
    }
    impl Table {
        pub fn new(size_hint: usize) -> Self {
            Self {
                columns: ColumnHolder::new(size_hint),
                uniques: Uniques::new(size_hint),
            }
        }
        pub fn window(&mut self) -> Window<'_> {
            Window {
                columns: self.columns.window(),
                uniques: &mut self.uniques,
            }
        }
    }
    pub struct Window<'imm> {
        columns: WindowHolder<'imm>,
        uniques: &'imm mut Uniques,
    }
}
