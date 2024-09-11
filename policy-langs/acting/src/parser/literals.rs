//  LITERALS.rs
//    by Lut99
//
//  Created:
//    11 Sep 2024, 14:54:45
//  Last edited:
//    11 Sep 2024, 16:32:09
//  Auto updated?
//    Yes
//
//  Description:
//!   Defines combinators that parse literals in the Acting language.
//

use std::borrow::Cow;
use std::convert::Infallible;
use std::fmt::{Debug, Display, Formatter, Result as FResult};
use std::str::FromStr as _;

use ast_toolkit_snack::bytes::complete as bytes;
use ast_toolkit_snack::c::complete::{self as c, EscapedString};
use ast_toolkit_snack::error::{Common, Error, Failure};
use ast_toolkit_snack::span::{MatchBytes, NextChar, ToStr, WhileBytes, WhileUtf8};
use ast_toolkit_snack::utf8::complete as utf8;
use ast_toolkit_snack::{comb, combinator as comb, Result as SResult};
use ast_toolkit_span::range::SpanRange;
use ast_toolkit_span::{Span, Spannable, Spanning};
use ast_toolkit_tokens::snack::complete as tokens;
use regex::Regex;

use crate::ast::{self, RQuotes};


/***** ERRORS *****/
/// Defines errors occurring when parsing literals.
#[derive(Debug)]
pub enum ParseError<F, S> {
    // LitLang
    LitLang {
        err: Common<'static, F, S, tokens::ParseError<'static, F, S, Infallible>>,
    },

    // LitNow
    LitNow {
        span: Span<F, S>,
    },

    // LitRegex
    /// Failed to match the `r`
    LitRegexR {
        span: Span<F, S>,
    },
    /// Failed to match the escaped string.
    LitRegexString {
        err: Common<'static, F, S, Infallible>,
    },
    /// Failed to parse the matched string as a valid regex.
    LitRegexIllegal {
        span: Span<F, S>,
        err:  regex::Error,
    },

    // LitStr
    /// Failed to parse an escaped string literal.
    LitStr {
        err: Common<'static, F, S, Infallible>,
    },
}
impl<F, S> Display for ParseError<F, S> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        use ParseError::*;
        match self {
            LitLang { .. } => write!(f, "{LitLangExpectsFormatter}"),

            LitNow { .. } => write!(f, "{LitNowExpectsFormatter}"),

            LitRegexR { .. } => write!(f, "{LitRegexExpectsFormatter}"),
            LitRegexString { .. } => write!(f, "{LitRegexExpectsFormatter}"),
            LitRegexIllegal { .. } => write!(f, "Illegal regular expression"),

            LitStr { .. } => write!(f, "{LitStrExpectsFormatter}"),
        }
    }
}
impl<F: Debug, S: Debug> std::error::Error for ParseError<F, S> {}
impl<F: Clone, S: Clone> Spanning<F, S> for ParseError<F, S> {
    #[inline]
    fn span(&self) -> Span<F, S> {
        use ParseError::*;
        match self {
            LitLang { err } => err.span(),

            LitNow { span } => span.clone(),

            LitRegexR { span } => span.clone(),
            LitRegexString { err } => err.span(),
            LitRegexIllegal { span, .. } => span.clone(),

            LitStr { err } => err.span(),
        }
    }
}





