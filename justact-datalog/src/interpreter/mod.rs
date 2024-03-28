//  MOD.rs
//    by Lut99
//
//  Created:
//    26 Mar 2024, 19:36:31
//  Last edited:
//    28 Mar 2024, 11:36:56
//  Auto updated?
//    Yes
//
//  Description:
//!   Implements an interpreter that can evaluate $Datalog^\neg$ programs
//!   ([`Spec`]s).
//!
//!   The interpreter implements the _alternating fixed-point semantics_,
//!   an operational variation on the _well-founded semantics_.
//!
//!   The implementation is based on [1].
//!
//!   # References
//!   [1] A. Van Gelder. 1989. _The alternating fixpoint of logic programs with
//!       negation._ In Proceedings of the eighth ACM SIGACT-SIGMOD-SIGART
//!       symposium on Principles of database systems (PODS '89). Association
//!       for Computing Machinery, New York, NY, USA, 1–10.
//!       <https://doi.org/10.1145/73721.73722>

// Nested modules
pub mod interpretation;

// Imports
use std::error;
use std::fmt::{Display, Formatter, Result as FResult};

use ast_toolkit_span::Span;
use indexmap::set::IndexSet;
use stackvec::StackVec;

use self::interpretation::Interpretation;
use crate::ast::{Atom, AtomArg, Ident, Literal, Rule, Spec};
use crate::log::{debug, trace};


/***** TESTS *****/
#[cfg(all(test, feature = "derive"))]
mod tests {
    use ast_toolkit_punctuated::Punctuated;
    use ast_toolkit_span::Span;
    use justact_datalog_derive::datalog;

    use super::*;
    use crate::ast::{Atom, AtomArgs, Comma, Parens};


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

