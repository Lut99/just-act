//  TOKENS.rs
//    by Lut99
//
//  Created:
//    18 Mar 2024, 12:04:42
//  Last edited:
//    07 May 2024, 08:36:39
//  Auto updated?
//    Yes
//
//  Description:
//!   Defines parsers for $Datalog^\neg$ keywords.
//

use std::convert::Infallible;
use std::error::Error;
use std::fmt::{Display, Formatter, Result as FResult};

use ast_toolkit_snack::combinator::{self as comb, Map, MapErr};
use ast_toolkit_snack::sequence::{self as seq, Tuple};
use ast_toolkit_snack::utf8::complete::{self as utf8, Tag};
use ast_toolkit_snack::Combinator;
use ast_toolkit_span::Spanning as _;

use crate::ast::{self, Parens};


/***** TYPE ALIASES *****/
/// Convenience alias for a [`Span`](ast_toolkit_span::Span) over static strings.
type Span = ast_toolkit_span::Span<&'static str, &'static str>;





/***** ERRORS *****/
/// Errors returned when parsing [`parens()`].
#[derive(Debug)]
pub enum ParensParseError<E> {
    /// Failed to parse the opening parenthesis.
    Open { span: Span },
    /// Something went wrong in the middle.
    Middle { err: E },
    /// Failed to parse the closing parenthesis.
    Close { span: Span },
}
impl<E: Display> Display for ParensParseError<E> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Open { .. } => write!(f, "Expected an opening parenthesis"),
            Self::Middle { err } => write!(f, "{err}"),
            Self::Close { .. } => write!(f, "Expected a closing parenthesis"),
        }
    }
}
impl<E: 'static + Error> Error for ParensParseError<E> {
    #[inline]
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Open { .. } => None,
            Self::Middle { err } => Some(err),
            Self::Close { .. } => None,
        }
    }
}





/***** LIBRARY FUNCTIONS *****/
/// Combinator for parsing a `:-`.
///
/// # Returns
/// A combinator that parses [`Arrow`]s.
///
/// # Fails
/// The returned combinator fails if the input is not `,`.
///
/// # Example
/// ```rust
/// use ast_toolkit_snack::error::{Common, Failure};
/// use ast_toolkit_snack::{Combinator as _, Result as SResult};
/// use ast_toolkit_span::Span;
/// use datalog::ast::Arrow;
/// use datalog::parser::tokens::arrow;
///
/// let span1 = Span::new("<example>", ":-");
/// let span2 = Span::new("<example>", "foo");
///
/// let mut comb = arrow();
/// assert_eq!(comb.parse(span1).unwrap(), (span1.slice(2..), Arrow { span: span1.slice(..2) }));
/// assert!(matches!(comb.parse(span2), SResult::Fail(Failure::Common(Common::TagUtf8 { .. }))));
/// ```
pub const fn arrow() -> Map<&'static str, &'static str, Tag<'static, &'static str, &'static str>, fn(Span) -> ast::Arrow> {
    comb::map(utf8::tag(":-"), |span| ast::Arrow { span })
}

/// Combinator for parsing a `,`.
///
/// # Returns
/// A combinator that parses [`Comma`]s.
///
/// # Fails
/// The returned combinator fails if the input is not `,`.
///
/// # Example
/// ```rust
/// use ast_toolkit_snack::error::{Common, Failure};
/// use ast_toolkit_snack::{Combinator as _, Result as SResult};
/// use ast_toolkit_span::Span;
/// use datalog::ast::Comma;
/// use datalog::parser::tokens::comma;
///
/// let span1 = Span::new("<example>", ",");
/// let span2 = Span::new("<example>", "foo");
///
/// let mut comb = comma();
/// assert_eq!(comb.parse(span1).unwrap(), (span1.slice(1..), Comma { span: span1.slice(..1) }));
/// assert!(matches!(comb.parse(span2), SResult::Fail(Failure::Common(Common::TagUtf8 { .. }))));
/// ```
pub const fn comma() -> Map<&'static str, &'static str, Tag<'static, &'static str, &'static str>, fn(Span) -> ast::Comma> {
    comb::map(utf8::tag(","), |span| ast::Comma { span })
}

