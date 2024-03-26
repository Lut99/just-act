//  HERBRAND.rs
//    by Lut99
//
//  Created:
//    21 Mar 2024, 10:55:27
//  Last edited:
//    26 Mar 2024, 18:12:26
//  Auto updated?
//    Yes
//
//  Description:
//!   Implements iterators for the Herbrand instantiation of a program.
//

use std::borrow::Cow;
use std::iter::{Chain, FlatMap, Map, Repeat, Take};

use ast_toolkit_punctuated::Punctuated;
use indexmap::IndexSet;

use crate::ast::{Atom, AtomArg, AtomArgs, Comma, Ident, Literal, Parens, Rule, RuleAntecedents, Span, Spec};


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
        let universe = HerbrandUniverse::new(&empty);
        let mut iter = universe.zero_base().iter();
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);

        // Spec with only constants (one, first)
        let one: Spec = datalog! {
            #![crate]
            foo.
        };
        let universe = HerbrandUniverse::new(&one);
        let mut iter = universe.zero_base().iter();
        assert_eq!(iter.next(), Some(&make_ident("foo")));
        assert_eq!(iter.next(), None);

        // Multiple constants
        let consts: Spec = datalog! {
            #![crate]
            foo. bar. baz.
        };
        let universe = HerbrandUniverse::new(&consts);
        let mut iter = universe.zero_base().iter();
        assert_eq!(iter.next(), Some(&make_ident("foo")));
        assert_eq!(iter.next(), Some(&make_ident("bar")));
        assert_eq!(iter.next(), Some(&make_ident("baz")));
        assert_eq!(iter.next(), None);

        // Duplicate constants
        let dups: Spec = datalog! {
            #![crate]
            foo. foo. bar.
        };
        let universe = HerbrandUniverse::new(&dups);
        let mut iter = universe.zero_base().iter();
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
        let universe = HerbrandUniverse::new(&funcs);
        let mut iter = universe.zero_base().iter();
        assert_eq!(iter.next(), Some(&make_ident("bar")));
        assert_eq!(iter.next(), Some(&make_ident("baz")));
        assert_eq!(iter.next(), Some(&make_ident("quz")));
        assert_eq!(iter.next(), None);

        // Mixed arity
        let arity: Spec = datalog! {
            #![crate]
            foo. bar(). baz(quz). quz(qux, quux). corge(grault, garply, waldo).
        };
        let universe = HerbrandUniverse::new(&arity);
        let mut iter = universe.zero_base().iter();
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
        let universe = HerbrandUniverse::new(&rules);
        let mut iter = universe.zero_base().iter();
        assert_eq!(iter.next(), Some(&make_ident("foo")));
        assert_eq!(iter.next(), Some(&make_ident("baz")));
        assert_eq!(iter.next(), Some(&make_ident("quux")));
        assert_eq!(iter.next(), None);
    }
}





/***** HELPER ITERATORS *****/
/// Repeats a set in a scaling way.
///
/// In particular, repeats the inner elements M times, and the outer iterator (including repetitions) N times.
struct RepeatIter<'s, I> {
    /// The set to repeat.
    set: &'s IndexSet<I>,
    /// The current (i, inner, outer) indices.
    idx: (usize, usize, usize),
    /// The given (inner, outer) repetition amounts.
    max: (usize, usize),
}
impl<'s, I> RepeatIter<'s, I> {
    /// Constructor for the RepeatIter.
    ///
    /// # Arguments
    /// - `set`: The [`IndexSet`] to repeat.
    /// - `m`: The number of times every _element_ is repeated.
    /// - `n`: The number of times the whole _iterator_ is repeated (including the `m` repetitions).
    ///
    /// # Returns
    /// A new RepeatIter.
    #[inline]
    pub fn new(set: &'s IndexSet<I>, m: usize, n: usize) -> Self { Self { set, idx: (0, 0, 0), max: (m, n) } }
}
impl<'s, I> Iterator for RepeatIter<'s, I> {
    type Item = &'s I;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let (m, i, n) = self.idx;
        if m < self.max.0 && i < self.set.len() {
            // Repeat this same element all the time
            self.idx.0 += 1;
            Some(&self.set[i])
        } else if i < self.set.len() {
            // Move to the next element
            self.idx.0 = 0;
            self.idx.1 += 1;
            self.next()
        } else if n < self.max.1 {
            // Repeat the whole iterator
            self.idx.0 = 0;
            self.idx.1 = 0;
            self.idx.2 += 1;
            self.next()
        } else {
            // Out-of-repetitions
            None
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len: usize = self.set.len() * self.max.0 * self.max.1;
        (len, Some(len))
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
    fn new(spec: &'s Spec) -> Self {
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



/// Type alias for the antecedent -> values map function.
type TailToValues<'s> = fn(&'s RuleAntecedents) -> ast_toolkit_punctuated::normal::Values<'s, Literal, Comma>;

/// Type alias for the literal -> atom map function.
type LiteralToAtom<'s> = fn(&'s Literal) -> &'s Atom;

/// Type alias for the atom iterator in the HerbrandXBaseIter.
type AtomIter<'s> = Chain<
    ast_toolkit_punctuated::normal::Values<'s, Atom, Comma>,
    Map<
        FlatMap<std::option::Iter<'s, RuleAntecedents>, ast_toolkit_punctuated::normal::Values<'s, Literal, Comma>, TailToValues<'s>>,
        LiteralToAtom<'s>,
    >,
>;

/// Given a [`Spec`], finds the X-base (i.e., full Herbrand base) of it.
///
/// This is simply all constants (i.e., atoms with arity 0) in the [`Spec`] plus all atoms with arity > 0 with their arguments substituted for all possible combinations of constants.
pub struct HerbrandXBaseIter<'s, 'u> {
    /// A reference to the Spec such that we can compute the size if we want.
    spec:   &'s Spec,
    /// A reference to the set of constants from the spec that lives in the parent [`HerbrandUniverse`].
    consts: &'u IndexSet<Ident>,

