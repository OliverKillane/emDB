//! ## Checking Types
//! Generates required static assertions (e.g. for types)
//! - [plan::GroupBy], [plan::Unique] require inputs that are hashable and comparable.
//! - [plan::Sort] requires comparable fields.

// use crate::plan;