    /// Makes an [`Ident`] conveniently.
    fn make_ident(name: &'static str) -> Ident { Ident { value: Span::new("make_ident::value", name) } }

    /// Makes an [`Atom`] conveniently.
    fn make_atom(name: &'static str, args: impl IntoIterator<Item = &'static str>) -> Atom {
        // Make the punctuation
        let mut punct: Punctuated<AtomArg, Comma> = Punctuated::new();
        for (i, arg) in args.into_iter().enumerate() {
            if i == 0 {
                punct.push_first(AtomArg::Atom(Ident { value: Span::new("make_atom::arg", arg) }));
            } else {
                punct.push(Comma { span: Span::new("make_atom::arg::comma", ",") }, AtomArg::Atom(Ident { value: Span::new("make_atom::arg", arg) }));
            }
        }

        // Make the atom
        Atom {
            ident: Ident { value: Span::new("make_atom::name", name) },
            args:  if !punct.is_empty() {
                Some(AtomArgs {
                    paren_tokens: Parens { open: Span::new("make_atom::parens::open", "("), close: Span::new("make_atom::parens::close", ")") },
                    args: punct,
                })
            } else {
                None
            },
        }
    }


    #[test]
    fn test_find_0_base() {
        #[cfg(feature = "log")]
        setup_logger();

        // Empty spec first
        let empty: Spec = datalog! {
            #![crate]
        };
        let base = empty.find_0_base();
        let mut iter = base.iter();
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);

        // Spec with only constants (one, first)
        let one: Spec = datalog! {
            #![crate]
            foo.
        };
        let base = one.find_0_base();
        let mut iter = base.iter();
        assert_eq!(iter.next(), Some(&make_ident("foo")));
        assert_eq!(iter.next(), None);

        // Multiple constants
        let consts: Spec = datalog! {
            #![crate]
            foo. bar. baz.
        };
        let base = consts.find_0_base();
        let mut iter = base.iter();
        assert_eq!(iter.next(), Some(&make_ident("foo")));
        assert_eq!(iter.next(), Some(&make_ident("bar")));
        assert_eq!(iter.next(), Some(&make_ident("baz")));
        assert_eq!(iter.next(), None);

        // Duplicate constants
        let dups: Spec = datalog! {
            #![crate]
            foo. foo. bar.
        };
        let base = dups.find_0_base();
        let mut iter = base.iter();
        assert_eq!(iter.next(), Some(&make_ident("foo")));
        // NOTE: Would be here if it weren't for the fact we're currently going thru an IndexSet
        // assert_eq!(iter.next(), Some(&make_ident("foo")));
        assert_eq!(iter.next(), Some(&make_ident("bar")));
        assert_eq!(iter.next(), None);

        // Spec with arity-1 atoms (functions)
        let funcs: Spec = datalog! {
            #![crate]
            foo(bar). bar(baz). baz(quz).
        };
        let base = funcs.find_0_base();
        let mut iter = base.iter();
        assert_eq!(iter.next(), Some(&make_ident("bar")));
        assert_eq!(iter.next(), Some(&make_ident("baz")));
        assert_eq!(iter.next(), Some(&make_ident("quz")));
        assert_eq!(iter.next(), None);

        // Mixed arity
        let arity: Spec = datalog! {
            #![crate]
            foo. bar(). baz(quz). quz(qux, quux). corge(grault, garply, waldo).
        };
        let base = arity.find_0_base();
        let mut iter = base.iter();
        assert_eq!(iter.next(), Some(&make_ident("foo")));
        assert_eq!(iter.next(), Some(&make_ident("bar")));
        assert_eq!(iter.next(), Some(&make_ident("quz")));
        assert_eq!(iter.next(), Some(&make_ident("qux")));
        assert_eq!(iter.next(), Some(&make_ident("quux")));
        assert_eq!(iter.next(), Some(&make_ident("grault")));
        assert_eq!(iter.next(), Some(&make_ident("garply")));
        assert_eq!(iter.next(), Some(&make_ident("waldo")));
        assert_eq!(iter.next(), None);

        // Full rules
        let rules: Spec = datalog! {
            #![crate]
            foo. bar(baz). quz(X) :- bar(X), qux(quux).
        };
        let base = rules.find_0_base();
        let mut iter = base.iter();
        assert_eq!(iter.next(), Some(&make_ident("foo")));
        assert_eq!(iter.next(), Some(&make_ident("baz")));
        assert_eq!(iter.next(), Some(&make_ident("quux")));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_rule_immediate_consequence() {
        // Try some constants, no prior knowledge
        let consts: Spec = datalog! {
            #![crate]
            foo. bar. baz.
        };
        let zero_base: IndexSet<Ident> = consts.find_0_base();
        let pre: Interpretation = Interpretation::new();
        let mut aft: Interpretation = Interpretation::new();
        if let Err(err) = consts.rules[0].immediate_consequence::<1>(&zero_base, &pre, &mut aft) {
            panic!("{err}");
        }
        assert_eq!(aft.len(), 1);
        assert_eq!(aft.truth_of_atom(&make_atom("foo", None)), Some(true));
        if let Err(err) = consts.rules[1].immediate_consequence::<1>(&zero_base, &pre, &mut aft) {
            panic!("{err}");
        }
        assert_eq!(aft.len(), 2);
        assert_eq!(aft.truth_of_atom(&make_atom("bar", None)), Some(true));
        if let Err(err) = consts.rules[2].immediate_consequence::<1>(&zero_base, &pre, &mut aft) {
            panic!("{err}");
        }
        assert_eq!(aft.len(), 3);
        assert_eq!(aft.truth_of_atom(&make_atom("baz", None)), Some(true));

        // Make some derivat
    }

    #[test]
    fn test_spec_immediate_consequence() {
        // Try some constants, no prior knowledge
        let consts: Spec = datalog! {
            #![crate]
            foo. bar. baz.
        };
        let mut pre: Interpretation = Interpretation::new();
        let mut aft: Interpretation = Interpretation::new();
        if let Err(err) = consts.immediate_consequence::<16>(&mut pre, &mut aft) {
            panic!("{err}");
        }
        assert_eq!(aft.len(), 3);
        assert_eq!(aft.truth_of_atom(&make_atom("foo", None)), Some(true));
        assert_eq!(aft.truth_of_atom(&make_atom("bar", None)), Some(true));
        assert_eq!(aft.truth_of_atom(&make_atom("baz", None)), Some(true));

        // Try some functions, no prior knowledge
        let funcs: Spec = datalog! {
            #![crate]
            foo(bar). bar(baz). baz(quz).
        };
        let mut pre: Interpretation = Interpretation::new();
        let mut aft: Interpretation = Interpretation::new();
        if let Err(err) = funcs.immediate_consequence::<16>(&mut pre, &mut aft) {
            panic!("{err}");
        }
        assert_eq!(aft.len(), 3);
        assert_eq!(aft.truth_of_atom(&make_atom("foo", Some("bar"))), Some(true));
        assert_eq!(aft.truth_of_atom(&make_atom("bar", Some("baz"))), Some(true));
        assert_eq!(aft.truth_of_atom(&make_atom("baz", Some("quz"))), Some(true));

        // Try some (grounded) rules, no prior knowledge
        let rules: Spec = datalog! {
            #![crate]
            foo. bar(foo) :- foo. quz(bar) :- bar.
        };
        let mut pre: Interpretation = Interpretation::new();
        let mut aft: Interpretation = Interpretation::new();
        if let Err(err) = rules.immediate_consequence::<16>(&mut pre, &mut aft) {
            panic!("{err}");
        }
        assert_eq!(aft.len(), 2);
        assert_eq!(aft.truth_of_atom(&make_atom("foo", None)), Some(true));
        assert_eq!(aft.truth_of_atom(&make_atom("bar", Some("foo"))), Some(true));

        // Try some (grounded) rules _with_ prior knowledge
        let mut pre: Interpretation = Interpretation::from([(make_atom("bar", None), true)]);
        let mut aft: Interpretation = Interpretation::new();
        if let Err(err) = rules.immediate_consequence::<16>(&mut pre, &mut aft) {
            panic!("{err}");
        }
        assert_eq!(aft.len(), 4);
        assert_eq!(aft.truth_of_atom(&make_atom("foo", None)), Some(true));
        assert_eq!(aft.truth_of_atom(&make_atom("bar", None)), Some(true));
        assert_eq!(aft.truth_of_atom(&make_atom("bar", Some("foo"))), Some(true));
        assert_eq!(aft.truth_of_atom(&make_atom("quz", Some("bar"))), Some(true));

        // Try some rules with variables
        let vars: Spec = datalog! {
            #![crate]
            foo. bar. baz.
            quz(foo). quz(bar). quz(baz).
            qux(X) :- quz(X).
            quux(X, Y) :- qux(X), quz(Y).
        };
        let mut pre: Interpretation = Interpretation::new();
        let mut aft: Interpretation = Interpretation::new();
        if let Err(err) = vars.immediate_consequence::<16>(&mut pre, &mut aft) {
            panic!("{err}");
        }
        assert_eq!(aft.len(), 18);
        assert_eq!(aft.truth_of_atom(&make_atom("foo", [])), Some(true));
        assert_eq!(aft.truth_of_atom(&make_atom("bar", [])), Some(true));
        assert_eq!(aft.truth_of_atom(&make_atom("baz", [])), Some(true));
        assert_eq!(aft.truth_of_atom(&make_atom("quz", ["foo"])), Some(true));
        assert_eq!(aft.truth_of_atom(&make_atom("quz", ["bar"])), Some(true));
        assert_eq!(aft.truth_of_atom(&make_atom("quz", ["baz"])), Some(true));
        assert_eq!(aft.truth_of_atom(&make_atom("qux", ["foo"])), Some(true));
        assert_eq!(aft.truth_of_atom(&make_atom("qux", ["bar"])), Some(true));
        assert_eq!(aft.truth_of_atom(&make_atom("qux", ["baz"])), Some(true));
        assert_eq!(aft.truth_of_atom(&make_atom("quux", ["foo", "foo"])), Some(true));
        assert_eq!(aft.truth_of_atom(&make_atom("quux", ["foo", "bar"])), Some(true));
        assert_eq!(aft.truth_of_atom(&make_atom("quux", ["foo", "baz"])), Some(true));
        assert_eq!(aft.truth_of_atom(&make_atom("quux", ["bar", "foo"])), Some(true));
        assert_eq!(aft.truth_of_atom(&make_atom("quux", ["bar", "bar"])), Some(true));
        assert_eq!(aft.truth_of_atom(&make_atom("quux", ["bar", "baz"])), Some(true));
        assert_eq!(aft.truth_of_atom(&make_atom("quux", ["baz", "foo"])), Some(true));
        assert_eq!(aft.truth_of_atom(&make_atom("quux", ["baz", "bar"])), Some(true));
        assert_eq!(aft.truth_of_atom(&make_atom("quux", ["baz", "baz"])), Some(true));
    }

    #[test]
    fn test_spec_stable_transformation() {
        // Try some constants, no prior knowledge
        let consts: Spec = datalog! {
            #![crate]
            foo. bar. baz.
        };
        let mut pre: Interpretation = Interpretation::new();
        let mut aft: Interpretation = Interpretation::new();
        if let Err(err) = consts.stable_transformation::<16>(&consts.find_0_base(), &mut pre, &mut aft) {
            panic!("{err}");
        }
        assert_eq!(aft.len(), 3);
        assert_eq!(aft.truth_of_atom(&make_atom("foo", None)), Some(false));
        assert_eq!(aft.truth_of_atom(&make_atom("bar", None)), Some(false));
        assert_eq!(aft.truth_of_atom(&make_atom("baz", None)), Some(false));

        // Try some constants, prior knowledge
        let consts2: Spec = datalog! {
            #![crate]
            foo. bar. baz.
        };
        let mut pre: Interpretation = Interpretation::from([(make_atom("foo", None), true)]);
        let mut aft: Interpretation = Interpretation::new();
        if let Err(err) = consts2.stable_transformation::<16>(&consts2.find_0_base(), &mut pre, &mut aft) {
            panic!("{err}");
        }
        assert_eq!(aft.len(), 2);
        assert_eq!(aft.truth_of_atom(&make_atom("bar", None)), Some(false));
        assert_eq!(aft.truth_of_atom(&make_atom("baz", None)), Some(false));

        // Try some functions, some prior knowledge
        let funcs: Spec = datalog! {
            #![crate]
            foo(bar). bar(baz). baz(quz).
        };
        let mut pre: Interpretation = Interpretation::from([(make_atom("bar", None), true), (make_atom("foo", Some("bar")), true)]);
        let mut aft: Interpretation = Interpretation::new();
        if let Err(err) = funcs.stable_transformation::<16>(&funcs.find_0_base(), &mut pre, &mut aft) {
            panic!("{err}");
        }
        assert_eq!(aft.len(), 4);
        assert_eq!(aft.truth_of_atom(&make_atom("baz", None)), Some(false));
        assert_eq!(aft.truth_of_atom(&make_atom("quz", None)), Some(false));
        assert_eq!(aft.truth_of_atom(&make_atom("bar", Some("baz"))), Some(false));
        assert_eq!(aft.truth_of_atom(&make_atom("baz", Some("quz"))), Some(false));

        // Try some (grounded) rules, no prior knowledge
        let rules: Spec = datalog! {
            #![crate]
            foo. bar(foo) :- foo. quz(bar) :- bar.
        };
        let mut pre: Interpretation = Interpretation::new();
        let mut aft: Interpretation = Interpretation::new();
        if let Err(err) = rules.stable_transformation::<16>(&rules.find_0_base(), &mut pre, &mut aft) {
            panic!("{err}");
        }
        assert_eq!(aft.len(), 4);
        assert_eq!(aft.truth_of_atom(&make_atom("foo", None)), Some(false));
        assert_eq!(aft.truth_of_atom(&make_atom("bar", None)), Some(false));
        assert_eq!(aft.truth_of_atom(&make_atom("bar", Some("foo"))), Some(false));
        assert_eq!(aft.truth_of_atom(&make_atom("quz", Some("bar"))), Some(false));

        // Try some (grounded) rules _with_ prior knowledge
        let mut pre: Interpretation = Interpretation::from([(make_atom("bar", None), true), (make_atom("quz", Some("bar")), true)]);
        let mut aft: Interpretation = Interpretation::new();
        if let Err(err) = rules.stable_transformation::<16>(&rules.find_0_base(), &mut pre, &mut aft) {
            panic!("{err}");
        }
        assert_eq!(aft.len(), 2);
        assert_eq!(aft.truth_of_atom(&make_atom("foo", None)), Some(false));
        assert_eq!(aft.truth_of_atom(&make_atom("bar", Some("foo"))), Some(false));

        // Try some rules with variables
        let vars: Spec = datalog! {
            #![crate]
            foo. bar. baz.
            quz(foo). quz(bar). quz(baz).
            qux(X) :- quz(X).
            quux(X, Y) :- qux(X), quz(Y).
        };
        let mut pre: Interpretation = Interpretation::from([(make_atom("foo", []), true), (make_atom("quux", ["foo", "foo"]), true)]);
        let mut aft: Interpretation = Interpretation::new();
        if let Err(err) = vars.stable_transformation::<16>(&vars.find_0_base(), &mut pre, &mut aft) {
            panic!("{err}");
        }
        assert_eq!(aft.len(), 16);
        assert_eq!(aft.truth_of_atom(&make_atom("bar", [])), Some(false));
        assert_eq!(aft.truth_of_atom(&make_atom("baz", [])), Some(false));
        assert_eq!(aft.truth_of_atom(&make_atom("quz", ["foo"])), Some(false));
        assert_eq!(aft.truth_of_atom(&make_atom("quz", ["bar"])), Some(false));
        assert_eq!(aft.truth_of_atom(&make_atom("quz", ["baz"])), Some(false));
        assert_eq!(aft.truth_of_atom(&make_atom("qux", ["foo"])), Some(false));
        assert_eq!(aft.truth_of_atom(&make_atom("qux", ["bar"])), Some(false));
        assert_eq!(aft.truth_of_atom(&make_atom("qux", ["baz"])), Some(false));
        assert_eq!(aft.truth_of_atom(&make_atom("quux", ["foo", "bar"])), Some(false));
        assert_eq!(aft.truth_of_atom(&make_atom("quux", ["foo", "baz"])), Some(false));
        assert_eq!(aft.truth_of_atom(&make_atom("quux", ["bar", "foo"])), Some(false));
        assert_eq!(aft.truth_of_atom(&make_atom("quux", ["bar", "bar"])), Some(false));
        assert_eq!(aft.truth_of_atom(&make_atom("quux", ["bar", "baz"])), Some(false));
        assert_eq!(aft.truth_of_atom(&make_atom("quux", ["baz", "foo"])), Some(false));
        assert_eq!(aft.truth_of_atom(&make_atom("quux", ["baz", "bar"])), Some(false));
        assert_eq!(aft.truth_of_atom(&make_atom("quux", ["baz", "baz"])), Some(false));
    }
}





/***** ERRORS *****/
/// Defines logic errors over the quantification in rules.
#[derive(Debug)]
pub enum Error {
    QuantifyOverflow { rule: Rule, max: usize },
}
impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        use Error::*;
        match self {
            QuantifyOverflow { rule, max } => {
                write!(f, "Rule '{rule}' has more than {max} arguments in its atoms; cannot quantify over its variables")
            },
        }
    }
}
impl error::Error for Error {}





