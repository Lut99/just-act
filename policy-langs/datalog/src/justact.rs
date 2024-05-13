//  JUSTACT.rs
//    by Lut99
//
//  Created:
//    13 May 2024, 18:39:10
//  Last edited:
//    13 May 2024, 19:02:07
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
use justact_core::policy::{ExtractablePolicy, Policy};
use justact_core::wire::Message;

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





/***** LIBRARY *****/
// Implement `Policy` for Datalog
impl<'f, 's> Policy for Spec<'f, 's> {
    type Explanation = Interpretation<'f, 's>;

    #[inline]
    #[track_caller]
    fn check_validity(&self) -> Result<(), Self::Explanation> {
        // Simply derive and see if `error` occurs.
        let int: Interpretation<'f, 's> = match self.alternating_fixpoint() {
            Ok(int) => int,
            Err(err) => panic!("Failed to run derivation: {err}"),
        };
        let error_truth: Option<bool> = int
            .closed_world_truth(&Atom { ident: Ident { value: Span::new("<justact_policy::datalog::Policy::is_valid()>", "error") }, args: None });
        if error_truth == Some(false) { Ok(()) } else { Err(int) }
    }
}



// Implement `ExtractablePolicy` for Datalog
impl<'f, 's, 'a, I, M> ExtractablePolicy<I> for Spec<'f, 's>
where
    I: Iterator<Item = &'s M>,
    M: 's + Message<Id = &'f str>,
    <M as Authored>::Author: Identifiable<Id = &'a str>,
{
    type ExtractError = ParseError<'f, 's>;

    #[inline]
    fn extract_from(msgs: I) -> Result<Self, Self::ExtractError>
    where
        Self: Sized,
    {
        // Parse the policy in the messages one-by-one
        let mut add_error: bool = false;
        let mut spec = Spec { rules: vec![] };
        for msg in msgs {
            // Parse as UTF-8
            let snippet: &str = match std::str::from_utf8(msg.payload()) {
                Ok(snippet) => snippet,
                Err(err) => return Err(ParseError::Utf8 { err }),
            };

            // Parse as Datalog
            let msg_spec: Spec = match parse(msg.id(), snippet) {
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
