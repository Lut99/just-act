//  INTERPRETER.rs
//    by Lut99
//
//  Created:
//    13 Mar 2024, 17:54:05
//  Last edited:
//    19 Mar 2024, 13:51:03
//  Auto updated?
//    Yes
//
//  Description:
//!   Evaluates a given $Datalog^\neg$ AST.
//

use std::cmp::Ordering;

use ast_toolkit_punctuated::Punctuated;
use indexmap::{IndexMap, IndexSet};
use itertools::Itertools as _;

use crate::ast::{Atom, AtomArg, AtomArgs, Comma, Ident, Literal, NegAtom, Parens, Rule, RuleAntecedents, Spec};


/***** TESTS *****/
#[cfg(all(test, feature = "derive"))]
mod tests {
    use ast_toolkit_punctuated::punct;
    use ast_toolkit_span::Span;

    use super::*;
    use crate::ast::{datalog, Arrow, AtomArgs, Dot, Ident, Parens};

    #[test]
    fn test_knowledge_base_iterator() {
        let kb: IndexSet<Atom<&str, &str>> = IndexSet::new();
        assert_eq!(KnowledgeBaseIterator::new(&kb, 0).next(), None);
        assert_eq!(KnowledgeBaseIterator::new(&kb, 1).next(), None);
        assert_eq!(KnowledgeBaseIterator::new(&kb, 2).next(), None);
        assert_eq!(KnowledgeBaseIterator::new(&kb, 3).next(), None);
        assert_eq!(KnowledgeBaseIterator::new(&kb, 4).next(), None);



        let source: Span<&str, &str> = Span::new("test_knowledge_base_iterator::example", "foo :- bar, baz, quz(qux).");
        let foo: Ident<&str, &str> = Ident { value: source.slice(..3) };
        let bar: Ident<&str, &str> = Ident { value: source.slice(7..10) };
        let kb: IndexSet<Atom<&str, &str>> = IndexSet::from([Atom { ident: foo, args: None }, Atom { ident: bar, args: None }]);

        let mut iter = KnowledgeBaseIterator::new(&kb, 0);
        assert_eq!(iter.next(), None);

        let mut iter = KnowledgeBaseIterator::new(&kb, 1);
        assert_eq!(iter.next(), Some([&foo].as_slice()));
        assert_eq!(iter.next(), Some([&bar].as_slice()));
        assert_eq!(iter.next(), None);

        let mut iter = KnowledgeBaseIterator::new(&kb, 2);
        assert_eq!(iter.next(), Some([&foo, &foo].as_slice()));
        assert_eq!(iter.next(), Some([&foo, &bar].as_slice()));
        assert_eq!(iter.next(), Some([&bar, &foo].as_slice()));
        assert_eq!(iter.next(), Some([&bar, &bar].as_slice()));
        assert_eq!(iter.next(), None);

        let mut iter = KnowledgeBaseIterator::new(&kb, 3);
        assert_eq!(iter.next(), Some([&foo, &foo, &foo].as_slice()));
        assert_eq!(iter.next(), Some([&foo, &foo, &bar].as_slice()));
        assert_eq!(iter.next(), Some([&foo, &bar, &foo].as_slice()));
        assert_eq!(iter.next(), Some([&foo, &bar, &bar].as_slice()));
        assert_eq!(iter.next(), Some([&bar, &foo, &foo].as_slice()));
        assert_eq!(iter.next(), Some([&bar, &foo, &bar].as_slice()));
        assert_eq!(iter.next(), Some([&bar, &bar, &foo].as_slice()));
        assert_eq!(iter.next(), Some([&bar, &bar, &bar].as_slice()));
        assert_eq!(iter.next(), None);



        let baz: Ident<&str, &str> = Ident { value: source.slice(12..15) };
        let kb: IndexSet<Atom<&str, &str>> =
            IndexSet::from([Atom { ident: foo, args: None }, Atom { ident: bar, args: None }, Atom { ident: baz, args: None }]);

        let mut iter = KnowledgeBaseIterator::new(&kb, 2);
        assert_eq!(iter.next(), Some([&foo, &foo].as_slice()));
        assert_eq!(iter.next(), Some([&foo, &bar].as_slice()));
        assert_eq!(iter.next(), Some([&foo, &baz].as_slice()));
        assert_eq!(iter.next(), Some([&bar, &foo].as_slice()));
        assert_eq!(iter.next(), Some([&bar, &bar].as_slice()));
        assert_eq!(iter.next(), Some([&bar, &baz].as_slice()));
        assert_eq!(iter.next(), Some([&baz, &foo].as_slice()));
        assert_eq!(iter.next(), Some([&baz, &bar].as_slice()));
        assert_eq!(iter.next(), Some([&baz, &baz].as_slice()));
        assert_eq!(iter.next(), None);



        let mut args: Punctuated<AtomArg<&str, &str>, Comma<&str, &str>> = Punctuated::with_capacity(1);
        args.push_first(AtomArg::Atom(Ident { value: source.slice(21..24) }));
        let kb: IndexSet<Atom<&str, &str>> =
            IndexSet::from([Atom { ident: foo, args: None }, Atom { ident: bar, args: None }, Atom { ident: baz, args: None }, Atom {
                ident: Ident { value: source.slice(17..20) },
                args:  Some(AtomArgs { paren_tokens: Parens { open: source.slice(20..21), close: source.slice(24..25) }, args }),
            }]);

        let mut iter = KnowledgeBaseIterator::new(&kb, 2);
        assert_eq!(iter.next(), Some([&foo, &foo].as_slice()));
        assert_eq!(iter.next(), Some([&foo, &bar].as_slice()));
        assert_eq!(iter.next(), Some([&foo, &baz].as_slice()));
        assert_eq!(iter.next(), Some([&bar, &foo].as_slice()));
        assert_eq!(iter.next(), Some([&bar, &bar].as_slice()));
        assert_eq!(iter.next(), Some([&bar, &baz].as_slice()));
        assert_eq!(iter.next(), Some([&baz, &foo].as_slice()));
        assert_eq!(iter.next(), Some([&baz, &bar].as_slice()));
        assert_eq!(iter.next(), Some([&baz, &baz].as_slice()));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_rule_concretizer() {
        /// Builds a rule with the given placeholder as argument.
        ///
        /// # Arguments
        /// - `source`: The source to take slices from.
        /// - `lhs`: The thing to put as `X` in `quz(X) :- foo(Y).`.
        /// - `rhs`: The thing to put as `Y` in `quz(X) :- foo(Y).`.
        ///
        /// # Returns
        /// A new [`Rule`] that is the AST of the rule above.
        fn make_rule_one_var<'f, 's>(
            source: &Span<&'f str, &'s str>,
            lhs: AtomArg<&'f str, &'s str>,
            rhs: AtomArg<&'f str, &'s str>,
        ) -> Rule<&'f str, &'s str> {
            Rule {
                consequences: punct![ v => Atom {
                    ident: Ident { value: source.slice(20..23) },
                    args: Some(AtomArgs {
                        paren_tokens: Parens { open: source.slice(23..24), close: source.slice(25..26) },
                        args: punct![v => lhs]
                    })
                }],
                tail: Some(RuleAntecedents {
                    arrow_token: Arrow { span: source.slice(27..29) },
                    antecedents: punct![ v => Literal::Atom(Atom {
                        ident: Ident { value: source.slice(30..33) },
                        args: Some(AtomArgs {
                            paren_tokens: Parens { open: source.slice(33..34), close: source.slice(35..36) },
                            args: punct![v => rhs]
                        })
                    })],
                }),
                dot: Dot { span: source.slice(36..37) },
            }
        }

        /// Builds a rule with the given placeholder as argument.
        ///
        /// # Arguments
        /// - `source`: The source to take slices from.
        /// - `lhs`: The thing to put as `X` in `qux(X, Y) :- foo(X), quz(Y).`.
        /// - `rhs`: The thing to put as `Y` in `qux(X, Y) :- foo(X), quz(Y).`.
        ///
        /// # Returns
        /// A new [`Rule`] that is the AST of the rule above.
        fn make_rule_two_var<'f, 's>(
            source: &Span<&'f str, &'s str>,
            lhs: AtomArg<&'f str, &'s str>,
            rhs: AtomArg<&'f str, &'s str>,
        ) -> Rule<&'f str, &'s str> {
            Rule {
                consequences: punct![ v => Atom {
                    ident: Ident { value: source.slice(48..51) },
                    args: Some(AtomArgs {
                        paren_tokens: Parens { open: source.slice(51..52), close: source.slice(56..57) },
                        args: punct![v => lhs.clone(), p => Comma { span: source.slice(53..54) }, v => rhs.clone()]
                    })
                }],
                tail: Some(RuleAntecedents {
                    arrow_token: Arrow { span: source.slice(58..60) },
                    antecedents: punct![
                        v => Literal::Atom(Atom {
                            ident: Ident { value: source.slice(61..64) },
                            args: Some(AtomArgs {
                                paren_tokens: Parens { open: source.slice(64..65), close: source.slice(66..67) },
                                args: punct![v => lhs]
                            })
                        }),
                        p => Comma { span: source.slice(67..68) },
                        v => Literal::Atom(Atom {
                            ident: Ident { value: source.slice(69..70) },
                            args: Some(AtomArgs {
                                paren_tokens: Parens { open: source.slice(72..73), close: source.slice(74..75) },
                                args: punct![v => rhs]
                            })
                        })
                    ],
                }),
                dot: Dot { span: source.slice(75..76) },
            }
        }



        // Setup the source text and an AST "parsed" from it
        let source: Span<&str, &str> =
            Span::new("test_rule_concretizer::example", "bar. baz. foo(bar). foo(baz). quz(X) :- foo(X). qux(X, Y) :- foo(X), quz(Y).");

        // The rule to concretize (`quz(X) :- foo(X)`)
        let rule: Rule<&str, &str> =
            make_rule_one_var(&source, AtomArg::Var(Ident { value: source.slice(34..35) }), AtomArg::Var(Ident { value: source.slice(44..45) }));



        // Empty knowledge base test
        let kb: IndexSet<Atom<&str, &str>> = IndexSet::new();
        assert_eq!(RuleConcretizer::new(&rule, &kb).next(), None);

        // Non-empty knowledge base test but no "elementary" atoms (i.e., no arguments)
        let kb: IndexSet<Atom<&str, &str>> = IndexSet::from([
            Atom {
                ident: Ident { value: source.slice(10..13) },
                args:  Some(AtomArgs {
                    paren_tokens: Parens { open: source.slice(13..14), close: source.slice(17..18) },
                    args: punct![v => AtomArg::Atom(Ident { value: source.slice(14..17) })],
                }),
            },
            Atom {
                ident: Ident { value: source.slice(20..23) },
                args:  Some(AtomArgs {
                    paren_tokens: Parens { open: source.slice(23..24), close: source.slice(27..28) },
                    args: punct![v => AtomArg::Atom(Ident { value: source.slice(24..27) })],
                }),
            },
        ]);
        assert_eq!(RuleConcretizer::new(&rule, &kb).next(), None);

        // Non-empty knowledge base test
        let bar: Ident<&str, &str> = Ident { value: source.slice(0..3) };
        let baz: Ident<&str, &str> = Ident { value: source.slice(5..8) };
        let kb: IndexSet<Atom<&str, &str>> = IndexSet::from([
            Atom { ident: bar, args: None },
            Atom { ident: baz, args: None },
            Atom {
                ident: Ident { value: source.slice(10..13) },
                args:  Some(AtomArgs {
                    paren_tokens: Parens { open: source.slice(13..14), close: source.slice(17..18) },
                    args: punct![v => AtomArg::Atom(Ident { value: source.slice(14..17) })],
                }),
            },
            Atom {
                ident: Ident { value: source.slice(20..23) },
                args:  Some(AtomArgs {
                    paren_tokens: Parens { open: source.slice(23..24), close: source.slice(27..28) },
                    args: punct![v => AtomArg::Atom(Ident { value: source.slice(24..27) })],
                }),
            },
        ]);
        let mut iter = RuleConcretizer::new(&rule, &kb);
        assert_eq!(iter.next(), Some(make_rule_one_var(&source, AtomArg::Atom(bar), AtomArg::Atom(bar))));
        assert_eq!(iter.next(), Some(make_rule_one_var(&source, AtomArg::Atom(baz), AtomArg::Atom(baz))));
        assert_eq!(iter.next(), None);



        // Now with two vars
        let rule: Rule<&str, &str> =
            make_rule_two_var(&source, AtomArg::Var(Ident { value: source.slice(52..53) }), AtomArg::Var(Ident { value: source.slice(55..56) }));
        let mut iter = RuleConcretizer::new(&rule, &kb);
        assert_eq!(iter.next(), Some(make_rule_two_var(&source, AtomArg::Atom(bar), AtomArg::Atom(bar))));
        assert_eq!(iter.next(), Some(make_rule_two_var(&source, AtomArg::Atom(bar), AtomArg::Atom(baz))));
        assert_eq!(iter.next(), Some(make_rule_two_var(&source, AtomArg::Atom(baz), AtomArg::Atom(bar))));
        assert_eq!(iter.next(), Some(make_rule_two_var(&source, AtomArg::Atom(baz), AtomArg::Atom(baz))));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_herbrand_base_iterator() {
        fn make_atom(name: &'static str, args: Option<Vec<&'static str>>) -> Atom<&'static str, &'static str> {
            // Convert the arguments
            let puncs: Option<Punctuated<AtomArg<&'static str, &'static str>, Comma<&'static str, &'static str>>> = args.map(|args| {
                let mut puncs: Punctuated<_, _> = Punctuated::new();
                for (i, a) in args.into_iter().enumerate() {
                    if i == 0 {
                        puncs.push_first(AtomArg::Atom(Ident { value: Span::new("make_atom::arg::ident", a) }));
                    } else {
                        puncs.push(
                            Comma { span: Span::new("make_atom::arg::comma", ",") },
                            AtomArg::Atom(Ident { value: Span::new("make_atom::arg::ident", a) }),
                        );
                    }
                }
                puncs
            });

            // Leggo
            Atom {
                ident: Ident { value: Span::new("make_atom::ident", name) },
                args:  puncs.map(|puncs| AtomArgs {
                    paren_tokens: Parens { open: Span::new("make_atom::parens::open", "("), close: Span::new("make_atom::parens::close", ")") },
                    args: puncs,
                }),
            }
        }



        // Check some empty programs
        let empty: Spec<&str, &str> = datalog! { #![crate] };
        assert_eq!(HerbrandBaseIterator::new(&empty).next(), None);

        // Check constants
        let consts: Spec<&str, &str> = datalog! {
            #![crate]
            foo. bar. baz.
        };
        let mut iter = HerbrandBaseIterator::new(&consts);
        assert_eq!(iter.next(), Some(make_atom("bar", None)));
        assert_eq!(iter.next(), Some(make_atom("baz", None)));
        assert_eq!(iter.next(), Some(make_atom("foo", None)));
        assert_eq!(iter.next(), None);

        // Check functions
        let funcs: Spec<&str, &str> = datalog! {
            #![crate]
            foo(bar). bar(baz). baz(quz).
        };
        let mut iter = HerbrandBaseIterator::new(&funcs);
        assert_eq!(iter.next(), Some(make_atom("bar", None)));
        assert_eq!(iter.next(), Some(make_atom("baz", None)));
        assert_eq!(iter.next(), Some(make_atom("quz", None)));
        assert_eq!(iter.next(), Some(make_atom("bar", Some(vec!["bar"]))));
        assert_eq!(iter.next(), Some(make_atom("bar", Some(vec!["baz"]))));
        assert_eq!(iter.next(), Some(make_atom("bar", Some(vec!["quz"]))));
        assert_eq!(iter.next(), Some(make_atom("baz", Some(vec!["bar"]))));
        assert_eq!(iter.next(), Some(make_atom("baz", Some(vec!["baz"]))));
        assert_eq!(iter.next(), Some(make_atom("baz", Some(vec!["quz"]))));
        assert_eq!(iter.next(), Some(make_atom("foo", Some(vec!["bar"]))));
        assert_eq!(iter.next(), Some(make_atom("foo", Some(vec!["baz"]))));
        assert_eq!(iter.next(), Some(make_atom("foo", Some(vec!["quz"]))));
        assert_eq!(iter.next(), None);

        // Multi-argument functions
        let multi_funcs: Spec<&str, &str> = datalog! {
            #![crate]
            foo. bar. baz.
            quz(foo, bar, baz).
        };
        let mut iter = HerbrandBaseIterator::new(&multi_funcs);
        println!(
            "{:?}",
            HerbrandBaseIterator::new(&multi_funcs)
                .map(|a| format!(
                    "{}({})",
                    a.ident.value.value(),
                    a.args
                        .iter()
                        .map(|a| a.args.values())
                        .flatten()
                        .map(|v| if let AtomArg::Atom(a) = v { a.value.value().to_string() } else { panic!() })
                        .collect::<Vec<String>>()
                        .join(", ")
                ))
                .collect::<Vec<String>>()
                .join(". ")
        );
        assert_eq!(iter.next(), Some(make_atom("bar", None)));
        assert_eq!(iter.next(), Some(make_atom("baz", None)));
        assert_eq!(iter.next(), Some(make_atom("foo", None)));
        assert_eq!(iter.next(), Some(make_atom("quz", Some(vec!["bar", "bar", "bar"]))));
        assert_eq!(iter.next(), Some(make_atom("quz", Some(vec!["bar", "bar", "baz"]))));
        assert_eq!(iter.next(), Some(make_atom("quz", Some(vec!["bar", "bar", "foo"]))));
        assert_eq!(iter.next(), Some(make_atom("quz", Some(vec!["bar", "baz", "bar"]))));
        assert_eq!(iter.next(), Some(make_atom("quz", Some(vec!["bar", "baz", "baz"]))));
        assert_eq!(iter.next(), Some(make_atom("quz", Some(vec!["bar", "baz", "foo"]))));
        assert_eq!(iter.next(), Some(make_atom("quz", Some(vec!["bar", "foo", "bar"]))));
        assert_eq!(iter.next(), Some(make_atom("quz", Some(vec!["bar", "foo", "baz"]))));
        assert_eq!(iter.next(), Some(make_atom("quz", Some(vec!["bar", "foo", "foo"]))));
        assert_eq!(iter.next(), Some(make_atom("quz", Some(vec!["baz", "bar", "bar"]))));
        assert_eq!(iter.next(), Some(make_atom("quz", Some(vec!["baz", "bar", "baz"]))));
        assert_eq!(iter.next(), Some(make_atom("quz", Some(vec!["baz", "bar", "foo"]))));
        assert_eq!(iter.next(), Some(make_atom("quz", Some(vec!["baz", "baz", "bar"]))));
        assert_eq!(iter.next(), Some(make_atom("quz", Some(vec!["baz", "baz", "baz"]))));
        assert_eq!(iter.next(), Some(make_atom("quz", Some(vec!["baz", "baz", "foo"]))));
        assert_eq!(iter.next(), Some(make_atom("quz", Some(vec!["baz", "foo", "bar"]))));
        assert_eq!(iter.next(), Some(make_atom("quz", Some(vec!["baz", "foo", "baz"]))));
        assert_eq!(iter.next(), Some(make_atom("quz", Some(vec!["baz", "foo", "foo"]))));
        assert_eq!(iter.next(), Some(make_atom("quz", Some(vec!["foo", "bar", "bar"]))));
        assert_eq!(iter.next(), Some(make_atom("quz", Some(vec!["foo", "bar", "baz"]))));
        assert_eq!(iter.next(), Some(make_atom("quz", Some(vec!["foo", "bar", "foo"]))));
        assert_eq!(iter.next(), Some(make_atom("quz", Some(vec!["foo", "baz", "bar"]))));
        assert_eq!(iter.next(), Some(make_atom("quz", Some(vec!["foo", "baz", "baz"]))));
        assert_eq!(iter.next(), Some(make_atom("quz", Some(vec!["foo", "baz", "foo"]))));
        assert_eq!(iter.next(), Some(make_atom("quz", Some(vec!["foo", "foo", "bar"]))));
        assert_eq!(iter.next(), Some(make_atom("quz", Some(vec!["foo", "foo", "baz"]))));
        assert_eq!(iter.next(), Some(make_atom("quz", Some(vec!["foo", "foo", "foo"]))));
        assert_eq!(iter.next(), None);

        // Alright now some complex rules
        let rules: Spec<&str, &str> = datalog! {
            #![crate]
            foo. bar(foo).
            baz(X) :- bar(X).
        };
        let mut iter = HerbrandBaseIterator::new(&rules);
        assert_eq!(iter.next(), Some(make_atom("foo", None)));
        assert_eq!(iter.next(), Some(make_atom("bar", Some(vec!["foo"]))));
        assert_eq!(iter.next(), Some(make_atom("baz", Some(vec!["foo"]))));
        assert_eq!(iter.next(), None);
    }
}





/***** HELPER FUNCTIONS *****/
/// Traverses a rule to find all variables.
///
/// # Arguments
/// - `rule`: The [`Rule`] to analyze.
///
/// # Returns
/// A list of the names of variables found.
fn find_vars_in_rule<'r, 'f, 's>(rule: &'r Rule<&'f str, &'s str>) -> IndexSet<&'r str> {
    // Iterate over the consequences
    let mut res: IndexSet<&'r str> = IndexSet::new();
    for con in rule.consequences.values() {
        for arg in con.args.iter().map(|c| c.args.values()).flatten() {
            match arg {
                AtomArg::Atom(_) => continue,
                AtomArg::Var(v) => {
                    res.insert(v.value.value());
                },
            }
        }
    }
    // Add the antecedents
    for ant in rule.tail.iter().map(|h| h.antecedents.values()).flatten() {
        match ant {
            Literal::Atom(atom) | Literal::NegAtom(NegAtom { atom, not_token: _ }) => {
                for arg in atom.args.iter().map(|c| c.args.values()).flatten() {
                    match arg {
                        AtomArg::Atom(_) => continue,
                        AtomArg::Var(v) => {
                            res.insert(v.value.value());
                        },
                    }
                }
            },
        }
    }
    res
}

/// Swaps all variables in a rule with a given list of concrete instances.
///
/// # Arguments
/// - `rule`: The [`Rule`] to clone.
/// - `vars`: The instantiation of all the variables.
///
/// # Returns
/// A new [`Rule`] that doesn't have any variables.
fn replace_vars_in_rule<'i, 'r, 'f, 's>(
    rule: &'r Rule<&'f str, &'s str>,
    vars: IndexMap<&'r str, &'i Ident<&'f str, &'s str>>,
) -> Rule<&'f str, &'s str> {
    // Clone the consequents
    let mut consequences: Punctuated<Atom<&'f str, &'s str>, Comma<&'f str, &'s str>> = rule.consequences.clone();
    for cons in consequences.values_mut() {
        for arg in cons.args.iter_mut().map(|c| c.args.values_mut()).flatten() {
            let new_arg: Option<AtomArg<&'f str, &'s str>> = match arg {
                AtomArg::Atom(_) => continue,
                AtomArg::Var(v) => {
                    vars.get(&v.value.value()).cloned().cloned().map(|ident| {
                        // Replace the full assignment
                        AtomArg::Atom(ident)
                    })
                },
            };
            if let Some(new_arg) = new_arg {
                *arg = new_arg;
            }
        }
    }

    // Clone the antecedents
    let mut tail: Option<RuleAntecedents<&'f str, &'s str>> = rule.tail.clone();
    for ante in tail.iter_mut().map(|t| t.antecedents.values_mut()).flatten() {
        for arg in ante.atom_mut().args.iter_mut().map(|a| a.args.values_mut()).flatten() {
            let new_arg: Option<AtomArg<&'f str, &'s str>> = match arg {
                AtomArg::Atom(_) => continue,
                AtomArg::Var(v) => {
                    vars.get(&v.value.value()).cloned().cloned().map(|ident| {
                        // Replace the full assignment
                        AtomArg::Atom(ident)
                    })
                },
            };
            if let Some(new_arg) = new_arg {
                *arg = new_arg;
            }
        }
    }

    // K done
    Rule { consequences, tail, dot: rule.dot.clone() }
}



/// Finds the largest possible unfounded set for the given specification.
///
/// An _unfounded set_ is a set of literals of which, for all rules that produce it, at least one of the following holds:
/// 1. One of the rule's antecedents are false in the knowledge base; or
/// 2. One of the rule's _positive_ antecedents occurs in the unfounded set.
///
/// An unfounded set therefore represents a set of facts that we _could_ try to assume we know something about, but which would yield us little additional deductions.
///
/// Luckily for us, unfounded sets are trivially composable by the union operator, so we can simply find sets of size 1 and join 'em all up.
///
/// # Arguments
/// - `spec`: A [`Spec`] that denotes the program to find the largest unfounded set of.
/// - `kb`: A knowledge base that we will use as partial interpretation to investigate rule 1 with.
///
/// # Returns
/// An [`IndexSet`] with the literals that are part of the largest unfounded set.
fn find_largest_unfounded_set<'a, 'f, 's>(
    spec: &'a Spec<&'f str, &'s str>,
    kb: &'_ IndexMap<&'a Atom<&'f str, &'s str>, bool>,
) -> IndexSet<&'a Atom<&'f str, &'s str>> {
    let mut unfounded_set: IndexSet<&'a Literal<&'f str, &'s str>> = IndexSet::new();
    // Iterate over all concrete rule instances
    for rule in &spec.rules {}

    todo!()
}





