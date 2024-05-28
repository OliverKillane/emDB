//! # Minister
//! A library operating on streams, with integrations for [`pulpit`].
//! 
//! ## Performance Considerations
//! ### Cardinality
//! Tracked by streams to allow for buffers to be allocated without a need to 
//! extend.
//! 
//! ### Parallelism
//! Depending on the stream type, and the operation being performed, parallelism 
//! can be extracted.
//! - Using the [`rayon`] library for parallel iterators 

