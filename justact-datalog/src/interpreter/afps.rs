//  AFPS.rs
//    by Lut99
//
//  Created:
//    21 Mar 2024, 11:19:14
//  Last edited:
//    22 Mar 2024, 18:00:14
//  Auto updated?
//    Yes
//
//  Description:
//!   Implements the Alternating Fixed-Point Semantics for $Datalog^\neg$
//!   programs.
//

use indexmap::IndexSet;

use super::herbrand::HerbrandInstantiationIterator;
use super::interpretation::Interpretation;
use crate::ast::{Atom, Ident, Spec};
use crate::log::{debug, trace};


/***** TESTS *****/
#[cfg(all(test, feature = "derive"))]
mod tests {
    use ast_toolkit_punctuated::Punctuated;
    use ast_toolkit_span::Span;
    use justact_datalog_derive::datalog;

    use super::super::herbrand::{Constants as _, HerbrandBaseIterator};
    use super::*;
    use crate::ast::{Atom, AtomArg, AtomArgs, Comma, Ident, Parens};


    /// Sets up a logger if wanted.
    #[cfg(feature = "log")]
    fn setup_logger() {
        use humanlog::{DebugMode, HumanLogger};

        // Check if the envs tell us to
        if let Ok(logger) = std::env::var("LOGGER") {
            if logger == "1" || logger == "true" {
                // Create the logger
                if let Err(err) = HumanLogger::terminal(DebugMode::Full).init() {
                    eprintln!("WARNING: Failed to setup logger: {err} (no logging for this session)");
                }
            }
        }
    }

    /// Makes an [`Atom`] conveniently.
    ///
    /// # Arguments
    /// - `name`: The name of the atom.
    /// - `args`: Any arguments to the atom, given as identifiers.
    ///
    /// # Returns
    /// A new [`Atom`].
    fn make_atom(name: &'static str, args: impl IntoIterator<Item = &'static str>) -> Atom {
        // Build the arguments
        let mut args_punct: Punctuated<AtomArg, Comma> = Punctuated::new();
        for (i, arg) in args.into_iter().enumerate() {
            if i == 0 {
                args_punct.push_first(AtomArg::Atom(Ident { value: Span::new("make_atom::arg", arg) }));
            } else {
                args_punct
                    .push(Comma { span: Span::new("make_atom::arg::comma", ",") }, AtomArg::Atom(Ident { value: Span::new("make_atom::arg", arg) }));
            }
        }

        // Build the full atom
        Atom {
            ident: Ident { value: Span::new("make_atom::ident", name) },
            args:  if !args_punct.is_empty() {
                Some(AtomArgs {
                    paren_tokens: Parens { open: Span::new("make_atom::parens::open", "("), close: Span::new("make_atom::parens::close", ")") },
                    args: args_punct,
                })
            } else {
                None
            },
        }
    }