/***** AUXILLARY *****/
/// Given a [`Spec`], generates the full Herbrand Universe as we found it in it.
///
/// This means it essentially produces a list of all combinations of atoms in the spec that are suggested by it:
/// - All constants (arity-0 functions) (e.g., `foo`, `bar`, ...)
/// - All functions of found arity with all constants (e.g., `bar(foo)`, `bar(baz)`, `quz(foo, bar)`, ...)
#[derive(Debug)]
pub struct HerbrandBaseIterator<'f, 's> {
    /// The list of atoms we found on construction and now just iterate over.
    iter: std::vec::IntoIter<Atom<&'f str, &'s str>>,
}
impl<'f, 's> HerbrandBaseIterator<'f, 's> {
    /// Constructor for the HerbrandBaseIterator that does all the work.
    ///
    /// # Arguments
    /// - `spec`: The [`Spec`] to find the Herbrand Base of.
    ///
    /// # Returns
    /// A new HerbrandBaseIterator that will generate the base.
    pub fn new<'p>(spec: &'p Spec<&'f str, &'s str>) -> Self {
        // Organise all atoms in the spec into constants and functions
        let mut constants: Vec<&'p Ident<&'f str, &'s str>> = Vec::new();
        let mut functions: Vec<(&'p Ident<&'f str, &'s str>, &'p Parens<&'f str, &'s str>, usize)> = Vec::new();
        for rule in &spec.rules {
            // Search the rule's consequences
            for cons in rule.consequences.values() {
                // If there are arguments, consider those too
                if let Some(args) = &cons.args {
                    for arg in args.args.values() {
                        match arg {
                            AtomArg::Atom(ident) => constants.push(ident),
                            AtomArg::Var(_) => continue,
                        }
                    }
                    functions.push((&cons.ident, &args.paren_tokens, args.args.len()));
                } else {
                    constants.push(&cons.ident);
                }
            }

            // Next, search its antecedents
            if let Some(tail) = &rule.tail {
                for ante in tail.antecedents.values() {
                    // If there are arguments, consider those too
                    if let Some(args) = &ante.atom().args {
                        for arg in args.args.values() {
                            match arg {
                                AtomArg::Atom(ident) => constants.push(ident),
                                AtomArg::Var(_) => continue,
                            }
                        }
                        functions.push((&ante.atom().ident, &args.paren_tokens, args.args.len()));
                    } else {
                        constants.push(&ante.atom().ident);
                    }
                }
            }
        }

        // De-duplicate both lists
        constants.sort_by(|lhs, rhs| -> Ordering { lhs.value.value().cmp(rhs.value.value()) });
        constants.dedup_by(|lhs, rhs| -> bool { lhs.value.value() == rhs.value.value() });
        functions.sort_by(|(lhs, _, _), (rhs, _, _)| -> Ordering { lhs.value.value().cmp(rhs.value.value()) });
        functions.dedup_by(|(lhs, _, _), (rhs, _, _)| -> bool { lhs.value.value() == rhs.value.value() });

        // Now re-generate the functions into all possible combinations of them + constants
        // NOTE: The re-building of the atom below is quite cheap, because constants don't have argument vectors to clone
        let mut herbrand_base: Vec<Atom<&'f str, &'s str>> = constants.iter().map(|i| Atom { ident: **i, args: None }).collect();
        herbrand_base.reserve(functions.len() * constants.len());
        for (func, parens, arity) in functions {
            // Iterate over the function's arity
            for args in (0..arity).map(|_| constants.iter()).multi_cartesian_product() {
                // Turn it into a punctuated list
                let mut puncs: Punctuated<AtomArg<&'f str, &'s str>, Comma<&'f str, &'s str>> = Punctuated::new();
                for (i, ident) in args.into_iter().enumerate() {
                    if i == 0 {
                        puncs.push_first(AtomArg::Atom(**ident));
                    } else {
                        // Compute a span that covers the value
                        puncs.push(Comma { span: ident.value }, AtomArg::Atom(**ident));
                    }
                }

                // Build a new atom with it
                herbrand_base.push(Atom { ident: *func, args: Some(AtomArgs { paren_tokens: *parens, args: puncs }) })
            }
        }

        // Coolio, store that and ready for iteration!
        Self { iter: herbrand_base.into_iter() }
    }
}
impl<'f, 's> Iterator for HerbrandBaseIterator<'f, 's> {
    type Item = Atom<&'f str, &'s str>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> { self.iter.next() }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) { self.iter.size_hint() }
}