/***** HELPER FUNCTIONS *****/
/// Generates a string that represents an instantiated rule.
///
/// # Arguments
/// - `rule`: The actual [`Rule`] to instantiate.
/// - `assign`: Some assignment for the rule's variables.
///
/// # Returns
/// A [`String`] representing the format.
fn format_rule_assign<const LEN: usize>(rule: &Rule, assign: &StackVec<LEN, Ident>) -> String {
    let mut buf: String = String::new();

    // Consequences
    let mut i: usize = 0;
    for cons in rule.consequences.values() {
        buf.push_str(&format!(
            "{}({})",
            cons.ident,
            cons.args
                .iter()
                .flat_map(|a| a.args.values())
                .map(|_| {
                    let arg: String = assign[i].to_string();
                    i += 1;
                    arg
                })
                .collect::<Vec<String>>()
                .join(",")
        ));
    }

    // Write the arrow
    buf.push_str(" :- ");

    // Antecedents
    for ante in rule.tail.iter().flat_map(|t| t.antecedents.values()) {
        buf.push_str(&format!(
            "{}({})",
            ante.atom().ident,
            ante.atom()
                .args
                .iter()
                .flat_map(|a| a.args.values())
                .map(|_| {
                    let arg: String = assign[i].to_string();
                    i += 1;
                    arg
                })
                .collect::<Vec<String>>()
                .join(",")
        ));
    }

    // Done
    buf.push('.');
    buf
}