    /// The iterator over the rules in the spec.
    rules: std::slice::Iter<'s, Rule>,
    /// The iterator over the atoms in the current rule.
    atoms: Option<AtomIter<'s>>,
    /// A buffer with iterators we use to quantify.
    iters: Vec<RepeatIter<'u, Ident>>,
}
impl<'s, 'u> HerbrandXBaseIter<'s, 'u> {
    /// Constructor for the HerbrandXBaseIter.
    ///
    /// # Arguments
    /// - `spec`: The [`Spec`] to iterate over.
    /// - `consts`: A set of constants to use to quantify atoms with arity > 0.
    ///
    /// # Returns
    /// A new HerbrandXBaseIter.
    fn new(spec: &'s Spec, consts: &'u IndexSet<Ident>) -> Self { Self { spec, consts, rules: spec.rules.iter(), atoms: None, iters: vec![] } }

    /// Replaces [`Iterator::next()`] such that an existing [`Atom`] is populated instead of allocating a new one.
    ///
    /// # Arguments
    /// - `atom`: Some [`Atom`] to set with the next value, or else [`None`].
    ///
    /// # Returns
    /// The given Atom for convenience.
    ///
    /// # Panics
    /// This function can panic if the user did some weird stuff, like giving us a different buffer in-between runs.
    #[inline]
    #[track_caller]
    pub fn next<'a>(&mut self, atom: &'a mut Option<Atom>) -> &'a mut Option<Atom> {
        /// Stand-in for a closure such that we can name the type.
        #[inline]
        fn tail_to_values<'s>(tail: &'s RuleAntecedents) -> ast_toolkit_punctuated::normal::Values<'s, Literal, Comma> { tail.antecedents.values() }


        // Get the next instantiation of the atom
        if !self.iters.is_empty() {
            // Ensure there is an atom allocated with enough space
            let args: &mut Punctuated<AtomArg, Comma> = if let Some(Atom { args: Some(args), .. }) = atom {
                &mut args.args
            } else {
                panic!("Got an Atom with non-instantiated arguments after creating iterators for it.");
            };

            // Populate new arguments for the atom
            args.clear();
            for (i, iter) in self.iters.iter_mut().enumerate() {
                // Attempt to get the next element
                if let Some(next) = iter.next() {
                    if i == 0 {
                        args.push_first(AtomArg::Atom(*next));
                    } else {
                        args.push(Comma { span: Span::new("<auto generated by HerbrandXBaseIter::next()>", ",") }, AtomArg::Atom(*next));
                    }
                }
                // Otherwise, got None
                break;
            }

            // OK done return
            return atom;
        }

