//  MOD.rs
//    by Lut99
//
//  Created:
//    03 May 2024, 13:42:38
//  Last edited:
//    07 May 2024, 16:38:01
//  Auto updated?
//    Yes
//
//  Description:
//!   Implements a parser for $Datalog^\neg$ using the `ast-toolkit`'s
//!   `snack`-library.
//

// Declare appropriate submodules
pub mod atoms;
pub mod literals;
pub mod rules;
pub mod tokens;

// Imports
use ast_toolkit_snack::{Combinator as _, Result as SResult};
use ast_toolkit_span::Span;

use crate::ast::Literal;


/***** ERRORS *****/
/// The concrete error type returned by the [`parse()`] function.
pub type Error<'f, 's> = ast_toolkit_snack::error::Error<'static, &'f str, &'s str, literals::ParseError<'f, 's>>;





/***** LIBRARY *****/
/// Implements a full parser of some kind of input source to an AST.
///
/// # Arguments
/// - `what`: Some kind of string describing what the input source is, e.g., `<in-memory>` or `/path/to/file`.
/// - `source`: Some kind of `'static` source string. The resulting AST will depent on it for parsing.
///
/// # Returns
/// A parsed $Datalog^\neg$-AST, starting as [`Spec`].
///
/// # Errors
/// This function returns an [`Error`] if the given `input` was not a valid $Datalog^\neg$-program.
#[inline]
pub fn parse<'f, 's>(what: &'f str, source: &'s str) -> Result<Literal<'f, 's>, Error<'f, 's>> {
    // Simply parse as a literal
    match literals::literal().parse(Span::new(what, source)) {
        SResult::Ok(_, res) => Ok(res),
        SResult::Fail(fail) => Err(fail.try_into().unwrap()),
        SResult::Error(err) => Err(err),
    }
}