/// Generates a string that represents an instantiated atom.
///
/// # Arguments
/// - `atom`: The actual [`Atom`] to instantiate.
/// - `assign`: Some assignment for the atom's variables.
///
/// # Returns
/// A [`String`] representing the format.
fn format_atom_assign<const LEN: usize>(atom: &Atom, assign: &StackVec<LEN, Ident>) -> String {
    let mut i: usize = 0;
    format!(
        "{}({})",
        atom.ident,
        atom.args
            .iter()
            .flat_map(|a| a.args.values())
            .map(|_| {
                let arg: String = assign[i].to_string();
                i += 1;
                arg
            })
            .collect::<Vec<String>>()
            .join(",")
    )
}





/***** ITERATORS *****/
/// Iterates over either a specific identifier (in the case of an atom), or a quantification of all constants (in case of a variable).
#[derive(Clone, Copy, Debug)]
pub enum AntecedentQuantifier<'c> {
    /// It's a specific identifier we keep on yielding. The first `usize` is the length of the `consts`-array (i.e., Herbrand 0-base), and the second is the current number of yielded elements.
    Atom(usize, Ident, usize),
    /// It's a full set of constants to quantify, with possible element-wise repetitions and full iterator repetition (respectively).
    /// Finally, stores the "index" of this variable, which determines the sequence of constants used to generate it.
    Var(&'c IndexSet<Ident>, (usize, usize, usize), usize),
}
impl<'c> AntecedentQuantifier<'c> {
    /// Returns the next [`Ident`] in this quantifier.
    ///
    /// # Arguments
    /// - `n_vars`: The total number of variables that this quantifier quantifies over. Determines amounts of repetitions.
    ///
    /// # Returns
    /// The next [`Ident`] in line.
    #[cfg_attr(debug_assertions, track_caller)]
    pub fn next(&mut self, n_vars: usize) -> Option<Ident> {
        match self {
            Self::Atom(consts_len, ident, idx) => {
                // Yield as long as we need to
                if *idx < consts_len.pow(n_vars as u32) {
                    *idx += 1;
                    Some(*ident)
                } else {
                    None
                }
            },
            Self::Var(consts, (inner_idx, idx, outer_idx), i) => {
                // Check if `i` isn't too large
                #[cfg(debug_assertions)]
                if *i >= n_vars {
                    panic!("Internal i ({i}) is too large for given number of variabels ({n_vars})");
                }

                // Compute the bounds.
                // We scale from essentially doing `111111...333333`, to `111222...222333`, to `123123...123123`
                //
                // Some examples:
                // ```plain
                // 123, three variables:
                // 111111111222222222333333333      (outer = 1, inner = 9)
                // 111222333111222333111222333      (outer = 3, inner = 3)
                // 123123123123123123123123123      (outer = 9, inner = 1)
                //
                // 12, four variables
                // 1111111122222222                 (outer = 1, inner = 8)
                // 1111222211112222                 (outer = 2, inner = 4)
                // 1122112211221122                 (outer = 4, inner = 2)
                // 1212121212121212                 (outer = 8, inner = 1)
                //
                // 1234, two variables
                // 1111222233334444                 (outer = 1, inner = 4)
                // 1234123412341234                 (outer = 4, inner = 1)
                // ```
                // From this we can observe that the outer grows exponentially over the Herbrand base size, whereas the inner grows inverse exponentially.
                let consts_len: usize = consts.len();
                let n_inner_repeats: usize = consts.len().pow((n_vars - 1 - *i) as u32);
                let n_outer_repeats: usize = consts.len().pow(*i as u32);

                // Consider whether to return the current element or advance any of the counters
                loop {
                    if *inner_idx < n_inner_repeats && *idx < consts_len {
                        // We're in the inner-repeat loop
                        *inner_idx += 1;
                        break Some(consts[*idx]);
                    } else if *idx < consts_len {
                        // We're advancing to the next element
                        *inner_idx = 0;
                        *idx += 1;
                        continue;
                    } else if *outer_idx < n_outer_repeats {
                        // We're advancing to the next iterator repetition
                        *inner_idx = 0;
                        *idx = 0;
                        *outer_idx += 1;
                        continue;
                    } else {
                        // Nothing to return anymore
                        break None;
                    }
                }
            },
        }
    }

