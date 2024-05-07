//  TOKENS.rs
//    by Lut99
//
//  Created:
//    18 Mar 2024, 12:04:42
//  Last edited:
//    07 May 2024, 11:49:09
//  Auto updated?
//    Yes
//
//  Description:
//!   Defines parsers for $Datalog^\neg$ keywords.
//

use ast_toolkit_snack::combinator::{self as comb, Map, Transmute};
use ast_toolkit_snack::sequence::{self as seq, Delim};
use ast_toolkit_snack::utf8::complete::{self as utf8, Tag};
use ast_toolkit_snack::Combinator;

use crate::ast;


/***** TYPE ALIASES *****/
/// The returned type of the [`parens()`]-combinator.
pub type Parens<'t, C> = Map<
    &'static str,
    &'static str,
    Delim<
        &'static str,
        &'static str,
        Transmute<&'static str, &'static str, Tag<'static, &'static str, &'static str>, <C as Combinator<'t, &'static str, &'static str>>::Error>,
        C,
        Transmute<&'static str, &'static str, Tag<'static, &'static str, &'static str>, <C as Combinator<'t, &'static str, &'static str>>::Error>,
    >,
    fn(
        (Span, <C as Combinator<'t, &'static str, &'static str>>::Output, Span),
    ) -> (ast::Parens, <C as Combinator<'t, &'static str, &'static str>>::Output),
>;

/// Convenience alias for a [`Span`](ast_toolkit_span::Span) over static strings.
type Span = ast_toolkit_span::Span<&'static str, &'static str>;





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
///     (span1.slice(2..), (Parens { open: span1.slice(..1), close: span1.slice(1..2) }, ()))
/// );
/// assert!(matches!(comb.parse(span2), SResult::Fail(Failure::Common(Common::DelimOpen { .. }))));
/// ```
pub const fn parens<'t, C>(comb: C) -> Parens<'t, C>
where
    C: Combinator<'t, &'static str, &'static str>,
{
    comb::map(seq::delim(comb::transmute(utf8::tag("(")), comb, comb::transmute(utf8::tag(")"))), |(open, middle, close)| {
        (ast::Parens { open, close }, middle)
    })
}
