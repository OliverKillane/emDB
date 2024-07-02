//! # Optimisations to mutate and improve the plan.
//! ```ignore
//! type optimisation = fn(&mut plan::Plan);
//! ```
//! 
//! ## Difficulties
//! The current [`crate::plan::Plan`] includes arbitrary rust expressions.
//! - Re-ordering nodes needs to consider the `map` operator applying any expression 
//! - Analysing rust expressions is difficult, and `map` uses these (with no analysis of inner expression)
//! 
//! ## Proposed Optimisations
//! 1. Push limit through map
//! 2. Split table based on predicates (e.g. for optionals, or a filter predicate)