/***** LIBRARY *****/
/// Parses a language identifier literal in the Acting language.
///
/// # Returns
/// This combinator returns a [`LitLang`](ast::LitLang)-node.
///
/// # Fails
/// This combinator fails if there was no `<` on the input stream.
///
/// # Errors
/// This combinator fails if the `<` was not followed up by at least one alphanumerical character,
/// or if there was no `>` following that.
///
/// # Example
/// ```rust
/// use acting::ast::{LitLang, Triangles};
/// use acting::parser::literals::{lit_lang, ParseError};
/// use ast_toolkit_snack::error::{Common, Error, Failure};
/// use ast_toolkit_snack::{Combinator as _, Result as SResult};
/// use ast_toolkit_span::Span;
///
/// let span1 = Span::new("<example>", "<datalog>");
/// let span2 = Span::new("<example>", "<slick v1>");
/// let span3 = Span::new("<example>", "<%^>");
/// let span4 = Span::new("<example>", "<datalog");
///
/// let mut comb = lit_lang();
/// assert_eq!(
///     comb.parse(span1).unwrap(),
///     (span1.slice(9..), LitLang {
///         value: span1.slice(1..8),
///         triangle_tokens: Triangles { open: span1.slice(0..1), close: span1.slice(8..9) },
///     })
/// );
/// assert_eq!(
///     comb.parse(span2).unwrap(),
///     (span2.slice(10..), LitLang {
///         value: span2.slice(1..9),
///         triangle_tokens: Triangles { open: span2.slice(0..1), close: span2.slice(9..10) },
///     })
/// );
/// assert!(matches!(
///     comb.parse(span3),
///     SResult::Fail(Failure::Common(Common::Custom(ParseError::LitLang {
///         err: Common::Custom(ast_toolkit_tokens::snack::complete::ParseError::Inner {
///             err: Common::While1Bytes { .. },
///         }),
///     })))
/// ));
/// assert!(matches!(
///     comb.parse(span4),
///     SResult::Error(Error::Common(Common::Custom(ParseError::LitLang {
///         err: Common::Custom(
///             ast_toolkit_tokens::snack::complete::ParseError::Utf8CloseToken { .. }
///         ),
///     })))
/// ));
/// ```
#[inline]
#[comb(expected = "a language identifier", Output = ast::LitLang<F, S>, Error = ParseError<F, S>)]
pub fn lit_lang<F, S>(input: Span<F, S>) -> _
where
    F: Clone,
    S: Clone + MatchBytes + WhileBytes,
{
    match tokens::utf8_delimiter::<F, S, ast::Triangles<F, S>, _>(bytes::while1(|c: u8| -> bool {
        (c >= b'A' && c <= b'Z')
            || (c >= b'a' && c <= b'z')
            || (c >= b'0' && c <= b'9')
            || c == b'-'
            || c == b'_'
            || c == b' '
            || c == b'\t'
            || c == b'\r'
    }))
    .parse(input)
    {
        SResult::Ok(rem, (value, triangle_tokens)) => SResult::Ok(rem, ast::LitLang { value, triangle_tokens }),
        SResult::Fail(Failure::NotEnough { needed, span }) => SResult::Fail(Failure::NotEnough { needed, span }),
        SResult::Fail(fail) => SResult::Fail(Failure::Common(Common::Custom(ParseError::LitLang { err: fail.try_into().unwrap() }))),
        SResult::Error(Error::Context { context, span }) => SResult::Error(Error::Context { context, span }),
        SResult::Error(err) => SResult::Error(Error::Common(Common::Custom(ParseError::LitLang { err: err.try_into().unwrap() }))),
    }
}

/// Parses the `now` literal in the Acting language.
///
/// # Returns
/// This combinator returns a [`LitNow`](ast::LitNow)-node.
///
/// # Fails
/// This combinator fails if the `now`-keyword was not at the top of the input stream.
///
/// # Example
/// ```rust
/// use acting::ast::{LitNow, Now};
/// use acting::parser::literals::{lit_now, ParseError};
/// use ast_toolkit_snack::error::{Common, Failure};
/// use ast_toolkit_snack::{Combinator as _, Result as SResult};
/// use ast_toolkit_span::Span;
///
/// let span1 = Span::new("<example>", "now");
/// let span2 = Span::new("<example>", "noww");
/// let span3 = Span::new("<example>", "foo");
///
/// let mut comb = lit_now();
/// assert_eq!(
///     comb.parse(span1).unwrap(),
///     (span1.slice(3..), LitNow { now_token: Now { span: span1.slice(..3) } })
/// );
/// assert!(matches!(
///     comb.parse(span2),
///     SResult::Fail(Failure::Common(Common::Custom(ParseError::LitNow { .. })))
/// ));
/// assert!(matches!(
///     comb.parse(span3),
///     SResult::Fail(Failure::Common(Common::Custom(ParseError::LitNow { .. })))
/// ));
/// ```
#[inline]
#[comb(expected = "a `now`-literal", Output = ast::LitNow<F, S>, Error = ParseError<F, S>)]
pub fn lit_now<F, S>(input: Span<F, S>) -> _
where
    F: Clone,
    S: Clone + MatchBytes + WhileBytes,
{
    match tokens::utf8_token::<F, S, ast::Now<F, S>, _>(comb::not(bytes::while1(|c: u8| -> bool { c >= b'a' || c <= b'z' }))).parse(input) {
        SResult::Ok(rem, res) => SResult::Ok(rem, ast::LitNow { now_token: res }),
        SResult::Fail(fail) => SResult::Fail(Failure::Common(Common::Custom(ParseError::LitNow { span: fail.span() }))),
        SResult::Error(err) => SResult::Error(Error::Common(Common::Custom(ParseError::LitNow { span: err.span() }))),
    }
}

