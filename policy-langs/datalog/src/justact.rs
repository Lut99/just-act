//  JUSTACT.rs
//    by Lut99
//
//  Created:
//    13 May 2024, 18:39:10
//  Last edited:
//    23 May 2024, 17:04:52
//  Auto updated?
//    Yes
//
//  Description:
//!   Implements the JustAct-compatible [`Policy`]-trait for
//!   $Datalog^\neg$'s [`Spec`].
//

use std::error::Error;
use std::fmt::{Display, Formatter, Result as FResult};

use ast_toolkit_punctuated::Punctuated;
use justact_core::auxillary::{Authored, Identifiable};
use justact_core::set::LocalSet;
use justact_core::statements::{Extractable, Message, Policy};

use crate::ast::{Atom, Comma, Dot, Ident, Rule, Span, Spec};
use crate::interpreter::interpretation::Interpretation;
use crate::parser::parse;


/***** ERRORS *****/
/// Defines a failure during parsing [`Spec`]s from [`MessageSet`](justact_core::wire::MessageSet)s.
#[derive(Debug)]
pub enum ParseError<'f, 's> {
    /// Failed to read the input as valid UTF-8.
    Utf8 { err: std::str::Utf8Error },
    /// Failed to parse the input UTF-8 as Datalog.
    Datalog { err: crate::parser::Error<'f, 's> },
}
impl<'f, 's> Display for ParseError<'f, 's> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        use ParseError::*;
        match self {
            Utf8 { .. } => write!(f, "Failed to parse message set contents as valid UTF-8"),
            Datalog { err } => write!(f, "{err}\n\nFailed to parse message set contents as valid Datalog (see output above)"),
        }
    }
}
impl<'f, 's> Error for ParseError<'f, 's> {
    #[inline]
    fn source(&self) -> Option<&(dyn 'static + Error)> {
        use ParseError::*;
        match self {
            Utf8 { err } => Some(err),
            Datalog { .. } => None,
        }
    }
}

/// Defines reasons why a policy wasn't valid.
#[derive(Debug)]
pub enum ValidityError<'f, 's> {
    /// `error.` was derived.
    ErrorHolds { int: Interpretation<'f, 's> },
}
impl<'f, 's> Display for ValidityError<'f, 's> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        use ValidityError::*;
        match self {
            ErrorHolds { int } => write!(f, "\"error\" holds in the interpretation\n\n{int}"),
        }
    }
}
impl<'f, 's> Error for ValidityError<'f, 's> {}





/***** LIBRARY *****/
// Implements `Extractable` for Datalog
impl<'v, M> Extractable<'v, M> for Spec<'v, 'v>
where
    M: Authored<AuthorId = str> + Identifiable<Id = str> + Message<'v>,
{
    type SyntaxError = ParseError<'v, 'v>;

    #[inline]
    fn extract_from<R>(set: &LocalSet<M, R>) -> Result<Self, Self::SyntaxError>
    where
        Self: Sized,
    {
        // Parse the policy in the messages one-by-one
        let mut add_error: bool = false;
        let mut spec = Spec { rules: vec![] };
        for msg in set {
            // Parse as UTF-8
            let snippet: &str = match std::str::from_utf8(msg.payload()) {
                Ok(snippet) => snippet,
                Err(err) => return Err(ParseError::Utf8 { err }),
            };

            // Parse as Datalog
            let msg_spec: Spec = match parse(msg.id_v(), snippet) {
                Ok(spec) => spec,
                Err(err) => return Err(ParseError::Datalog { err }),
            };

            // Check if there's any illegal rules
            if !add_error {
                'rules: for rule in &msg_spec.rules {
                    for cons in rule.consequences.values() {
                        // If a consequent begins with 'ctl-'...
                        if cons.ident.value.value().starts_with("ctl-") {
                            // ...and its first argument is _not_ the author of the message...
                            if let Some(arg) = cons.args.iter().flat_map(|a| a.args.values().next()).next() {
                                if arg.ident().value.value() != msg.author() {
                                    continue;
                                } else {
                                    // ...then we derive error
                                    add_error = true;
                                    break 'rules;
                                }
                            } else {
                                // ...then we derive error
                                add_error = true;
                                break 'rules;
                            }
                        }
                    }
                }
            }

            // OK, now we can add all the rules together
            spec.rules.extend(msg_spec.rules);
        }

        // If there were any illegal rules, inject error
        if add_error {
            // Build the list of consequences
            let mut consequences: Punctuated<Atom, Comma> = Punctuated::new();
            consequences.push_first(Atom { ident: Ident { value: Span::new("<datalog::justact::Spec::extract_from>", "error") }, args: None });

            // Then add the rule
            spec.rules.push(Rule { consequences, tail: None, dot: Dot { span: Span::new("<datalog::justact::Spec::extract_from>", ".") } })
        }

        // OK, return the spec
        Ok(spec)
    }
}



// Implements `Policy` for Datalog
impl<'v> Policy<'v> for Spec<'v, 'v> {
    type SemanticError = ValidityError<'v, 'v>;

    #[inline]
    #[track_caller]
    fn assert_validity(&self) -> Result<(), Self::SemanticError> {
        // Simply derive and see if `error` occurs.
        let int: Interpretation<'v, 'v> = match self.alternating_fixpoint() {
            Ok(int) => int,
            Err(err) => panic!("Failed to run derivation: {err}"),
        };
        let error_truth: Option<bool> = int
            .closed_world_truth(&Atom { ident: Ident { value: Span::new("<justact_policy::datalog::Policy::is_valid()>", "error") }, args: None });
        if error_truth == Some(false) { Ok(()) } else { Err(ValidityError::ErrorHolds { int }) }
    }
}