    /// Resets the iterator to nothing yielded.
    #[inline]
    pub fn reset(&mut self) {
        match self {
            Self::Atom(_, _, i) => *i = 0,
            Self::Var(_, (inner_idx, idx, outer_idx), _) => {
                *inner_idx = 0;
                *idx = 0;
                *outer_idx = 0;
            },
        }
    }
}





/***** LIBRARY *****/
// Interpreter extensions for the [`Spec`].
impl Spec {
    /// Computes a simplified version of the _Herbrand base_ of the specification.
    ///
    /// We like to call this the "0-base", as it is simply a list of all constants (i.e., atoms
    /// with arity 0) in the spec. This is useful for us because $Datalog^\neg$ doesn't do nesting
    /// of functions, so we only need to quantify over the (relevant) constants.
    ///
    /// # Returns
    /// An ordered [`IndexSet`] with the consequents as we encountered them. Not necessary, but does make testing quite some easier.
    fn find_0_base(&self) -> IndexSet<Ident> {
        let mut base: IndexSet<Ident> = IndexSet::new();
        for rule in &self.rules {
            // Note down atomic consequents _or_ its arguments (these are always constants)
            for cons in rule.consequences.values() {
                // Consider if the consequent has arguments
                if let Some(args) = &cons.args {
                    if !args.args.is_empty() {
                        // We found some of those sweet arguments
                        base.extend(args.args.values().filter_map(|v| match v {
                            AtomArg::Atom(a) => Some(*a),
                            AtomArg::Var(_) => None,
                        }));
                        continue;
                    }
                }

                // It doesn't, so note it itself down
                if base.len() >= base.capacity() {
                    base.reserve(16);
                }
                base.insert(cons.ident);
            }

            // Do the same for antecedents, if there are any
            for ante in rule.tail.iter().flat_map(|t| t.antecedents.values()) {
                // Consider if the antecedent has arguments
                if let Some(args) = &ante.atom().args {
                    if !args.args.is_empty() {
                        // We found some of those sweet arguments
                        base.extend(args.args.values().filter_map(|v| match v {
                            AtomArg::Atom(a) => Some(*a),
                            AtomArg::Var(_) => None,
                        }));
                        continue;
                    }
                }

                // It doesn't, so note it itself down
                if base.len() >= base.capacity() {
                    base.reserve(16);
                }
                base.insert(ante.atom().ident);
            }
        }
        base
    }