/// Given a HashSet, generates only atoms that are flat (i.e., identifiers).
///
/// Basically, changes
/// ```datalog
/// foo. bar. baz(foo).
/// ```
/// into
/// ```datalog
/// foo. bar.
/// ```
#[derive(Debug)]
pub struct FlatAtomIterator<'k, 'f, 's>(indexmap::set::Iter<'k, Atom<&'f str, &'s str>>);
impl<'k, 'f, 's> FlatAtomIterator<'k, 'f, 's> {
    /// Constructor for the FlatAtomIterator.
    ///
    /// # Arguments
    /// - `atoms`: The list of atoms to iterate over.
    ///
    /// # Returns
    /// A new FlatAtomIterator that will only return the atoms without arguments (as [`Ident`]s).
    #[inline]
    pub fn new(atoms: &'k IndexSet<Atom<&'f str, &'s str>>) -> Self { Self(atoms.iter()) }
}
impl<'k, 'f, 's> Iterator for FlatAtomIterator<'k, 'f, 's> {
    type Item = &'k Ident<&'f str, &'s str>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(atom) = self.0.next() {
            if atom.args.is_none() {
                return Some(&atom.ident);
            }
        }
        None
    }
}

/// Given a HashSet, generates all possible tuples of arbitrary length that can be taken from it.
///
/// Basically, changes
/// ```datalog
/// foo. bar. baz(foo).
/// ```
/// into
/// ```datalog
/// foo.
/// bar.
/// ```
/// or
/// ```datalog
/// foo. foo.
/// foo. bar.
/// bar. foo.
/// bar. bar
/// ```
/// etc.
#[derive(Debug)]
pub struct KnowledgeBaseIterator<'k, 'f, 's> {
    /// The knowledge base to iterate over.
    kb:    &'k IndexSet<Atom<&'f str, &'s str>>,
    /// The current set of iterators that we're iterating.
    iters: Vec<FlatAtomIterator<'k, 'f, 's>>,
    /// A vector that we allocated once and then return for efficiency.
    res:   Vec<&'k Ident<&'f str, &'s str>>,
}
impl<'k, 'f, 's> KnowledgeBaseIterator<'k, 'f, 's> {
    /// Constructor for the KnowledgeBaseIterator.
    ///
    /// # Arguments
    /// - `knowledge_base`: The knowledge base (as a [`HashSet<String>`]) to iterate over.
    /// - `n_vars`: The number of variables to iterate (i.e., the tuple length).
    ///
    /// # Returns
    /// A new KnowledgeBaseIterator ready to go.
    #[inline]
    pub fn new(knowledge_base: &'k IndexSet<Atom<&'f str, &'s str>>, n_vars: usize) -> Self {
        // Build the iterators
        let mut iters: Vec<FlatAtomIterator<'k, 'f, 's>> = Vec::with_capacity(n_vars);
        for _ in 0..n_vars {
            iters.push(FlatAtomIterator::new(knowledge_base));
        }

        // Build ourselves
        Self { kb: knowledge_base, iters, res: Vec::with_capacity(n_vars) }
    }