/// Parses regular expressions in the Acting language.
///
/// # Returns
/// This combinator returns a [`LitRegex`](ast::LitRegex)-node.
///
/// # Fails
/// This combinator fails if the head of the input was not a regular expression quote (`r"`).
///
/// # Errors
/// This combinators errors if the contents of the regex were not a regex, or there was no closing
/// quote (`"`).
///
/// # Example
/// ```rust
/// use acting::ast::{LitRegex, RQuotes};
/// use acting::parser::literals::lit_regex;
/// use ast_toolkit_snack::Combinator as _;
/// use ast_toolkit_span::Span;
/// use regex::Regex;
///
/// let span1 = Span::new("<example>", "r\"(foo|bar)\"");
///
/// let mut comb = lit_regex();
/// assert_eq!(
///     comb.parse(span1).unwrap(),
///     (span1.slice(12..), LitRegex {
///         value: Regex::new("(foo|bar)").unwrap(),
///         value_span: span1.slice(2..11),
///         quote_tokens: RQuotes { open: span1.slice(0..2), close: span1.slice(11..12) },
///     })
/// )
/// ```
#[comb(expected = "a regular expression literal", Output = ast::LitRegex<F, S>, Error = ParseError<F, S>)]
pub fn lit_regex<F, S>(input: Span<F, S>) -> _
where
    F: Clone,
    S: Clone + MatchBytes + NextChar + Spannable + ToStr + WhileUtf8,
{
    // First, ensure to match the `r`
    let (rem, r): (Span<F, S>, Span<F, S>) = match utf8::tag("r").parse(input) {
        SResult::Ok(rem, res) => (rem, res),
        SResult::Fail(fail) => return SResult::Fail(Failure::Common(Common::Custom(ParseError::LitRegexR { span: fail.span() }))),
        SResult::Error(_) => unreachable!(),
    };

    // Then match the escaped string
    let (rem, res): (Span<F, S>, EscapedString<F, S>) = match c::escaped("\"", "\\", |c: &str| -> Result<_, Infallible> { Ok(c.into()) }).parse(rem) {
        SResult::Ok(rem, res) => (rem, res),
        SResult::Fail(Failure::NotEnough { needed, span }) => return SResult::Fail(Failure::NotEnough { needed, span }),
        SResult::Fail(fail) => return SResult::Fail(Failure::Common(Common::Custom(ParseError::LitRegexString { err: fail.try_into().unwrap() }))),
        SResult::Error(Error::Context { context, span }) => return SResult::Error(Error::Context { context, span }),
        SResult::Error(err) => return SResult::Error(Error::Common(Common::Custom(ParseError::LitRegexString { err: err.try_into().unwrap() }))),
    };

    // Attempt to parse the value as a regex
    match Regex::from_str(res.value.as_ref().map(String::as_str).map(Cow::Borrowed).unwrap_or_else(|| res.span.to_str(SpanRange::Open)).as_ref()) {
        Ok(regex) => SResult::Ok(rem, ast::LitRegex {
            value: regex,
            value_span: res.span,
            quote_tokens: RQuotes { open: r.join(&res.delim.0).unwrap(), close: res.delim.1 },
        }),
        Err(err) => SResult::Error(Error::Common(Common::Custom(ParseError::LitRegexIllegal { span: res.span, err }))),
    }
}

/// Parses strings in the Acting language.
///
/// # Returns
/// This combinator returns a [`LitStr`](ast::LitStr)-node.
///
/// # Fails
/// This combinator fails if the head of the input was not a quote (`"`).
///
/// # Errors
/// This combinators errors if the contents of the string were incorrectly escaped, or there was no
/// closing quote (`"`).
///
/// # Example
/// ```rust
/// use acting::ast::{LitStr, Quotes};
/// use acting::parser::literals::lit_str;
/// use ast_toolkit_snack::Combinator as _;
/// use ast_toolkit_span::Span;
///
/// let span1 = Span::new("<example>", "\"Hello, there!\"");
/// let span2 = Span::new("<example>", "\"This: '\\\"' is a quote\"");
///
/// let mut comb = lit_str();
/// assert_eq!(
///     comb.parse(span1).unwrap(),
///     (span1.slice(15..), LitStr {
///         value: "Hello, there!".into(),
///         quote_tokens: Quotes { open: span1.slice(0..1), close: span1.slice(14..15) },
///     })
/// );
/// assert_eq!(
///     comb.parse(span2).unwrap(),
///     (span2.slice(23..), LitStr {
///         value: "This: '\"' is a quote".into(),
///         quote_tokens: Quotes { open: span2.slice(0..1), close: span2.slice(22..23) },
///     })
/// );
/// ```
#[comb(expected = "a string literal", Output = ast::LitStr<F, S>, Error = ParseError<F, S>)]
pub fn lit_str<F, S>(input: Span<F, S>) -> _
where
    F: Clone,
    S: Clone + MatchBytes + NextChar + Spannable + ToStr + WhileUtf8,
{
    match c::escaped("\"", "\\", |c: &str| -> Result<_, Infallible> { Ok(c.into()) }).parse(input) {
        SResult::Ok(rem, res) => SResult::Ok(rem, ast::LitStr {
            value: res.value.unwrap_or_else(|| res.span.to_str(SpanRange::Open).into()),
            quote_tokens: res.delim.into(),
        }),
        SResult::Fail(Failure::NotEnough { needed, span }) => SResult::Fail(Failure::NotEnough { needed, span }),
        SResult::Fail(fail) => SResult::Fail(Failure::Common(Common::Custom(ParseError::LitStr { err: fail.try_into().unwrap() }))),
        SResult::Error(Error::Context { context, span }) => SResult::Error(Error::Context { context, span }),
        SResult::Error(err) => SResult::Error(Error::Common(Common::Custom(ParseError::LitStr { err: err.try_into().unwrap() }))),
    }
}