    /// Performs forward derivation of the Spec.
    ///
    /// In the paper, this is called the _immediate consequence operator_. It is simply defined as
    /// the "forward derivation" of all rules, where we note the rule's consequences as derived if we
    /// observe all of its antecedents to be in the given interpretation.
    ///
    /// Note that the paper makes a point to consider all negative antecedents to be "new" atoms,
    /// i.e., we must observe negative atoms explicitly instead of the absence of positives.
    ///
    /// # Generics
    /// - `LEN`: Some buffer length to use internally. This determines the maximum total number of arguments among _all_ consequences and antecedents in a rule. **Must be > 0.**
    ///
    /// # Arguments
    /// - `int`: Some [`Interpretation`] to derive from. Note that we alternate this one with `res`, so after derivation, this will hold the interpretation for the second-to-last saturation run.
    /// - `res`: Another [`Interpretation`] that we populate with derived facts.
    ///
    /// # Returns
    /// Whether any new facts were derived or not.
    ///
    /// # Errors
    /// This function can error if the total number of arguments in a rule exceeds `LEN`,
    pub fn immediate_consequence<const LEN: usize>(&self, int: &mut Interpretation, res: &mut Interpretation) -> Result<bool, Error> {
        debug!("Running immediate consequent transformation");

        // Compute the Herbrand 0-base
        let consts: IndexSet<Ident> = self.find_0_base();

        // Sync the interpretations to carry over existing knowledge
        *res = int.clone();

        // This transformation is saturating, so continue until the database did not change anymore.
        // NOTE: Monotonic because we can never remove truths, inferring the same fact does not count as a change and we are iterating over a Herbrand instantiation so our search space is finite (for $Datalog^\neg$, at least).
        let mut changed: bool = true;
        let mut i: usize = 0;
        while i == 0 || int != res {
            changed = false;
            i += 1;
            trace!("Derivation run {i} starting");

            // Swap the instances to use the previous' run's result as input to this one
            if i > 0 {
                std::mem::swap(int, res);
            }

            // Go thru da rules
            for rule in &self.rules {
                // Run the rule's derivation
                changed |= rule.immediate_consequence::<64>(&consts, int, res)?;
            }
        }

        // Done!
        trace!("Done saturating immediate consequent transformation (took {i} passes)");
        Ok(changed)
    }

    /// Performs the stable transformation for the given Interpretation.
    ///
    /// Concretely, this takes the complement of the Interpretation in this Spec's _full_ Herbrand-base (which includes atoms with arity > 0), and then negates all those atoms.
    ///
    /// This produces an interpretation with atoms "assumed to be false". The specific use is described in the paper.
    ///
    /// # Generics
    /// - `LEN`: Some buffer length to use internally. This determines the maximum total number of arguments among _all_ consequences and antecedents in a rule. **Must be > 0.**
    ///
    /// # Arguments
    /// - `consts`: An already computed zero-base to use.
    /// - `int`: Some [`Interpretation`] to transform.
    /// - `res`: Another [`Interpretation`] that we populate with the transformed version of `int`.
    ///
    /// # Errors
    /// This function can error if the total number of arguments in a rule exceeds `LEN`.
    pub fn stable_transformation<const LEN: usize>(
        &self,
        consts: &IndexSet<Ident>,
        int: &mut Interpretation,
        res: &mut Interpretation,
    ) -> Result<(), Error> {
        debug!("Running stable transformation for {int}");

        // Go through the rules
        res.clear();
        let mut iters: StackVec<LEN, AntecedentQuantifier> = StackVec::new();
        let (mut n_cons_args, mut n_vars): (usize, usize) = (0, 0);
        let mut assign: StackVec<LEN, Ident> = StackVec::new();
        let mut atom_assign: StackVec<LEN, Ident> = StackVec::new();
        for rule in &self.rules {
            trace!("Considering rule '{rule}'");

            // Go through the assignments
            rule.find_iters(consts, &mut iters, &mut n_cons_args, &mut n_vars)?;
            'assign: loop {
                assign.clear();
                for iter in &mut iters {
                    assign.push(match iter.next(n_vars) {
                        Some(next) => next,
                        None => break 'assign,
                    });
                }
                trace!("[Rule '{rule}'] Considering instantiation '{}'", format_rule_assign(rule, &assign));

                // Go through the consequences first to find all applicable atoms
                let mut assign_iter = assign.iter();
                for cons in rule.consequences.values() {
                    // Collect a specific assignment for this atom only
                    atom_assign.clear();
                    atom_assign.extend((&mut assign_iter).take(cons.args.as_ref().map(|a| a.args.len()).unwrap_or(0)).cloned());

                    // See if this atom exists in the interpretation
                    if !matches!(int.truth_of_atom_by_hash(int.hash_atom_with_assign(&cons.ident, atom_assign.iter().cloned())), Some(true)) {
                        trace!(
                            "[Rule '{rule}' // '{}'] Consequent '{}' does not appear in interpretation, creating negation in consequent",
                            format_rule_assign(rule, &assign),
                            format_atom_assign(cons, &atom_assign),
                        );

                        // It doesn't; so write its negation to the interpretation
                        let mut cons: Atom = cons.clone();
                        for (i, arg) in cons.args.iter_mut().flat_map(|a| a.args.values_mut()).enumerate() {
                            *arg = AtomArg::Atom(atom_assign[i]);
                        }
                        // Don't forget to write it negated
                        res.learn(cons, false);
                    }

                    // Then also consider the atom's arguments (which are all constants, much easier!)
                    for arg in &atom_assign {
                        // If it's in the interpretation, then we _don't_ complement it
                        let arg: Atom = Atom { ident: *arg, args: None };
                        if !matches!(int.truth_of_atom(&arg), Some(true)) {
                            trace!(
                                "[Rule '{rule}' // '{}'] Argument '{arg}' does not appear in interpretation, creating negation in consequent",
                                format_rule_assign(rule, &assign)
                            );
                            // Don't forget to write it negated
                            res.learn(arg, false);
                        }
                    }
                }

                // Then do all antecedents
                for ante in rule.tail.iter().flat_map(|t| t.antecedents.values()) {
                    // Collect a specific assignment for this atom only
                    atom_assign.clear();
                    atom_assign.extend((&mut assign_iter).take(ante.atom().args.as_ref().map(|a| a.args.len()).unwrap_or(0)).cloned());

                    // See if this atom exists in the interpretation
                    if !matches!(int.truth_of_atom_by_hash(int.hash_atom_with_assign(&ante.atom().ident, atom_assign.iter().cloned())), Some(true)) {
                        trace!(
                            "[Rule '{rule}' // '{}'] Antecedent '{}' does not appear in interpretation, creating negation in consequent",
                            format_rule_assign(rule, &assign),
                            format_atom_assign(ante.atom(), &atom_assign),
                        );

                        // It doesn't; so write its negation to the interpretation
                        let mut ante: Atom = ante.atom().clone();
                        for (i, arg) in ante.args.iter_mut().flat_map(|a| a.args.values_mut()).enumerate() {
                            *arg = AtomArg::Atom(atom_assign[i]);
                        }
                        // Don't forget to write it negated
                        res.learn(ante, false);
                    }

                    // Then also consider the atom's arguments (which are all constants, much easier!)
                    for arg in &atom_assign {
                        // If it's in the interpretation, then we _don't_ complement it
                        let arg: Atom = Atom { ident: *arg, args: None };
                        if !matches!(int.truth_of_atom(&arg), Some(true)) {
                            trace!(
                                "[Rule '{rule}' // '{}'] Argument '{arg}' does not appear in interpretation, creating negation in consequent",
                                format_rule_assign(rule, &assign)
                            );
                            // Don't forget to write it negated
                            res.learn(arg, false);
                        }
                    }
                }
            }
        }