    /// Gets the next sample.
    ///
    /// Note: doesn't implement [`Iterator`] because it doesn't GAT :/
    ///
    /// # Returns
    /// A new [`&[&'k String]`] that represents the current assignment of variables.
    #[inline]
    pub fn next<'i>(&'i mut self) -> Option<&'i [&'k Ident<&'f str, &'s str>]> {
        // We're out-iter'ed if there's no iterators to speak of
        if self.iters.is_empty() {
            return None;
        }

        // Initialize the array if it hasn't been yet
        if self.res.is_empty() {
            // Ensure it's worth it to initialize
            if self.kb.is_empty() {
                self.iters.clear();
                return None;
            }

            // Alright collect the initial step
            for iter in &mut self.iters {
                match iter.next() {
                    Some(first) => self.res.push(first),
                    // NOTE: This can happen if the knowledge base only contains atoms with arguments.
                    None => {
                        self.iters.clear();
                        return None;
                    },
                }
            }

            // Cool, return this combination
            return Some(&self.res);
        }

        // Else, continue with our plight by trying to update the res back-to-front
        let iters = self.iters.iter_mut();
        for (i, iter) in iters.enumerate().rev() {
            match iter.next() {
                Some(val) => {
                    self.res[i] = val;
                    return Some(&self.res);
                },
                None => {
                    // Write this iterator afresh; then try again with the next one
                    *iter = FlatAtomIterator::new(self.kb);
                    // SAFETY: We just reset the iterator and we know it's non-empty because we iterated at least once before (the init).
                    self.res[i] = iter.next().unwrap();
                    continue;
                },
            }
        }

        // Done! Nothing left to iterate
        self.iters.clear();
        None
    }
}

