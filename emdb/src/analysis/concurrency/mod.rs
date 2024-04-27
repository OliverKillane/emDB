//! # Concurrency Analysis
//! To determine which queries conflict, and how.
//! 
//! Each query can have a conflict with all others (including itself).
//! - Conflicts occur when one query writes to the same data
//! - Reading the same data, or reading different sets results in no conflict.

