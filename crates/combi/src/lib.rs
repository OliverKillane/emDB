//! An easily extensible function combinator library
#![allow(internal_features)]
#![cfg_attr(feature = "nightly", feature(rustc_attrs))]
// TODO: Add further verification to ensure
#![warn(clippy::style)]
#![warn(clippy::perf)]
#![warn(clippy::cargo)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::panic)]

use std::fmt::{Display, Error, Formatter};

pub mod core;
pub mod derived;
pub mod logical;
pub mod macros;

pub mod text;
pub mod tokens;

/// The result of [Combi] computation
#[derive(PartialEq, Eq, Debug, Clone)]
pub enum CombiResult<S, C, E> {
    Suc(S),
    Con(C),
    Err(E),
}

impl<S, E> CombiResult<S, E, E> {
    /// When the error and continuation types are identical, we can convert a result into an regular rust [`Result`]
    pub fn to_result(self) -> Result<S, E> {
        match self {
            CombiResult::Suc(s) => Ok(s),
            CombiResult::Con(e) | CombiResult::Err(e) => Err(e),
        }
    }
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
/// ```
/// # use combi::{Combi, CombiErr, CombiCon, CombiResult};
/// # struct NothingCont;
/// # impl CombiCon<(), NothingCont> for NothingCont {
/// #     fn combine_suc(self, _: ()) -> Self {
/// #        NothingCont
/// #    }
/// #    fn combine_con(self, con: NothingCont) -> Self {
/// #        NothingCont
/// #    }
/// # }
/// # let results = (NothingCont, (), (), NothingCont);
/// // from some combinators we get:
/// let (cont_1, suc_2, suc_3, cont_4) = results;
///
/// // and can combine in order:
/// let c_total = cont_1.combine_suc(suc_2).combine_suc(suc_3).combine_con(cont_4);
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
/// ```
/// # use combi::{Combi, CombiErr, CombiCon, CombiResult};
/// # struct NothingErr;
/// # struct NothingCont;
/// # impl CombiErr<NothingCont> for NothingErr {
/// #     fn inherit_con(self, con: NothingCont) -> Self {
/// #        NothingErr
/// #    }
/// #    fn catch_con(con: NothingCont) -> Self {
/// #        NothingErr
/// #    }
/// # }
/// # impl CombiCon<(), NothingCont> for NothingCont {
/// #     fn combine_suc(self, _: ()) -> Self {
/// #        NothingCont
/// #    }
/// #    fn combine_con(self, con: NothingCont) -> Self {
/// #        NothingCont
/// #    }
/// # }
/// # let results = (NothingCont, (), NothingCont, NothingErr);
/// // we can take continuations, successes and errors:
/// let (cont_1, suc_2, cont_3, error_4) = results;
///
/// // and can combine in order, propagating context:
/// let c_total = cont_1.combine_suc(suc_2).combine_con(cont_3);
///
/// // but we have reached an error, and want to keep the context of continuations:
/// let e_final = error_4.inherit_con(c_total);
/// ```
/// Also allow for [continuations](Combi::Con) to be caught as errors (for example in [`core::choice`], where the branch is know from a [success](Combi::Suc)).
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
pub struct Repr<T>(pub T);

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
