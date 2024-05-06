//! # The Independent Borrow Pattern
//! The purpose of this pattern is to allow parts data structures to be split in two parts:
//!
//! | Part      | Access                                                                                    |
//! |-----------|-------------------------------------------------------------------------------------------|
//! | Immutable | Borrows can last lifetime of object, and are independent from borrows of the mutable side |
//! | Mutable   | Normal borrow rules apply                                                                 |
//!
//! This can allow substantial performance improvement for append only data structures.
//!
//! ## Alternatives
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
//! Here we demonstate the immutable data lives longer than the borrow from the `Window` but 
//! not longer than the data itself.
//! 
//! ```compile_fail,E0597
//! mod ind_brw {
//!     use std::mem::transmute;
//!     pub struct Data<DImm, DMut> {
//!         imm_data: DImm,
//!         mut_data: DMut,
//!     }
//! 
//!     impl<DImm, DMut> Data<DImm, DMut> {
//!         pub fn new(dimm: DImm, dmut: DMut) -> Self {
//!             Data {
//!                 imm_data: dimm,
//!                 mut_data: dmut,
//!             }
//!         }
//! 
//!         pub fn interface(&mut self) -> Window<'_, DImm, DMut> {
//!             Window::new(self)
//!         }
//!     }
//! 
//!     pub struct Window<'imm, DImm, DMut> {
//!         data: &'imm mut Data<DImm, DMut>,
//!     }
//! 
//!     impl<'imm, DImm, DMut> Window<'imm, DImm, DMut> {
//!         fn new(data: &'imm mut Data<DImm, DMut>) -> Self {
//!             Window { data }
//!         }
//! 
//!         pub fn imm_data<'brw>(&'brw self) -> &'imm DImm {
//!             unsafe {
//!                 transmute(&self.data.imm_data) // extend lifetime to immutable borrow
//!             }
//!         }
//! 
//!         pub fn mut_data(&mut self) -> &mut DMut {
//!             &mut self.data.mut_data
//!         }
//!     }
//! }
//! 
//! fn test() {
//!     use ind_brw::*;
//! 
//!     let x_glob;
//!     {
//!         let mut data = Data::new(3, 4);
//!         {
//!             let mut interface = data.interface();
//!             let x = interface.imm_data();
//!             let y = interface.mut_data();
//!             *y += *x;
//!             x_glob = x;
//!         }
//!         let z_allowed = x_glob;
//!     }
//!     let z_not_alive = x_glob; // fails as the data does not live to this point.
//! }
//! ```

