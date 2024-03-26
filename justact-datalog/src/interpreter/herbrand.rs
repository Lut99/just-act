//  HERBRAND.rs
//    by Lut99
//
//  Created:
//    21 Mar 2024, 10:55:27
//  Last edited:
//    26 Mar 2024, 10:39:25
//  Auto updated?
//    Yes
//
//  Description:
//!   Implements iterators for the Herbrand instantiation of a program.
//

use indexmap::IndexSet;

use crate::ast::{AtomArg, Ident, Spec};


/***** TESTS *****/
#[cfg(all(test, feature = "derive"))]
mod tests {
    use ast_toolkit_punctuated::Punctuated;
    use ast_toolkit_span::Span;
    use justact_datalog_derive::datalog;

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
        let mut iter = HerbrandUniverse::find_0_base(&empty);
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);

        // Spec with only constants (one, first)
        let one: Spec = datalog! {
            #![crate]
            foo.
        };
        let mut iter = HerbrandUniverse::find_0_base(&one);
        assert_eq!(iter.next(), Some(make_ident("foo")));
        assert_eq!(iter.next(), None);

        // Multiple constants
        let consts: Spec = datalog! {
            #![crate]
            foo. bar. baz.
        };
        let mut iter = HerbrandUniverse::find_0_base(&consts);
        assert_eq!(iter.next(), Some(make_ident("foo")));
        assert_eq!(iter.next(), Some(make_ident("bar")));
        assert_eq!(iter.next(), Some(make_ident("baz")));
        assert_eq!(iter.next(), None);

        // Spec with arity-1 atoms (functions)
        let funcs: Spec = datalog! {
            #![crate]
            foo(bar). bar(baz). baz(quz).
        };
        let mut iter = HerbrandUniverse::find_0_base(&funcs);
        assert_eq!(iter.next(), Some(make_ident("bar")));
        assert_eq!(iter.next(), Some(make_ident("baz")));
        assert_eq!(iter.next(), Some(make_ident("quz")));
        assert_eq!(iter.next(), None);

        // Mixed arity
        let arity: Spec = datalog! {
            #![crate]
            foo. bar(). baz(quz). quz(qux, quux). corge(grault, garply, waldo).
        };
        let mut iter = HerbrandUniverse::find_0_base(&arity);
        assert_eq!(iter.next(), Some(make_ident("foo")));
        assert_eq!(iter.next(), Some(make_ident("bar")));
        assert_eq!(iter.next(), Some(make_ident("quz")));
        assert_eq!(iter.next(), Some(make_ident("qux")));
        assert_eq!(iter.next(), Some(make_ident("quux")));
        assert_eq!(iter.next(), Some(make_ident("grault")));
        assert_eq!(iter.next(), Some(make_ident("garply")));
        assert_eq!(iter.next(), Some(make_ident("waldo")));
        assert_eq!(iter.next(), None);

        // Full rules
        let rules: Spec = datalog! {
            #![crate]
            foo. bar(baz). quz(X) :- bar(X), qux(quux).
        };
        let mut iter = HerbrandUniverse::find_0_base(&rules);
        assert_eq!(iter.next(), Some(make_ident("foo")));
        assert_eq!(iter.next(), Some(make_ident("baz")));
        assert_eq!(iter.next(), Some(make_ident("quux")));
        assert_eq!(iter.next(), None);
    }
}