/// Given a rule and a knowledge base, generates new rules that are concrete instances over its variables.
///
/// Basically, given
/// ```datalog
/// foo. bar.
/// ```
/// changes
/// ```datalog
/// baz(X) :- X.
/// ```
/// into
/// ```datalog
/// baz(foo) :- foo.
/// baz(bar) :- bar.
/// ```
#[derive(Debug)]
pub struct RuleConcretizer<'r, 'k, 'f, 's> {
    /// The rule that needs to be concretized.
    rule: &'r Rule<&'f str, &'s str>,
    /// The iterator that does the actual quantification.
    iter: KnowledgeBaseIterator<'k, 'f, 's>,
    /// The variables that we will populate.
    vars: IndexSet<&'r str>,
}
impl<'r, 'k, 'f, 's> RuleConcretizer<'r, 'k, 'f, 's> {
    /// Constructor that creates a new RuleConretizer.
    ///
    /// # Arguments
    /// - `rule`: The [`Rule`] to concretize.
    /// - `knowledge_base`: The knowledge base that we quantify over.
    ///
    /// # Returns
    /// A new RuleConcretizer that implements [`Iterator`].
    pub fn new(rule: &'r Rule<&'f str, &'s str>, knowledge_base: &'k IndexSet<Atom<&'f str, &'s str>>) -> Self {
        // Build self by scanning which variables exist
        let vars: IndexSet<&'r str> = find_vars_in_rule(rule);
        Self { rule, iter: KnowledgeBaseIterator::new(knowledge_base, vars.len()), vars }
    }
}
impl<'r, 'k, 'f, 's> Iterator for RuleConcretizer<'r, 'k, 'f, 's> {
    type Item = Rule<&'f str, &'s str>;