        // Done, complement computed
        Ok(())
    }
}



// Interpreter extensions for the [`Rule`].
impl Rule {
    /// Finds a set of [`AntecedentQuantifier`] that can be used to do quantification over a rule.
    ///
    /// # Arguments
    /// - `consts`: A set of _constants_ (atoms with arity 0) we found in the parent [`Spec`].
    /// - `iters`: A buffer to store the iterators in. It is guaranteed that, when done, the first [`None`] indicates the current end of the buffer.
    /// - `n_cons`: Keeps track of the number of arguments in the consequences. Because why not, if we're iterating anyway.
    /// - `n_vars`: Keeps track of how many iterators actually quantify variables.
    ///
    /// # Errors
    /// This function can error if the total number of arguments in the function exceeds `LEN`,
    fn find_iters<'c, const LEN: usize>(
        &self,
        consts: &'c IndexSet<Ident>,
        iters: &mut StackVec<LEN, AntecedentQuantifier<'c>>,
        n_cons: &mut usize,
        n_vars: &mut usize,
    ) -> Result<(), Error> {
        // A shadow buffer we use to keep track of the variables we've already seen.
        iters.clear();
        *n_cons = 0;
        *n_vars = 0;
        let mut vars: StackVec<LEN, (Ident, usize)> = StackVec::new();

        // Examine everything in one big happy heap
        'arg: for arg in self
            .consequences
            .values()
            .flat_map(|v| v.args.iter().flat_map(|a| a.args.values()))
            .inspect(|_| *n_cons += 1)
            .chain(self.tail.iter().flat_map(|t| t.antecedents.values().flat_map(|v| v.atom().args.iter().flat_map(|a| a.args.values()))))
        {
            // Catch out-of-bounds
            if iters.len() >= iters.capacity() {
                return Err(Error::QuantifyOverflow { rule: self.clone(), max: LEN });
            }

            // See if we're dealing with an atom or a VARIABLE
            match arg {
                AtomArg::Atom(a) => {
                    // Insert an atom iterator at the end of the current list
                    iters.push(AntecedentQuantifier::Atom(consts.len(), *a, 0));
                },
                AtomArg::Var(v) => {
                    // Check if we've seen this variable before somewhere
                    for i in 0..*n_vars {
                        // SAFETY: We promise ourselves that we only see [`None`]s after the list ended, i.e., i >= vars_end. But the range prevents this from happening.
                        let (var, idx): (Ident, usize) = vars[i];
                        if v == &var {
                            // We have seen this before! So insert this variable's quantifier.
                            iters.push(iters[idx]);
                            continue 'arg;
                        }

                        // Else, keep on searching
                    }

                    // We haven't seen this variable before. Add a new quantifier.
                    vars.push((*v, iters.len()));
                    iters.push(AntecedentQuantifier::Var(consts, (0, 0, 0), *n_vars));
                    *n_vars += 1;
                },
            }
        }

        // Inject a phony argument if none were found. This is important to still derive constants.
        if iters.is_empty() {
            iters.push(AntecedentQuantifier::Atom(
                1,
                Ident { value: Span::new("<auto generated by Rule::find_iters()>", "you should never see this :)") },
                0,
            ));
            *n_cons += 1;
        }

        // Alrighty done
        Ok(())
    }

    /// Performs forward derivation of the Rule.
    ///
    /// In the paper, this is called the _immediate consequence operator_. It is simply defined as
    /// the "forward derivation" of a rule, where we note the rule's consequences as derived if we
    /// observe all of its antecedents to be in the given interpretation.
    ///
    /// Note that the paper makes a point to consider all negative antecedents to be "new" atoms,
    /// i.e., we must observe negative atoms explicitly instead of the absence of positives.
    ///
    /// # Generics
    /// - `LEN`: Some buffer length to use internally. This determines the maximum total number of arguments among _all_ consequences and antecedents in the rule. **Must be > 0.**
    ///
    /// # Arguments
    /// - `cons`: A set of _constants_ (atoms with arity 0) we found in the parent [`Spec`].
    /// - `int`: Some [`Interpretation`] to derive from.
    /// - `res`: Another [`Interpretation`] that we populate with derived facts.
    ///
    /// # Returns
    /// Whether any new facts were derived or not.
    ///
    /// # Errors
    /// This function can error if the total number of arguments in the function exceeds `LEN`,
    pub fn immediate_consequence<const LEN: usize>(
        &self,
        consts: &IndexSet<Ident>,
        int: &Interpretation,
        res: &mut Interpretation,
    ) -> Result<bool, Error> {
        debug!("Running immediate consequent operator for rule '{self}'");

        // Create a set of iterators that quantify over any variables found in the rule
        let mut n_vars: usize = 0;
        let mut n_cons_args: usize = 0;
        let mut iters: StackVec<LEN, AntecedentQuantifier> = StackVec::new();
        self.find_iters(consts, &mut iters, &mut n_cons_args, &mut n_vars)?;

        // Now hit the road jack no more no more (or something along those lines)
        let mut changed: bool = false;
        let mut assign: StackVec<LEN, Ident> = StackVec::new();
        'instance: loop {
            // Find the next assignment
            assign.clear();
            for iter in &mut iters {
                assign.push(match iter.next(n_vars) {
                    Some(next) => next,
                    None => break 'instance,
                });
            }
            trace!("[Rule '{self}'] Considering instantiation '{}'", format_rule_assign(self, &assign));

            // See if we can find the concrete antecedents in the interpretation. No antecedents? Rule trivially accepted!
            let mut ant_assign = assign.iter().skip(n_cons_args);
            for ante in self.tail.iter().flat_map(|t| t.antecedents.values()) {
                // Get the polarity of the literal
                let polarity: bool = matches!(ante, Literal::Atom(_));

                // Check if
                if int.truth_of_atom_by_hash(int.hash_atom_with_assign(
                    &ante.atom().ident,
                    (&mut ant_assign).take(ante.atom().args.as_ref().map(|a| a.args.len()).unwrap_or(0)).map(|a| *a),
                )) != Some(polarity)
                {
                    // The antecedant (as a literal, so negation taken into account) is not true; cannot derive this fact
                    trace!(
                        "[Rule '{self}' // '{}'] Antecedent '{ante}' not true in the interpretation; rule does not hold",
                        format_rule_assign(self, &assign)
                    );
                    continue 'instance;
                }
            }
            trace!("[Rule '{self}' // '{}'] All antecedents hold", format_rule_assign(self, &assign));

            // If all antecedants were in the interpretation, then derive the consequents.
            let mut con_assign = assign.iter();
            for cons in self.consequences.values() {
                // Create the instantiation of the consequent
                let mut cons: Atom = cons.clone();
                for arg in cons.args.iter_mut().flat_map(|a| a.args.values_mut()) {
                    *arg = AtomArg::Atom(con_assign.next().cloned().unwrap());
                }

                // Now derive it
                trace!("[Rule '{self}' // '{}'] Deriving '{cons}'", format_rule_assign(self, &assign));
                if !matches!(res.learn(cons, true), Some(true)) {
                    changed = true;
                }
            }
        }

        // Done, return whether this had any effect
        Ok(changed)

        // debug!("Running immediate consequent transformation");

        // // This transformation is saturating, so continue until no rules are triggered anymore.
        // // NOTE: Monotonic because we can never remove truths, inferring the same fact does not count as a change and we are iterating over a Herbrand instantiation so our search space is finite (for $Datalog^\neg$, at least).
        // let mut changed: bool = true;
        // let mut i: usize = 0;
        // while changed {
        //     changed = false;
        //     i += 1;

        //     // Go thru da rules
        //     // NOTE: HerbrandInstantiationIterator is not an official iterator because it doesn't GAT. So we do this manually.
        //     'rule: while let Some(rule) = rules.next() {
        //         trace!("[{i}] Running immediate consequent transformation for '{rule}'");

        //         // See if we can find the antecedents in the interpretation. No antecedents? Rule trivially accepted!
        //         for ante in rule.tail.iter().map(|t| t.antecedents.values()).flatten() {
        //             if !matches!(int.truth_of_lit(ante), Some(true)) {
        //                 // The antecedant is not true; cannot derive this fact
        //                 trace!("[{i}] Antecedent '{ante}' not true in the interpretation; Rule '{rule}' does not hold");
        //                 continue 'rule;
        //             }
        //         }

        //         // If all antecedants were in the interpretation, then derive the consequents.
        //         for cons in rule.consequences.values() {
        //             trace!("[{i}] Deriving '{cons}' from '{rule}' (all antecedents explicitly hold)");
        //             if !matches!(int.learn(cons.clone(), true), Some(true)) {
        //                 changed = true;
        //             }
        //         }
        //     }
        // }

        // // Done!
        // trace!("Done saturating immediate consequent transformation (took {i} passes)");
    }
}