/***** ITERATORS *****/
/// Given a [`Spec`], finds the 0-base of it.
///
/// This is simply all constants (i.e., atoms with arity 0) in the [`Spec`].
pub struct Herbrand0BaseIter<'s> {
    /// A reference to the Spec such that we can compute the size if we want.
    spec: &'s Spec,
    /// The complex iterator doing the work.
    iter: Box<dyn 's + Iterator<Item = Ident>>,
}
impl<'s> Herbrand0BaseIter<'s> {
    /// Constructor for the Herbrand0BaseIter.
    ///
    /// # Arguments
    /// - `spec`: The [`Spec`] to iterate over.
    ///
    /// # Returns
    /// A new Herbrand0BaseIter ready to rock.
    #[inline]
    pub fn new(spec: &'s Spec) -> Self {
        Self {
            spec,
            // We iterate over all rules first...
            iter: Box::new(spec.rules.iter().flat_map(|r| {
                // ...then we iterate over all consequences...
                r.consequences
                    .values()
                    .flat_map(|v| match &v.args {
                        // ...either its arguments, as the consequent is no constant...
                        Some(args) => {
                            let iter: Box<dyn 's + Iterator<Item = Ident>> = if !args.args.is_empty() {
                                Box::new(args.args.values().filter_map(|v| match v { AtomArg::Atom(a) => Some(*a), AtomArg::Var(_) => None }))
                            } else {
                                // Oop still empty
                                Box::new(Some(v.ident).into_iter())
                            };
                            iter
                        },
                        // ...or the consequent itself if it has no arguments...
                        None => {
                            let iter: Box<dyn 's + Iterator<Item = Ident>> = Box::new(Some(v.ident).into_iter());
                            iter
                        },
                    })
                    // ...and then the rule's antecedents...
                    .chain(r.tail.iter().flat_map(|t| {
                        t.antecedents.values().flat_map(|v| match &v.atom().args {
                            // ...where we also do its arguments...
                            Some(args) => {
                                let iter: Box<dyn 's + Iterator<Item = Ident>> = if !args.args.is_empty() {
                                    Box::new(args.args.values().filter_map(|v| match v { AtomArg::Atom(a) => Some(*a), AtomArg::Var(_) => None }))
                                } else {
                                    // Oop still empty
                                    Box::new(Some(v.atom().ident).into_iter())
                                };
                                iter
                            },
                            // ...or the antecedent itself
                            None => {
                                let iter: Box<dyn 's + Iterator<Item = Ident>> = Box::new(Some(v.atom().ident).into_iter());
                                iter
                            },
                        })
                    }))
            })),
        }
    }
}
impl<'s> Iterator for Herbrand0BaseIter<'s> {
    type Item = Ident;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> { self.iter.next() }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let mut n_consts: usize = 0;
        for rule in &self.spec.rules {
            for cons in rule.consequences.values() {
                match &cons.args {
                    // The consequence is no constant, but its arguments are
                    Some(args) => n_consts += args.args.len(),
                    // The consequence itself is a constant
                    None => n_consts += 1,
                }
            }
            for ante in rule.tail.iter().map(|t| t.antecedents.values()).flatten() {
                match &ante.atom().args {
                    // The antecedent is no constant, but its arguments are
                    Some(args) => n_consts += args.args.len(),
                    // The antecedent itself is a constant
                    None => n_consts += 1,
                }
            }
        }
        (n_consts, Some(n_consts))
    }
}





/***** LIBRARY *****/
/// Represents our knowlede of a particular Herbrand Universe over a spec.
///
/// A Herbrand Universe is de-facto the universe of things to quantify over that might net in useful derivation.
///
/// We distinguish three types:
/// - A _Herbrand base_ that only contains interesting _constants_ (i.e., atoms with arity 0) in the spec (dubbed the "Herbrand 0-base");
/// - A full _Herbrand base_ that is the Herbrand 0-base plus all possible combinations of X-arity atoms with 0-arity arguments (ignoring variables); and
/// - A _Herbrand instantiation_, which is the rules in the spec but with (variable-respecting) concretization of all rules such that no variables occur anymore.
#[derive(Clone, Debug)]
pub struct HerbrandUniverse<'s> {
    // Data
    /// The Spec of which this HerbrandUniverse is a part.
    spec:   &'s Spec,
    /// Defines the Herbrand 0-base of the given Spec. We always compute this, since we need it for the other two bases.
    consts: IndexSet<Ident>,
    // Buffers
}
impl<'s> HerbrandUniverse<'s> {
    /// Constructor for the HerbrandUniverse.
    ///
    /// # Arguments
    /// - `spec`: The [`Spec`] to base it around.
    ///
    /// # Returns
    /// A new HerbrandUniverse that can be used to find all three bases.
    #[inline]
    pub fn new(spec: &'s Spec) -> Self { Self { spec, consts: Self::find_0_base(spec).collect() } }

    /// Returns the inner [`Spec`].
    ///
    /// # Returns
    /// A reference to the inner [`Spec`].
    #[inline]
    pub fn spec(&self) -> &'s Spec { self.spec }
}
impl<'s> HerbrandUniverse<'s> {
    /// Finds the 0-base of the given [`Spec`].
    ///
    /// This is an associated function as we are not depending on the 0-base (...wow).
    ///
    /// # Arguments
    /// - `spec`: The [`Spec`] to compute the 0-base of.
    ///
    /// # Returns
    /// A `Herbrand0BaseIter` that computes the zero-base without allocations.
    #[inline]
    pub fn find_0_base(spec: &'s Spec) -> Herbrand0BaseIter<'s> { Herbrand0BaseIter::new(spec) }
}