    fn next(&mut self) -> Option<Self::Item> {
        // Get the next variable mapping
        let vars: &[&'k Ident<&'f str, &'s str>] = self.iter.next()?;
        #[cfg(debug_assertions)]
        assert_eq!(vars.len(), self.vars.len());

        // Assign the new map to the actual variables
        // NOTE: The IndexMap guarantees a constant ordering, making the search completely covering
        let assignment: IndexMap<&'r str, &Ident<&'f str, &'s str>> = self.vars.iter().map(|v| *v).zip(vars.into_iter().map(|v| *v)).collect();

        // // Apply it to the variables in the rule
        Some(replace_vars_in_rule(self.rule, assignment))
    }
}





/***** LIBRARY *****/
/// Evaluates a given $Datalog^\neg$ AST.
///
/// Contains a knowledge base internally. That means that different interpreter instances may give different answers.
#[derive(Clone, Debug)]
pub struct Interpreter<'f, 's> {
    /// The set of facts that we know exist.
    pub knowledge_base: IndexSet<Atom<&'f str, &'s str>>,
}
impl<'f, 's> Default for Interpreter<'f, 's> {
    #[inline]
    fn default() -> Self { Self::new() }
}
impl<'f, 's> Interpreter<'f, 's> {
    /// Constructor for the Interpreter that initializes it with an empty knowledge base.
    ///
    /// # Returns
    /// A new Interpreter instance with nothing derived yet.
    #[inline]
    pub fn new() -> Self { Self { knowledge_base: IndexSet::new() } }

