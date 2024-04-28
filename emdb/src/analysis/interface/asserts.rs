//! ## Checking Types
//! Generates required static assertions (e.g. for types)
//! - [crate::plan::GroupBy], [crate::plan::Unique] require inputs that are hashable and comparable.
//! - [crate::plan::Sort] requires comparable fields.

// use crate::plan;
