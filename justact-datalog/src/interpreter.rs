//  INTERPRETER.rs
//    by Lut99
//
//  Created:
//    13 Mar 2024, 17:54:05
//  Last edited:
//    20 Mar 2024, 16:17:57
//  Auto updated?
//    Yes
//
//  Description:
//!   Evaluates a given $Datalog^\neg$ AST.
//

use std::borrow::Cow;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};

use ast_toolkit_punctuated::Punctuated;
use indexmap::IndexSet;
use itertools::Itertools as _;

use crate::ast::{Atom, AtomArg, AtomArgs, Comma, Ident, Literal, NegAtom, Not, Parens, Rule, Spec};
// use crate::refhash::RefHashMap;


/***** TESTS *****/
#[cfg(all(test, feature = "derive"))]
mod tests {
    use ast_toolkit_span::Span;

    use super::*;
    use crate::ast::{datalog, AtomArgs, Ident, Parens};


    fn make_atom(name: &'static str, args: Option<Vec<&'static str>>) -> Atom {
        // Convert the arguments
        let puncs: Option<Punctuated<AtomArg, Comma>> = args.map(|args| {
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


    #[test]
    fn test_find_herbrand_base() {
        // Check some empty programs
        let empty: Spec = datalog! { #![crate] };
        assert_eq!(find_herbrand_base(&empty), vec![]);

        // Check constants
        let consts: Spec = datalog! {
            #![crate]
            foo. bar. baz.
        };
        assert_eq!(find_herbrand_base(&consts), vec![make_atom("bar", None), make_atom("baz", None), make_atom("foo", None)]);

        // Check functions
        let funcs: Spec = datalog! {
            #![crate]
            foo(bar). bar(baz). baz(quz).
        };
        assert_eq!(find_herbrand_base(&funcs), vec![
            make_atom("bar", None),
            make_atom("baz", None),
            make_atom("quz", None),
            make_atom("bar", Some(vec!["bar"])),
            make_atom("bar", Some(vec!["baz"])),
            make_atom("bar", Some(vec!["quz"])),
            make_atom("baz", Some(vec!["bar"])),
            make_atom("baz", Some(vec!["baz"])),
            make_atom("baz", Some(vec!["quz"])),
            make_atom("foo", Some(vec!["bar"])),
            make_atom("foo", Some(vec!["baz"])),
            make_atom("foo", Some(vec!["quz"]))
        ]);

        // Multi-argument functions
        let multi_funcs: Spec = datalog! {
            #![crate]
            foo. bar. baz.
            quz(foo, bar, baz).
        };
        assert_eq!(find_herbrand_base(&multi_funcs), vec![
            make_atom("bar", None),
            make_atom("baz", None),
            make_atom("foo", None),
            make_atom("quz", Some(vec!["bar", "bar", "bar"])),
            make_atom("quz", Some(vec!["bar", "bar", "baz"])),
            make_atom("quz", Some(vec!["bar", "bar", "foo"])),
            make_atom("quz", Some(vec!["bar", "baz", "bar"])),
            make_atom("quz", Some(vec!["bar", "baz", "baz"])),
            make_atom("quz", Some(vec!["bar", "baz", "foo"])),
            make_atom("quz", Some(vec!["bar", "foo", "bar"])),
            make_atom("quz", Some(vec!["bar", "foo", "baz"])),
            make_atom("quz", Some(vec!["bar", "foo", "foo"])),
            make_atom("quz", Some(vec!["baz", "bar", "bar"])),
            make_atom("quz", Some(vec!["baz", "bar", "baz"])),
            make_atom("quz", Some(vec!["baz", "bar", "foo"])),
            make_atom("quz", Some(vec!["baz", "baz", "bar"])),
            make_atom("quz", Some(vec!["baz", "baz", "baz"])),
            make_atom("quz", Some(vec!["baz", "baz", "foo"])),
            make_atom("quz", Some(vec!["baz", "foo", "bar"])),
            make_atom("quz", Some(vec!["baz", "foo", "baz"])),
            make_atom("quz", Some(vec!["baz", "foo", "foo"])),
            make_atom("quz", Some(vec!["foo", "bar", "bar"])),
            make_atom("quz", Some(vec!["foo", "bar", "baz"])),
            make_atom("quz", Some(vec!["foo", "bar", "foo"])),
            make_atom("quz", Some(vec!["foo", "baz", "bar"])),
            make_atom("quz", Some(vec!["foo", "baz", "baz"])),
            make_atom("quz", Some(vec!["foo", "baz", "foo"])),
            make_atom("quz", Some(vec!["foo", "foo", "bar"])),
            make_atom("quz", Some(vec!["foo", "foo", "baz"])),
            make_atom("quz", Some(vec!["foo", "foo", "foo"]))
        ]);

        // Alright now some complex rules
        let rules: Spec = datalog! {
            #![crate]
            foo. bar(foo).
            baz(X) :- bar(X).
        };
        assert_eq!(find_herbrand_base(&rules), vec![
            make_atom("foo", None),
            make_atom("bar", Some(vec!["foo"])),
            make_atom("baz", Some(vec!["foo"]))
        ]);
    }

    #[test]
    fn test_find_largest_unfounded_set() {
        fn make_literal(positive: bool, name: &'static str, args: Option<Vec<&'static str>>) -> Literal {
            if positive {
                Literal::Atom(make_atom(name, args))
            } else {
                Literal::NegAtom(NegAtom { not_token: Not { span: Span::new("make_literal::not", "not") }, atom: make_atom(name, args) })
            }
        }
        fn print_unfounded_set_ordered(set: &HashSet<Literal>) -> String {
            let mut set: Vec<String> = set
                .iter()
                .map(|v| {
                    format!(
                        "{}{}{}",
                        if let Literal::NegAtom(_) = v { "not " } else { "" },
                        v.atom().ident.value.value(),
                        if let Some(args) = &v.atom().args {
                            format!("({})", args.args.values().map(|i| i.ident().value.value()).collect::<Vec<&str>>().join(", "))
                        } else {
                            String::new()
                        }
                    )
                })
                .collect::<Vec<String>>();
            set.sort();
            set.join(", ")
        }


        // Check some empty programs
        let empty: Spec = datalog! { #![crate] };
        let kb: HashMap<Cow<Literal>, bool> = HashMap::new();
        assert_eq!(find_largest_unfounded_set(&empty, &kb), HashSet::new());

        // Check constants
        let consts: Spec = datalog! {
            #![crate]
            foo. bar. baz.
        };
        let kb: HashMap<Cow<Literal>, bool> = HashMap::new();
        assert_eq!(
            find_largest_unfounded_set(&consts, &kb),
            HashSet::from([make_literal(false, "bar", None), make_literal(false, "baz", None), make_literal(false, "foo", None)])
        );

        // Check functions
        let funcs: Spec = datalog! {
            #![crate]
            foo(bar). bar(baz). baz(quz).
        };
        let kb: HashMap<Cow<Literal>, bool> = HashMap::new();
        assert_eq!(
            find_largest_unfounded_set(&funcs, &kb),
            HashSet::from([
                make_literal(true, "bar", None),
                make_literal(false, "bar", None),
                make_literal(true, "baz", None),
                make_literal(false, "baz", None),
                make_literal(true, "quz", None),
                make_literal(false, "quz", None),
                make_literal(false, "foo", Some(vec!["bar"])),
                make_literal(true, "foo", Some(vec!["baz"])),
                make_literal(false, "foo", Some(vec!["baz"])),
                make_literal(true, "foo", Some(vec!["quz"])),
                make_literal(false, "foo", Some(vec!["quz"])),
                make_literal(true, "bar", Some(vec!["bar"])),
                make_literal(false, "bar", Some(vec!["bar"])),
                make_literal(false, "bar", Some(vec!["baz"])),
                make_literal(true, "bar", Some(vec!["quz"])),
                make_literal(false, "bar", Some(vec!["quz"])),
                make_literal(true, "baz", Some(vec!["bar"])),
                make_literal(false, "baz", Some(vec!["bar"])),
                make_literal(true, "baz", Some(vec!["baz"])),
                make_literal(false, "baz", Some(vec!["baz"])),
                make_literal(false, "baz", Some(vec!["quz"])),
            ])
        );

        // Multi-argument functions
        let multi_funcs: Spec = datalog! {
            #![crate]
            foo. bar. baz.
            quz(foo, bar, baz).
        };
        let kb: HashMap<Cow<Literal>, bool> = HashMap::new();
        assert_eq!(
            find_largest_unfounded_set(&multi_funcs, &kb),
            HashSet::from([
                make_literal(false, "bar", None),
                make_literal(false, "baz", None),
                make_literal(false, "foo", None),
                make_literal(true, "quz", Some(vec!["bar", "bar", "bar"])),
                make_literal(true, "quz", Some(vec!["bar", "bar", "baz"])),
                make_literal(true, "quz", Some(vec!["bar", "bar", "foo"])),
                make_literal(true, "quz", Some(vec!["bar", "baz", "bar"])),
                make_literal(true, "quz", Some(vec!["bar", "baz", "baz"])),
                make_literal(true, "quz", Some(vec!["bar", "baz", "foo"])),
                make_literal(true, "quz", Some(vec!["bar", "foo", "bar"])),
                make_literal(true, "quz", Some(vec!["bar", "foo", "baz"])),
                make_literal(true, "quz", Some(vec!["bar", "foo", "foo"])),
                make_literal(true, "quz", Some(vec!["baz", "bar", "bar"])),
                make_literal(true, "quz", Some(vec!["baz", "bar", "baz"])),
                make_literal(true, "quz", Some(vec!["baz", "bar", "foo"])),
                make_literal(true, "quz", Some(vec!["baz", "baz", "bar"])),
                make_literal(true, "quz", Some(vec!["baz", "baz", "baz"])),
                make_literal(true, "quz", Some(vec!["baz", "baz", "foo"])),
                make_literal(true, "quz", Some(vec!["baz", "foo", "bar"])),
                make_literal(true, "quz", Some(vec!["baz", "foo", "baz"])),
                make_literal(true, "quz", Some(vec!["baz", "foo", "foo"])),
                make_literal(true, "quz", Some(vec!["foo", "bar", "bar"])),
                make_literal(true, "quz", Some(vec!["foo", "bar", "foo"])),
                make_literal(true, "quz", Some(vec!["foo", "baz", "bar"])),
                make_literal(true, "quz", Some(vec!["foo", "baz", "baz"])),
                make_literal(true, "quz", Some(vec!["foo", "baz", "foo"])),
                make_literal(true, "quz", Some(vec!["foo", "foo", "bar"])),
                make_literal(true, "quz", Some(vec!["foo", "foo", "baz"])),
                make_literal(true, "quz", Some(vec!["foo", "foo", "foo"])),
                make_literal(false, "quz", Some(vec!["bar", "bar", "bar"])),
                make_literal(false, "quz", Some(vec!["bar", "bar", "baz"])),
                make_literal(false, "quz", Some(vec!["bar", "bar", "foo"])),
                make_literal(false, "quz", Some(vec!["bar", "baz", "bar"])),
                make_literal(false, "quz", Some(vec!["bar", "baz", "baz"])),
                make_literal(false, "quz", Some(vec!["bar", "baz", "foo"])),
                make_literal(false, "quz", Some(vec!["bar", "foo", "bar"])),
                make_literal(false, "quz", Some(vec!["bar", "foo", "baz"])),
                make_literal(false, "quz", Some(vec!["bar", "foo", "foo"])),
                make_literal(false, "quz", Some(vec!["baz", "bar", "bar"])),
                make_literal(false, "quz", Some(vec!["baz", "bar", "baz"])),
                make_literal(false, "quz", Some(vec!["baz", "bar", "foo"])),
                make_literal(false, "quz", Some(vec!["baz", "baz", "bar"])),
                make_literal(false, "quz", Some(vec!["baz", "baz", "baz"])),
                make_literal(false, "quz", Some(vec!["baz", "baz", "foo"])),
                make_literal(false, "quz", Some(vec!["baz", "foo", "bar"])),
                make_literal(false, "quz", Some(vec!["baz", "foo", "baz"])),
                make_literal(false, "quz", Some(vec!["baz", "foo", "foo"])),
                make_literal(false, "quz", Some(vec!["foo", "bar", "bar"])),
                make_literal(false, "quz", Some(vec!["foo", "bar", "baz"])),
                make_literal(false, "quz", Some(vec!["foo", "bar", "foo"])),
                make_literal(false, "quz", Some(vec!["foo", "baz", "bar"])),
                make_literal(false, "quz", Some(vec!["foo", "baz", "baz"])),
                make_literal(false, "quz", Some(vec!["foo", "baz", "foo"])),
                make_literal(false, "quz", Some(vec!["foo", "foo", "bar"])),
                make_literal(false, "quz", Some(vec!["foo", "foo", "baz"])),
                make_literal(false, "quz", Some(vec!["foo", "foo", "foo"]))
            ])
        );

        // Alright now some complex rules
        let rules: Spec = datalog! {
            #![crate]
            foo. bar(foo).
            baz(X) :- bar(X).
        };
        let kb: HashMap<Cow<Literal>, bool> = HashMap::new();
        assert_eq!(
            find_largest_unfounded_set(&rules, &kb),
            HashSet::from([
                make_literal(false, "foo", None),
                make_literal(false, "bar", Some(vec!["foo"])),
                make_literal(false, "baz", Some(vec!["foo"]))
            ])
        );
        let kb: HashMap<Cow<Literal>, bool> = HashMap::from([(Cow::Owned(make_literal(true, "bar", Some(vec!["foo"]))), true)]);
        println!("LEFT {}", print_unfounded_set_ordered(&find_largest_unfounded_set(&rules, &kb)));
        println!(
            "RIGHT {}",
            print_unfounded_set_ordered(&HashSet::from([
                make_literal(false, "foo", None),
                make_literal(false, "bar", Some(vec!["foo"])),
                make_literal(true, "baz", Some(vec!["foo"])),
                make_literal(false, "baz", Some(vec!["foo"]))
            ]))
        );
        assert_eq!(
            find_largest_unfounded_set(&rules, &kb),
            HashSet::from([
                make_literal(false, "foo", None),
                make_literal(false, "bar", Some(vec!["foo"])),
                make_literal(true, "baz", Some(vec!["foo"])),
                make_literal(false, "baz", Some(vec!["foo"]))
            ])
        );
    }
}





/***** HELPER FUNCTIONS *****/
/// Finds the Herbrand Base for the given [`Spec`].
///
/// The _Herbrand Base_ of a program is the set of all (known) concrete atoms in it. Concretely, this is:
/// - All constants in the program (functions with arity 0); and
/// - All functions with the arity used in the program, concretized for every constant.
///
/// # Arguments
/// - `spec`: The [`Spec`]ification to find the Herbrand Base of.
///
/// # Returns
/// A vector with the found base, givne as a list of [`Atom`]s. None of these have variables in them.
fn find_herbrand_base(spec: &Spec) -> Vec<Atom> {
    // Organise all atoms in the spec into constants and functions
    let mut constants: Vec<&Ident> = Vec::new();
    let mut functions: Vec<(&Ident, &Parens, usize)> = Vec::new();
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
    functions
        .sort_by(|(lhs, _, lhs_arity), (rhs, _, rhs_arity)| -> Ordering { lhs.value.value().cmp(rhs.value.value()).then(lhs_arity.cmp(rhs_arity)) });
    functions.dedup_by(|(lhs, _, lhs_arity), (rhs, _, rhs_arity)| -> bool { lhs.value.value() == rhs.value.value() && lhs_arity == rhs_arity });

    // Now re-generate the functions into all possible combinations of them + constants
    // NOTE: The re-building of the atom below is quite cheap, because constants don't have argument vectors to clone
    let mut herbrand_base: Vec<Atom> = constants.iter().map(|i| Atom { ident: **i, args: None }).collect();
    herbrand_base.reserve(functions.len() * constants.len());
    for (func, parens, arity) in functions {
        #[cfg(debug_assertions)]
        assert!(arity > 0);

        // Iterate over the function's arity
        for args in (0..arity).map(|_| constants.iter()).multi_cartesian_product() {
            // Turn it into a punctuated list
            let mut puncs: Punctuated<AtomArg, Comma> = Punctuated::new();
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
    herbrand_base
}

/// Checks if a given rule would produce a given atom.
///
/// In other words: checks if the given atom is in the rule's consequents. Note that it is OK for the rule to have variables (which are treated as wildcards), but NOT for the atom itself (it must be based).
///
/// # Arguments
/// - `rule`: The [`Rule`] to inspect.
/// - `atom`: The (concrete!) [`Atom`] to check if it is produced by the given `rule`.
///
/// # Returns
/// True if this is the case, or false otherwise.
fn rule_produces_atom(rule: &Rule, atom: &Atom) -> bool {
    // Check if the atom appears in this rule's consequences
    // A bit of a complex search, but basically just ensures that the arity is correct AND the atoms match _or_ the consequence has a variable (i.e., variables are wildcards).
    for cons in rule.consequences.values() {
        if atom.ident.value.value() == cons.ident.value.value() {
            // Check if the functions match
            match (&atom.args, &cons.args) {
                // If they both have arguments, ensure they are the same
                (Some(aargs), Some(cargs)) => {
                    // Check for matching arity
                    if aargs.args.len() != cargs.args.len() {
                        return false;
                    }

                    // Check if the individual identifiers match up
                    // NOTE: Variables are counted as wildcards
                    for (aarg, carg) in aargs.args.values().zip(cargs.args.values()) {
                        match (aarg, carg) {
                            (AtomArg::Atom(a), AtomArg::Atom(c)) => {
                                if a.value.value() != c.value.value() {
                                    return false;
                                }
                            },
                            (AtomArg::Atom(_), AtomArg::Var(_)) => continue,
                            (_, _) => unreachable!(),
                        }
                    }
                },
                // It's also OK if neither of them have arguments
                (None, None) => continue,
                // But if one of them do and the other don't, that's sad
                (_, _) => return false,
            }
        } else {
            return false;
        }
    }
    true
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
fn find_largest_unfounded_set<'p, 'f, 's>(spec: &'p Spec, kb: &HashMap<Cow<'p, Literal>, bool>) -> HashSet<Literal> {
    // Iterate over the Herbrand Base to find candidates
    // This process is iterative, to ensure that we catch later atoms being added that make earlier atoms unfounded.
    let mut unfounded_set: HashSet<Literal> = HashSet::new();
    'toplevel: loop {
        // Start looping
        for atom in find_herbrand_base(spec) {
            // Find rules producing this atom
            let mut found_at_least_one_rule: bool = false;
            for rule in &spec.rules {
                if !rule_produces_atom(rule, &atom) {
                    continue;
                }
                found_at_least_one_rule = true;

                // For the atom to be part of the unfounded set, the rule either:
                // 1. Has to have a false antecedent in the knowledge base; or
                // 2. Has to have positive antecedent in the already found unfounded set.
                for ante in rule.tail.iter().map(|t| t.antecedents.values()).flatten() {
                    match ante {
                        Literal::Atom(_) => {
                            // (1.) If it's a positive atom, we're searching for a negative presense
                            // let lit: Literal<String, String> = Literal::Atom(Atom {});
                            if let Some(false) = kb.get(ante) {
                                if !unfounded_set.insert(Literal::Atom(atom.clone())) {
                                    continue 'toplevel;
                                }
                            }

                            // (2.) See if the current unfounded set already has this beauty
                            if unfounded_set.contains(ante) {
                                if !unfounded_set.insert(Literal::Atom(atom.clone())) {
                                    continue 'toplevel;
                                }
                            }
                        },
                        Literal::NegAtom(_) => {
                            // (1.) If it's a negative atom, we're searching for a positive precense
                            if let Some(true) = kb.get(ante) {
                                if !unfounded_set.insert(Literal::Atom(atom.clone())) {
                                    // This was a new literal, we gotta go at it again
                                    continue 'toplevel;
                                }
                            }

                            // (2.) No need, we only consider positive atoms
                        },
                    }
                }
            }

            // Emulate reasoning in absentia; if we never found a rule, it's in the unfounded set too (this actually always includes negative literals by Datalog's design)
            if !found_at_least_one_rule {
                unfounded_set.insert(Literal::Atom(atom.clone()));
            }
            unfounded_set.insert(Literal::NegAtom(NegAtom { not_token: Not { span: atom.span() }, atom }));
        }

        // If we never added anyone, we converged
        break;
    }

    // OK, return the found set
    unfounded_set
}





/***** LIBRARY *****/
/// Evaluates a given $Datalog^\neg$ AST.
///
/// Contains a knowledge base internally. That means that different interpreter instances may give different answers.
#[derive(Clone, Debug)]
pub struct Interpreter {
    /// The set of facts that we know exist.
    pub knowledge_base: IndexSet<Atom>,
}
impl Default for Interpreter {
    #[inline]
    fn default() -> Self { Self::new() }
}
impl Interpreter {
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
    pub fn evaluate_once(spec: &Spec) -> IndexSet<Atom> {
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
    pub fn evaluate(&mut self, spec: &Spec) {
        // //

        // // Go thru the rules
        // for rule in &spec.rules {
        //     // Consider all concrete instances based on variables
        //     let mut new_instances: IndexSet<Atom> = IndexSet::new();
        //     for concrete_rule in RuleConcretizer::new(rule, &self.knowledge_base) {}
        // }
    }
}