/// Combinator for parsing a `.`.
///
/// # Returns
/// A combinator that parses [`Dot`]s.
///
/// # Fails
/// The returned combinator fails if the input is not `.`.
///
/// # Example
/// ```rust
/// use ast_toolkit_snack::error::{Common, Failure};
/// use ast_toolkit_snack::{Combinator as _, Result as SResult};
/// use ast_toolkit_span::Span;
/// use datalog::ast::Dot;
/// use datalog::parser::tokens::dot;
///
/// let span1 = Span::new("<example>", ".");
/// let span2 = Span::new("<example>", "foo");
///
/// let mut comb = dot();
/// assert_eq!(comb.parse(span1).unwrap(), (span1.slice(1..), Dot { span: span1.slice(..1) }));
/// assert!(matches!(comb.parse(span2), SResult::Fail(Failure::Common(Common::TagUtf8 { .. }))));
/// ```
pub const fn dot() -> Map<&'static str, &'static str, Tag<'static, &'static str, &'static str>, fn(Span) -> ast::Dot> {
    comb::map(utf8::tag("."), |span| ast::Dot { span })
}

/// Combinator for parsing a `not`-keyword.
///
/// # Returns
/// A combinator that parses [`Not`]s.
///
/// # Fails
/// The returned combinator fails if the input is not `not`.
///
/// # Example
/// ```rust
/// use ast_toolkit_snack::error::{Common, Failure};
/// use ast_toolkit_snack::{Combinator as _, Result as SResult};
/// use ast_toolkit_span::Span;
/// use datalog::ast::Not;
/// use datalog::parser::tokens::not;
///
/// let span1 = Span::new("<example>", "not");
/// let span2 = Span::new("<example>", "foo");
///
/// let mut comb = not();
/// assert_eq!(comb.parse(span1).unwrap(), (span1.slice(3..), Not { span: span1.slice(..3) }));
/// assert!(matches!(comb.parse(span2), SResult::Fail(Failure::Common(Common::TagUtf8 { .. }))));
/// ```
pub const fn not() -> Map<&'static str, &'static str, Tag<'static, &'static str, &'static str>, fn(Span) -> ast::Not> {
    comb::map(utf8::tag("not"), |span| ast::Not { span })
}

/// Combinator for parsing parenthesis with something else in between.
///
/// # Arguments
/// - `comb`: Some other combinator that is found in between the parenthesis.
///
/// # Returns
/// A combinator that parses the parenthesis with the given `comb` in between them. Returns it as a tuple of the [`Parens`] and the result of `comb`.
///
/// # Fails
/// The returned combinator fails if the input is not parenthesis, or `comb` fails.
///
/// # Example
/// ```rust
/// use ast_toolkit_snack::combinator::nop;
/// use ast_toolkit_snack::error::{Common, Failure};
/// use ast_toolkit_snack::{Combinator as _, Result as SResult};
/// use ast_toolkit_span::Span;
/// use datalog::ast::Parens;
/// use datalog::parser::tokens::parens;
///
/// let span1 = Span::new("<example>", "()");
/// let span2 = Span::new("<example>", "foo");
///
/// let mut comb = parens(nop());
/// assert_eq!(
///     comb.parse(span1).unwrap(),
///     (span1.slice(2..), Parens { open: span1.slice(..1), close: span1.slice(1..2) })
/// );
/// assert!(matches!(comb.parse(span2), SResult::Fail(Failure::Common(Common::Custom { .. }))));
/// ```
pub const fn parens<'t, C>(
    comb: C,
) -> Map<
    &'static str,
    &'static str,
    Tuple<
        &'static str,
        &'static str,
        (
            MapErr<&'static str, &'static str, Tag<'static, &'static str, &'static str>, fn(Infallible) -> ParensParseError<C::Error>>,
            MapErr<&'static str, &'static str, C, fn(C::Error) -> ParensParseError<C::Error>>,
            MapErr<&'static str, &'static str, Tag<'static, &'static str, &'static str>, fn(Infallible) -> ParensParseError<C::Error>>,
        ),
    >,
    fn((Span, C::Output, Span)) -> (Parens, C::Output),
>
where
    C: Combinator<'t, &'static str, &'static str>,
{
    comb::map(
        seq::tuple((
            comb::map_err(utf8::tag("("), |err| ParensParseError::Open { span: err.span() }),
            comb::map_err(comb, |err| ParensParseError::Middle { err }),
            comb::map_err(utf8::tag(")"), |err| ParensParseError::Close { span: err.span() }),
        )),
        |(open, middle, close)| (Parens { open, close }, middle),
    )
}
