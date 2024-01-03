//! An easily extensible function combinator library

use std::fmt::{Display, Error, Formatter};

pub mod core;
pub mod derived;
pub mod macros;

// #[cfg(feature = "tokens")]
pub mod tokens;

#[derive(PartialEq, Eq, Debug)]
pub enum CombiResult<S, C, E> {
    Suc(S),
    Con(C),
    Err(E),
}

/// The core trait for defining combinable computations
#[doc=include_str!("../docs/combi_trait.drawio.svg")]
/// Provides an interface to run computations, and get a representation of them.
#[cfg_attr(
    feature = "nightly",
    rustc_on_unimplemented(
        message = "`{Self}` is not a `Combi` combinator so cannot be combined & used as one",
        label = "Not `Combi`",
    )
)]
pub trait Combi {
    type Suc;
    type Err;
    type Con;

    type Inp;
    type Out;

    /// Runs the computation represented by [Combi] to produce a result (of the computation), and the output to be further computed on.
    #[allow(clippy::type_complexity)]
    fn comp(&self, input: Self::Inp) -> (Self::Out, CombiResult<Self::Suc, Self::Con, Self::Err>);

    /// Produces a representation of the combinator for debugging & generating error messages.
    fn repr(&self, f: &mut Formatter<'_>) -> Result<(), Error>;
}

/// Allows a [continuation](Combi::Con) to be combined with subsequent [continuations](Combi::Con) and [successes](Combi::Suc).
/// ```ignore
/// // from some combinators we get:
/// let (c1, s2, s3, c4) = ...;
///
/// // and can combine in order:
/// let c_total = c1.combine_suc(s2).combine_suc(s3).combine_con(c4)
/// ```
#[cfg_attr(
    feature = "nightly",
    rustc_on_unimplemented(
        message = "`{Self}` cannot be combined with outputs of type `{Suc}` or other continuations of type `{Con}`",
        label = "Not `CombiCon` combinable",
    )
)]
pub trait CombiCon<Suc, Con> {
    fn combine_suc(self, suc: Suc) -> Self;
    fn combine_con(self, con: Con) -> Self;
}

/// Allows an error to inherit the previous [continuation](Combi::Con)
/// ```ignore
/// // from some combinators we get:
/// let (c1, s2, o3, e4) = ...;
///
/// // and can combine in order:
/// let c_total = c1.combine_suc(s2).combine_out(o3);
///
/// // but we have reached an error, and want to keep the context of continuations:
/// let e_final = e4.inherit_con(c_total);
/// ```
///
/// Also allow for [continuations](Combi::Con) to be caught as errors (for example in [core::Choice], where the branch is know from a [success](Combi::Suc)).
#[cfg_attr(
    feature = "nightly",
    rustc_on_unimplemented(
        message = "`{Self}` cannot be inherit from continuations of type `{Con}` to build errors with their context",
        label = "Cannot inherit a continuation",
    )
)]
pub trait CombiErr<Con> {
    fn inherit_con(self, con: Con) -> Self;
    fn catch_con(con: Con) -> Self;
}

/// A simple wrapper to allow the [Combi::repr] function to implement [Display]
pub struct Repr<T>(T);

impl<C: Combi> Display for Repr<&C> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.repr(f)
    }
}

impl<I, O, S, C, E> Display for Repr<&dyn Combi<Inp = I, Out = O, Suc = S, Con = C, Err = E>> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.repr(f)
    }
}
