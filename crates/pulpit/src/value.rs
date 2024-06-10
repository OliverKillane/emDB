//! # Independent Mutable & Immutable Borrows
//! Using the window pattern, we can prevent mutable access to a value field,
//! while assigning the correct lifetime to borrows of the immutable field.
//!
//! | Part      | Access                                                                                    |
//! |-----------|-------------------------------------------------------------------------------------------|
//! | Immutable | Borrows can last lifetime of object, and are independent from borrows of the mutable side |
//! | Mutable   | Normal borrow rules apply                                                                 |
//!
//! This pattern is used for the [`crate::column`] interfaces.
//!
//! ## Alternative Designs
//! ### Unsafe Interface
//! Using unsafe to provide a pointer to immutable data and exposing this unsafe interface to the user (bad api design)
//!
//! ### Dummy Objects
//! Providing unbounded lifetimes, or lifetimes bound to a separate dummy object.
//!
//! ### Split Data
//! Splitting the immutable data to a separate (and immutable) data structure solves all safety
//! issues elegantly, however requiring separate allocations for the immutable data has performance
//! consequences.
//!
//! ## Pattern Implementation
//! A single data store object contains the data and is mutably borrowed by a window object.
//! - The window holds the only mutable reference, so has exclusive access.
//! - The lifetime of the window is bound to the contained mutable reference, which it can now
//!   internally use to supply lifetime bounds for borrows of the immutable part.
//!
//! ## Examples
//! ### Valid Usage
//! The following demonstrates mutable and immutable references being managed independently.
//!
//! ```
//! # use pulpit::value::{Value, ValueWindow};
//! # fn test<ImmData, MutData>(imm_data: ImmData, mut_data: MutData) {
//! let mut val = Value{imm_data, mut_data};
//! let mut window = ValueWindow::new_window(&mut val);
//! let (imm_ref, imm_mut_ref) = window.brw();
//! let mut_ref = window.brw_mut();
//!
//! let imm_available = imm_ref; // still available
//! let mut_available = mut_ref; // new mutable taken over from imm_mut_ref
//! # }
//! ```
//!
//! ### Conflicting borrows on the mutable side
//! This demonstrates borrow checking working properly still, but for the mutable member.
//!
//! ```compile_fail,E0502
//! # use pulpit::value::{Value, ValueWindow};
//! # fn test2<ImmData, MutData>(imm_data: ImmData, mut_data: MutData) {    
//! let mut val = Value{imm_data, mut_data};
//! let mut window = ValueWindow::new_window(&mut val);
//! let (imm_ref, imm_mut_ref) = window.brw();
//! let mut_ref = window.brw_mut(); // ERROR! borrow of mut_ref not possible as imm_mut_ref used later
//!
//! let value_imm = imm_ref; // still available
//! let old_mut_unavailable = imm_mut_ref;  
//! # }
//! ```
//!
//! ### No Dangling references
//! This demonstrates that the references to the immutable part are restricted to be valid by the window's mutable reference.
//! ```compile_fail,E0597
//! # use pulpit::value::{Value, ValueWindow};
//! # fn test3<ImmData, MutData>(imm_data: ImmData, mut_data: MutData) {
//! let imm_ref_dangling;
//! {
//!     let mut val = Value{imm_data, mut_data};
//!
//!     // ERROR! needs to borrow long enough for imm_ref_dangling, but col does not live that long
//!     let mut window = ValueWindow::new_window(&mut val);
//!     
//!     let (imm_ref, imm_mut_ref) = window.brw();
//!     let mut_ref = window.brw_mut();
//!     
//!     imm_ref_dangling = imm_ref;
//! }
//! let imm_ref_dangling_unavailable = imm_ref_dangling;
//! # }
//! ```

pub struct Value<ImmData, MutData> {
    pub imm_data: ImmData,
    pub mut_data: MutData,
}

pub struct ValueWindow<'imm, ImmData, MutData> {
    data: &'imm mut Value<ImmData, MutData>,
}

impl<'imm, ImmData, MutData> ValueWindow<'imm, ImmData, MutData> {
    pub fn new_window(val: &'imm mut Value<ImmData, MutData>) -> Self {
        ValueWindow { data: val }
    }

    pub fn brw<'brw>(&'brw self) -> (&'imm ImmData, &'brw MutData) {
        (
            unsafe { std::mem::transmute::<&'brw ImmData, &'imm ImmData>(&self.data.imm_data) },
            &self.data.mut_data,
        )
    }

    pub fn brw_mut(&mut self) -> &mut MutData {
        &mut self.data.mut_data
    }
}