        // Get the next atom in the rule if there is any
        if let Some(new_atom) = self.atoms.as_mut().map(|i| i.next()).flatten() {
            // Find if this atom has any arguments
            let n_args: usize = new_atom.args.as_ref().map(|a| a.args.len()).unwrap_or(0);

            // If there are none, then simply return this instantiation
            if n_args == 0 {
                // Remove any arguments in the buffer
                self.iters.clear();
                if let Some(Atom { ident, args: Some(args) }) = atom {
                    *ident = new_atom.ident;
                    args.args.clear();
                } else {
                    *atom = Some(Atom { ident: new_atom.ident, args: None });
                }
                return atom;
            }

            // Otherwise, build enough iterators
            self.iters.clear();
            for i in 0..n_args {
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
                self.iters.push(RepeatIter::new(self.consts, self.consts.len().pow((n_args - 1 - i) as u32), self.consts.len().pow(i as u32)));
            }

            // Allocate enough space in the buffer
            match atom {
                Some(Atom { args: Some(args), .. }) => {
                    args.args.reserve(self.iters.len());
                },
                Some(Atom { args, .. }) => {
                    *args = Some(AtomArgs {
                        // SAFETY: We can `unwrap()` because we early quit if `new_atom` has no elements. If it does have, then it _must_ have `Some` args.
                        paren_tokens: new_atom.args.as_ref().unwrap().paren_tokens,
                        args: Punctuated::with_capacity(self.iters.len()),
                    });
                },
                None => {
                    *atom = Some(Atom {
                        ident: new_atom.ident,
                        args:  Some(AtomArgs {
                            // SAFETY: We can `unwrap()` because we early quit if `new_atom` has no elements. If it does have, then it _must_ have `Some` args.
                            paren_tokens: new_atom.args.as_ref().unwrap().paren_tokens,
                            args: Punctuated::with_capacity(self.iters.len()),
                        }),
                    })
                },
            }

            // Try again to find the next iteration
            return self.next(atom);
        }

        // Fall back to the next rule
        let rule: &'s Rule = match self.rules.next() {
            Some(rule) => rule,
            None => {
                // Nothing more; return None
                *atom = None;
                return atom;
            },
        };
        self.atoms = Some(
            rule.consequences.values().chain(rule.tail.iter().flat_map(tail_to_values as TailToValues<'s>).map(Literal::atom as LiteralToAtom<'s>)),
        );
        self.next(atom)
    }

    /// Returns the total number of elements we will find by iterating.
    ///
    /// Note that this searches the internal spec for atoms, so might be relatively expensive (order of O(n)).
    ///
    /// # Returns
    /// The expected number of elements this iterator yields in total.
    pub fn size_hint(&self) -> usize {
        let mut len: usize = 0;
        for rule in &self.spec.rules {
            for cons in rule.consequences.values() {
                len += cons
                    .args
                    .as_ref()
                    .map(|a| {
                        (0..a.args.len()).map(|i| self.consts.len().pow(i as u32)).sum::<usize>()
                            * (0..a.args.len()).map(|i| self.consts.len().pow((a.args.len() - 1 - i) as u32)).sum::<usize>()
                    })
                    .unwrap_or(1);
            }
            for ante in rule.tail.iter().flat_map(|t| t.antecedents.values()) {
                len += ante
                    .atom()
                    .args
                    .as_ref()
                    .map(|a| {
                        (0..a.args.len()).map(|i| self.consts.len().pow(i as u32)).sum::<usize>()
                            * (0..a.args.len()).map(|i| self.consts.len().pow((a.args.len() - 1 - i) as u32)).sum::<usize>()
                    })
                    .unwrap_or(1);
            }
        }
        len
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
    /// The Spec of which this HerbrandUniverse is a part.
    spec:   &'s Spec,
    /// Defines the Herbrand 0-base of the given Spec. We always compute this, since we need it for the other two bases.
    consts: IndexSet<Ident>,
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
    pub fn new(spec: &'s Spec) -> Self { Self { spec, consts: Herbrand0BaseIter::new(spec).collect() } }

    /// Returns the inner [`Spec`].
    ///
    /// # Returns
    /// A reference to the inner [`Spec`].
    #[inline]
    pub fn spec(&self) -> &'s Spec { self.spec }

    /// Returns the inner computed 0-base.
    ///
    /// # Returns
    /// A reference to an [`IndexSet`] that contains all constants (stored as [`Ident`]s) in the internal [`Spec`].
    #[inline]
    pub fn zero_base(&self) -> &IndexSet<Ident> { &self.consts }
}
impl<'s> HerbrandUniverse<'s> {
    /// Finds the 0-base of the given [`Spec`].
    ///
    /// # Returns
    /// A `Herbrand0BaseIter` that computes the zero-base of the internal [`Spec`] on-demand.
    #[inline]
    pub fn find_0_base(&self) -> Herbrand0BaseIter<'s> { Herbrand0BaseIter::new(self.spec) }

    /// Finds the X-base of the internal [`Spec`].
    ///
    /// # Returns
    /// A `Herbrand0BaseIter` that computes the full Herbrand base of the internal [`Spec`] on-demand.
    #[inline]
    pub fn find_x_base<'u>(&'u self) -> HerbrandXBaseIter<'s, 'u> { HerbrandXBaseIter::new(self.spec, &self.consts) }
}