    #[test]
    fn test_consequence_trans() {
        #[cfg(feature = "log")]
        setup_logger();

        // Every day, got nothing to do
        let spec: Spec = datalog! { #![crate] };
        let mut int: Interpretation = Interpretation::new();
        consequence_trans(HerbrandInstantiationIterator::new(&spec, &HerbrandBaseIterator::new(&spec).constants().collect()), &mut int);
        assert!(int.is_empty());

        // Derive some constants
        let spec: Spec = datalog! {
            #![crate]
            foo. bar. baz.
        };
        let mut int: Interpretation = Interpretation::new();
        consequence_trans(HerbrandInstantiationIterator::new(&spec, &HerbrandBaseIterator::new(&spec).constants().collect()), &mut int);
        assert_eq!(int.len(), 3);
        assert_eq!(int.truth_of_atom(&make_atom("foo", [])), Some(true));
        assert_eq!(int.truth_of_atom(&make_atom("bar", [])), Some(true));
        assert_eq!(int.truth_of_atom(&make_atom("baz", [])), Some(true));

        // Do derivation from the spec itself
        let spec: Spec = datalog! {
            #![crate]
            foo. bar :- foo.
        };
        let mut int: Interpretation = Interpretation::new();
        consequence_trans(HerbrandInstantiationIterator::new(&spec, &HerbrandBaseIterator::new(&spec).constants().collect()), &mut int);
        assert_eq!(int.len(), 2);
        assert_eq!(int.truth_of_atom(&make_atom("foo", [])), Some(true));
        assert_eq!(int.truth_of_atom(&make_atom("bar", [])), Some(true));

        // Do derivation from the interpretation
        let spec: Spec = datalog! {
            #![crate]
            bar :- foo.
        };
        let mut int: Interpretation = Interpretation::from([(make_atom("foo", []), true)]);
        consequence_trans(HerbrandInstantiationIterator::new(&spec, &HerbrandBaseIterator::new(&spec).constants().collect()), &mut int);
        assert_eq!(int.len(), 2);
        assert_eq!(int.truth_of_atom(&make_atom("foo", [])), Some(true));
        assert_eq!(int.truth_of_atom(&make_atom("bar", [])), Some(true));

        // Do derivation with functions
        let spec: Spec = datalog! {
            #![crate]
            foo. bar. baz(foo). quz(foo, bar).
        };
        let mut int: Interpretation = Interpretation::new();
        consequence_trans(HerbrandInstantiationIterator::new(&spec, &HerbrandBaseIterator::new(&spec).constants().collect()), &mut int);
        assert_eq!(int.len(), 4);
        assert_eq!(int.truth_of_atom(&make_atom("foo", [])), Some(true));
        assert_eq!(int.truth_of_atom(&make_atom("bar", [])), Some(true));
        assert_eq!(int.truth_of_atom(&make_atom("baz", ["foo"])), Some(true));
        assert_eq!(int.truth_of_atom(&make_atom("quz", ["foo", "bar"])), Some(true));

        // Do derivation with variables!
        let spec: Spec = datalog! {
            #![crate]
            foo. bar. baz(X). quz(X, Y) :- baz(X).
        };
        let mut int: Interpretation = Interpretation::new();
        consequence_trans(HerbrandInstantiationIterator::new(&spec, &HerbrandBaseIterator::new(&spec).constants().collect()), &mut int);
        assert_eq!(int.len(), 8);
        assert_eq!(int.truth_of_atom(&make_atom("foo", [])), Some(true));
        assert_eq!(int.truth_of_atom(&make_atom("bar", [])), Some(true));
        assert_eq!(int.truth_of_atom(&make_atom("baz", ["foo"])), Some(true));
        assert_eq!(int.truth_of_atom(&make_atom("baz", ["bar"])), Some(true));
        assert_eq!(int.truth_of_atom(&make_atom("quz", ["foo", "foo"])), Some(true));
        assert_eq!(int.truth_of_atom(&make_atom("quz", ["foo", "bar"])), Some(true));
        assert_eq!(int.truth_of_atom(&make_atom("quz", ["bar", "foo"])), Some(true));
        assert_eq!(int.truth_of_atom(&make_atom("quz", ["bar", "bar"])), Some(true));
    }
}





/***** TRANSFORMATIONS *****/
/// Implements a simple positive-only derivation procedure for a given [`Spec`] and knowledge base, known as an _immediate consequence transformation_ (or _-operator_).
///
/// In addition, a particular set of negative literals can be given that are assumed to be true.
///
/// # Arguments
/// - `rules`: A [`HerbrandInstantiationIterator`] that iterates of all the concrete rules in a [`Spec`].
/// - `int`: Some interpretation that contains facts we assume to be true. We will derive new facts in this interpretation as well.
pub fn consequence_trans(mut rules: HerbrandInstantiationIterator, int: &mut Interpretation) {
    debug!("Running immediate consequent transformation");

    // This transformation is saturating, so continue until no rules are triggered anymore.
    // NOTE: Monotonic because we can never remove truths, inferring the same fact does not count as a change and we are iterating over a Herbrand instantiation so our search space is finite (for $Datalog^\neg$, at least).
    let mut changed: bool = true;
    let mut i: usize = 0;
    while changed {
        changed = false;
        i += 1;

        // Go thru da rules
        // NOTE: HerbrandInstantiationIterator is not an official iterator because it doesn't GAT. So we do this manually.
        'rule: while let Some(rule) = rules.next() {
            trace!("[{i}] Running immediate consequent transformation for '{rule}'");

            // See if we can find the antecedents in the interpretation. No antecedents? Rule trivially accepted!
            for ante in rule.tail.iter().map(|t| t.antecedents.values()).flatten() {
                if !matches!(int.truth_of_lit(ante), Some(true)) {
                    // The antecedant is not true; cannot derive this fact
                    trace!("[{i}] Antecedent '{ante}' not true in the interpretation; Rule '{rule}' does not hold");
                    continue 'rule;
                }
            }

            // If all antecedants were in the interpretation, then derive the consequents.
            for cons in rule.consequences.values() {
                trace!("[{i}] Deriving '{cons}' from '{rule}' (all antecedents explicitly hold)");
                if !matches!(int.learn(cons.clone(), true), Some(true)) {
                    changed = true;
                }
            }
        }
    }

    // Done!
    trace!("Done saturating immediate consequent transformation (took {i} passes)");
}

/// Implements a complement-negation of an interpretation for a Herbrand base, know as a _stability transformation_.
///
/// Specifically, find all elements in the Herbrand base not in the interpretation, then negates those.
///  
/// # Arguments
/// - `hbase`: The Herbrand Base of a spec that we accept as the "universe" of literals.
/// - `int`: Some interpretation to negate.
/// - `res`: Some existing [`Interpretation`] that we [clear](Interpretation::clear()) and populate with the negated complement.
pub fn stability_trans(hbase: &IndexSet<&Atom>, int: &Interpretation, res: &mut Interpretation) {
    // // Go through the Herbrand Base of the specification
    // for atom in hbase {
    //     // Find
    //     if int.truth_of_atom(atom).is_some() {}
    // }
}





/***** LIBRARY FUNCTIONS *****/
/// Given a $Datalog^\neg$ specification, computes the well-founded model given to us by the alternating fixed-point semantics.
///
/// # Arguments
/// - `spec`: The $Datalog^\neg$-program to consider.
///
/// # Returns
/// Three new [`LogicalSet`] that represents the knowledge base of the program.
pub fn evaluate(spec: &Spec) -> Interpretation { todo!() }