    /// Performs "one-time" evaluation on the given specification.
    ///
    /// This is equivalent to creating a new interpreter and interpreting with that.
    ///
    /// # Arguments
    /// - `spec`: The $Datalog^\neg$ [`Spec`]ification to evaluate.
    ///
    /// # Returns
    /// A derived set of facts, as a [`HashSet<String>`].
    ///
    /// # Example
    /// ```rust
    /// use justact_ast::{datalog, Spec};
    /// use justact_datalog::Interpeter;
    ///
    /// let spec: Spec = datalog!(foo.);
    ///
    /// // The verbose way
    /// let mut int = Interpreter::new();
    /// int.evaluate(&spec);
    ///
    /// // The short way
    /// let short = Interpeter::evaluate_once(&spec);
    /// assert_eq!(int.knowledge_base, short);
    /// ```
    #[inline]
    pub fn evaluate_once(spec: &Spec<&'f str, &'s str>) -> IndexSet<Atom<&'f str, &'s str>> {
        let mut int: Self = Self::new();
        int.evaluate(spec);
        int.knowledge_base
    }

    /// Preforms evaluation on the given specification.
    ///
    /// This updates the internal `knowledge_base`. You can manually inspect this.
    ///
    /// # Algorithm
    /// The interpreter relies on the _well-founded semantics_ to do derivation in a way that deals more intuitively with negate antecedents.
    ///
    /// Concretely, the well-founded semantics works
    ///
    /// # Arguments
    /// - `spec`: The $Datalog^\neg$ [`Spec`]ification to evaluate.
    ///
    /// # Example
    /// ```rust
    /// use justact_ast::{datalog, Spec};
    /// use justact_datalog::Interpeter;
    ///
    /// let mut int = Interpreter::new();
    /// int.evaluate(&datalog!(foo.));
    /// assert!(int.holds("foo"));
    /// ```
    pub fn evaluate(&mut self, spec: &Spec<&'f str, &'s str>) {
        //

        // Go thru the rules
        for rule in &spec.rules {
            // Consider all concrete instances based on variables
            let mut new_instances: IndexSet<Atom<&'f str, &'s str>> = IndexSet::new();
            for concrete_rule in RuleConcretizer::new(rule, &self.knowledge_base) {}
        }
    }
}
