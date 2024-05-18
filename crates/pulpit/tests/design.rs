//! Design for a table with 5 fields in two indices
//!
//! ```
//! table! {
//!     primary {
//!              b: usize,
//!          mut a: i32,
//!          mut c: Option<String>,
//!     },
//!     assoc {
//!             d: char,
//!         mut e: String, (unique)
//!     }
//! }
//!
//! get(a,b,e) as get_abe
//! update(a, c, e) as update_ace
//! update(a) as update_a
//! delete as del,
//! insert as ins,
//! ```

mod my_table {
    use std::ops::Add;

    use pulpit::{
        self,
        column::{AssocWindow, AssocWindowPull, Column, PrimaryWindow, PrimaryWindowHide, PrimaryWindowPull},
    };

    use self::transactions::LogItem;

    mod column_types {
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
        }
    }

    type Key = <pulpit::column::PrimaryRetain<
        column_types::primary::Imm,
        column_types::primary::Mut,
        1024,
    > as pulpit::column::Keyable>::Key;

    pub mod updates {
        pub mod update_a {
            pub struct Update {
                pub a: i32,
            }
            pub type Error = pulpit::column::KeyError;
        }

        pub mod update_ace {
            pub struct Update {
                pub a: i32,
                pub c: Option<String>,
                pub e: String,
            }

            #[derive(Debug)]
            pub enum Error {
                key_error(pulpit::column::KeyError),
                unique_e(pulpit::access::UniqueConflict),
            }
        }
    }

    pub mod inserts {
        pub mod ins {
            pub struct Insert {
                pub a: i32,
                pub b: usize,
                pub c: Option<String>,
                pub d: char,
                pub e: String,
            }
            pub type Error = pulpit::access::UniqueConflict;
        }
    }

    pub mod gets {
        pub mod get_abe {
            // if borrows, add to lifetime
            pub struct Get<'imm> {
                pub a: i32,
                pub b: &'imm usize,
                pub e: String,
            }
        }

        pub mod get_a {
            pub struct Get {
                pub a: i32,
            }
        }
    }

    pub mod deletes {}

    mod transactions {
        use super::{updates, Key};

        pub enum UpdateKinds {
            update_a(updates::update_a::Update),
            update_ace(updates::update_ace::Update),
        }

        pub enum LogItem {
            Insert(Key),
            Hide(Key),
            Update { key: Key, update: UpdateKinds },
        }
    }

    struct Additional {
        transaction_log: Vec<transactions::LogItem>,
        transaction_append: bool,
        unique_e: pulpit::access::Unique<String, Key>,
    }

    struct Columns {
        primary: pulpit::column::PrimaryRetain<
            column_types::primary::Imm,
            column_types::primary::Mut,
            1024,
        >,
        assoc_0: pulpit::column::AssocVec<column_types::assoc_0::Imm, column_types::assoc_0::Mut>,
    }

    struct ColumnsWindow<'imm> {
        primary: <pulpit::column::PrimaryRetain<column_types::primary::Imm, column_types::primary::Mut, 1024> as pulpit::column::Column>::WindowKind<'imm>,
        assoc_0: <pulpit::column::AssocVec<column_types::assoc_0::Imm, column_types::assoc_0::Mut> as pulpit::column::Column>::WindowKind<'imm>,
    }

    pub struct Table {
        additional: Additional,
        columns: Columns,
    }

    impl Table {
        pub fn new(size_hint: usize) -> Self {
            Self {
                additional: Additional {
                    transaction_log: Vec::new(),
                    transaction_append: true,
                    unique_e: pulpit::access::Unique::new(size_hint),
                },
                columns: Columns {
                    primary: pulpit::column::PrimaryRetain::new(size_hint),
                    assoc_0: pulpit::column::AssocVec::new(size_hint),
                },
            }
        }

        pub fn window(&mut self) -> Window<'_> {
            Window {
                additional: &mut self.additional,
                columns: ColumnsWindow {
                    primary: self.columns.primary.window(),
                    assoc_0: self.columns.assoc_0.window(),
                },
            }
        }
    }

    pub struct Window<'imm> {
        additional: &'imm mut Additional,
        columns: ColumnsWindow<'imm>,
    }

    impl<'imm> Window<'imm> {
        pub fn ins(&mut self, arg: inserts::ins::Insert) -> Result<Key, inserts::ins::Error> {
            let unique_e_val = arg.e.clone();

            // get fields
            let (primary_imm, primary_mut, assoc_0_imm, assoc_0_mut) = {
                let inserts::ins::Insert { a, b, c, d, e } = arg;
                (
                    column_types::primary::Imm { b },
                    column_types::primary::Mut { a, c },
                    column_types::assoc_0::Imm { d },
                    column_types::assoc_0::Mut { e },
                )
            };

            // insert primary to get key
            let (key, action) = self.columns.primary.insert(pulpit::column::Data {
                imm_data: primary_imm,
                mut_data: primary_mut,
            });

            // update uniques, at each, undo previous steps
            match self.additional.unique_e.insert(unique_e_val, key) {
                Ok(()) => Ok(()),
                Err(e) => {
                    self.columns.primary.pull(key).unwrap();
                    Err(e.into())
                }
            }?;

            match action {
                pulpit::column::InsertAction::Place(index) => unsafe {
                    self.columns.assoc_0.place(
                        index,
                        pulpit::column::Data {
                            imm_data: assoc_0_imm,
                            mut_data: assoc_0_mut,
                        },
                    );
                },
                pulpit::column::InsertAction::Append => {
                    self.columns.assoc_0.append(pulpit::column::Data {
                        imm_data: assoc_0_imm,
                        mut_data: assoc_0_mut,
                    });
                }
            }

            if self.additional.transaction_append {
                self.additional
                    .transaction_log
                    .push(transactions::LogItem::Insert(key));
            }

            Ok(key)
        }

        pub fn get_abe(
            &self,
            key: Key,
        ) -> Result<gets::get_abe::Get<'imm>, pulpit::column::KeyError> {
            // generate data structures
            // access main, access assoc that are relevant
            // pass values to get.

            let pulpit::column::Entry {
                index,
                data: primarydata,
            } = self.columns.primary.get(key)?;
            let assoc_0_data = unsafe { self.columns.assoc_0.get(index) };

            {
                let pulpit::column::Data {
                    imm_data: column_types::primary::Imm { b },
                    mut_data: column_types::primary::Mut { a, c },
                } = primarydata;
                let pulpit::column::Data {
                    imm_data: column_types::assoc_0::Imm { d },
                    mut_data: column_types::assoc_0::Mut { e },
                } = assoc_0_data;

                Ok(gets::get_abe::Get { a, b, e })
            }
        }

        pub fn get_a(&self, key: Key) -> Result<gets::get_a::Get, pulpit::column::KeyError> {
            let pulpit::column::Entry {
                index,
                data: primarydata,
            } = self.columns.primary.get(key)?;

            {
                let pulpit::column::Data {
                    imm_data: column_types::primary::Imm { b },
                    mut_data: column_types::primary::Mut { a, c },
                } = primarydata;

                Ok(gets::get_a::Get { a })
            }
        }

        pub fn update_ace(&mut self, key: Key, mut update: updates::update_ace::Update) -> Result<(), updates::update_ace::Error> {
            // get fields
            
            let pulpit::column::Entry{ index, data } = match self.columns.primary.brw_mut(key) {
                Ok(acc) => Ok(acc),
                Err(e) => Err(updates::update_ace::Error::key_error(e)),
            }?;
            let assoc_0_data = unsafe { self.columns.assoc_0.brw_mut(index) };

            let unique_e_old_data = assoc_0_data.mut_data.e.clone();

            // update unique indexes
            self.additional.unique_e.pull(&unique_e_old_data).unwrap();
            match self.additional.unique_e.insert(assoc_0_data.mut_data.e.clone(), key) {
                Ok(()) => Ok(()),
                Err(e) => {
                    self.additional.unique_e.insert(unique_e_old_data, key).unwrap();
                    Err(updates::update_ace::Error::unique_e(e))
                },
            }?;

            // update values
            if self.additional.transaction_append {
                std::mem::swap(&mut data.mut_data.a, &mut update.a);
                std::mem::swap(&mut data.mut_data.c, &mut update.c);
                std::mem::swap(&mut assoc_0_data.mut_data.e, &mut update.e);
                self.additional.transaction_log.push(transactions::LogItem::Update { key, update: transactions::UpdateKinds::update_ace(update) });
            }

            Ok(())
        }

        pub fn update_a(&mut self, key: Key, mut update: updates::update_a::Update) -> Result<(), updates::update_a::Error> {
            let pulpit::column::Entry{ index, data } = match self.columns.primary.brw_mut(key) {
                Ok(acc) => Ok(acc),
                Err(e) => Err(e),
            }?;

            if self.additional.transaction_append {
                std::mem::swap(&mut data.mut_data.a, &mut update.a);
                self.additional.transaction_log.push(transactions::LogItem::Update { key, update: transactions::UpdateKinds::update_a(update) });
            }

            Ok(())
        }

        pub fn del(&mut self, key: Key) -> Result<(), pulpit::column::KeyError> {
            self.columns.primary.hide(key)?;

            // remove from unique_e



            if self.additional.transaction_append {
                self.additional.transaction_log.push(transactions::LogItem::Hide(key));
            }

            Ok(())
        }

        pub fn commit(&mut self) {
            for item in self.additional.transaction_log.iter().rev() {
                match item {
                    LogItem::Hide(key) => {
                        let pulpit::column::Entry { index, data } = self.columns.primary.pull(*key).unwrap();
                        unsafe {self.columns.assoc_0.pull(index);}
                    }
                    _ => ()
                }
            }
            self.additional.transaction_log.clear();
        }

        pub fn abort(&mut self) {
            while let Some(op) = self.additional.transaction_log.pop() {
                match op {
                    LogItem::Insert(key) => {

                    },
                    LogItem::Hide(_) => todo!(),
                    LogItem::Update { key, update } => match update {
                        transactions::UpdateKinds::update_a(data) => {self.update_a(key, data).unwrap();},
                        transactions::UpdateKinds::update_ace(data) => {self.update_ace(key, data).unwrap();},
                    },
                }
            }
        }
    }
}

fn check() {
    let mut t = my_table::Table::new(0);
    let mut w = t.window();
    
    w.ins(my_table::inserts::ins::Insert {
        a: 0,
        b: 0,
        c: None,
        d: 'a',
        e: "a".to_string(),
    }).unwrap();


    // w.update_ace(, update)

}