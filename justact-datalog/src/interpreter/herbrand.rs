//  HERBRAND.rs
//    by Lut99
//
//  Created:
//    21 Mar 2024, 10:55:27
//  Last edited:
//    21 Mar 2024, 17:18:58
//  Auto updated?
//    Yes
//
//  Description:
//!   Implements iterators for the Herbrand instantiation of a program.
//

use indexmap::IndexSet;

use crate::ast::{Atom, AtomArg, Ident, Literal, Rule, Spec};


/***** TESTS *****/
#[cfg(all(test, feature = "derive"))]
mod tests {
    use ast_toolkit_span::Span;
    use justact_datalog_derive::datalog;

    use super::*;


    /// Makes an [`Ident`] conveniently.
    fn make_ident(name: &'static str) -> Ident { Ident { value: Span::new("make_ident::value", name) } }


    #[test]
    fn test_herbrand_base_iterator() {
        // Check empty specs
        let empty: Spec = datalog! { #![crate] };
        let mut iter = HerbrandBaseIterator::new(&empty);
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);

        // Check with a few atoms
        let cons: Spec = datalog! {
            #![crate]

            foo. bar. bar. baz.
        };
        let mut iter = HerbrandBaseIterator::new(&cons);
        assert_eq!(iter.next(), Some(make_ident("foo")));
        assert_eq!(iter.next(), Some(make_ident("bar")));
        assert_eq!(iter.next(), Some(make_ident("bar")));
        assert_eq!(iter.next(), Some(make_ident("baz")));
        assert_eq!(iter.next(), None);

        // Check with functions
        let funcs: Spec = datalog! {
            #![crate]

            foo(bar). bar(baz, quz). baz(quz).
        };
        let mut iter = HerbrandBaseIterator::new(&funcs);
        assert_eq!(iter.next(), Some(make_ident("bar")));
        assert_eq!(iter.next(), Some(make_ident("baz")));
        assert_eq!(iter.next(), Some(make_ident("quz")));
        assert_eq!(iter.next(), Some(make_ident("quz")));
        assert_eq!(iter.next(), None);

        // Check with rules
        let rules: Spec = datalog! {
            #![crate]

            foo. bar.
            baz(X) :- quz.
        };
        let mut iter = HerbrandBaseIterator::new(&rules);
        assert_eq!(iter.next(), Some(make_ident("foo")));
        assert_eq!(iter.next(), Some(make_ident("bar")));
        assert_eq!(iter.next(), Some(make_ident("quz")));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_herbrand_instantiation_iterator() {
        // Check empty specs
        let empty: Spec = datalog! { #![crate] };
        let hbase: IndexSet<Ident> = HerbrandBaseIterator::new(&empty).collect();
        let mut iter = HerbrandInstantiationIterator::new(&empty, &hbase);
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);

        // Check with a few atoms
        let cons: Spec = datalog! {
            #![crate]

            foo. bar. bar. baz.
        };
        let hbase: IndexSet<Ident> = HerbrandBaseIterator::new(&cons).collect();
        let mut iter = HerbrandInstantiationIterator::new(&cons, &hbase);
        assert_eq!(iter.next(), Some(&datalog! { #![crate] foo. }.rules[0]));
        assert_eq!(iter.next(), Some(&datalog! { #![crate] bar. }.rules[0]));
        assert_eq!(iter.next(), Some(&datalog! { #![crate] bar. }.rules[0]));
        assert_eq!(iter.next(), Some(&datalog! { #![crate] baz. }.rules[0]));
        assert_eq!(iter.next(), None);

        // Check with functions
        let funcs: Spec = datalog! {
            #![crate]

            foo(bar). bar(baz, quz). baz(quz).
        };
        let hbase: IndexSet<Ident> = HerbrandBaseIterator::new(&funcs).collect();
        let mut iter = HerbrandInstantiationIterator::new(&funcs, &hbase);
        assert_eq!(iter.next(), Some(&datalog! { #![crate] foo(bar). }.rules[0]));
        assert_eq!(iter.next(), Some(&datalog! { #![crate] bar(baz, quz). }.rules[0]));
        assert_eq!(iter.next(), Some(&datalog! { #![crate] baz(quz). }.rules[0]));
        assert_eq!(iter.next(), None);

        // Check with rules
        let rules: Spec = datalog! {
            #![crate]

            foo. bar.
            baz(X) :- quz.
        };
        let hbase: IndexSet<Ident> = HerbrandBaseIterator::new(&rules).collect();
        let mut iter = HerbrandInstantiationIterator::new(&rules, &hbase);
        assert_eq!(iter.next(), Some(&datalog! { #![crate] foo. }.rules[0]));
        assert_eq!(iter.next(), Some(&datalog! { #![crate] bar. }.rules[0]));
        assert_eq!(iter.next(), Some(&datalog! { #![crate] baz(foo) :- quz. }.rules[0]));
        assert_eq!(iter.next(), Some(&datalog! { #![crate] baz(bar) :- quz. }.rules[0]));
        assert_eq!(iter.next(), Some(&datalog! { #![crate] baz(quz) :- quz. }.rules[0]));
        assert_eq!(iter.next(), None);

        // Longer rules
        let multi_rules: Spec = datalog! {
            #![crate]

            foo. bar. baz(foo, bar).
            quz(X, Y) :- baz(X, Y).
        };
        let hbase: IndexSet<Ident> = HerbrandBaseIterator::new(&multi_rules).collect();
        let mut iter = HerbrandInstantiationIterator::new(&multi_rules, &hbase);
        assert_eq!(iter.next(), Some(&datalog! { #![crate] foo. }.rules[0]));
        assert_eq!(iter.next(), Some(&datalog! { #![crate] bar. }.rules[0]));
        assert_eq!(iter.next(), Some(&datalog! { #![crate] baz(foo, bar). }.rules[0]));
        assert_eq!(iter.next(), Some(&datalog! { #![crate] quz(foo, foo) :- baz(foo, foo). }.rules[0]));
        assert_eq!(iter.next(), Some(&datalog! { #![crate] quz(foo, bar) :- baz(foo, bar). }.rules[0]));
        assert_eq!(iter.next(), Some(&datalog! { #![crate] quz(bar, foo) :- baz(bar, foo). }.rules[0]));
        assert_eq!(iter.next(), Some(&datalog! { #![crate] quz(bar, bar) :- baz(bar, bar). }.rules[0]));
        assert_eq!(iter.next(), None);
    }
}




/***** TYPE ALIASES *****/
/// The internal for the HerbrandInstantiationIterator variable assignment iterator.
type VarAssignInternal<'h> = std::iter::Flatten<std::iter::Take<std::iter::Repeat<indexmap::set::Iter<'h, Ident>>>>;

/// The external for the HerbrandInstantiationIterator variable assignment iterator.
type VarAssign<'h> = std::iter::Flatten<std::iter::Take<std::iter::Repeat<VarAssignInternal<'h>>>>;





/***** HELPER FUNCTIONS *****/
/// Returns the variables in the given rule.
///
/// # Arguments
/// - `rule`: The [`Rule`] to analyze.
/// - `vars`: A list of variables names that we recycle and populate.
fn find_vars_in_rule(rule: &Rule, vars: &mut IndexSet<Ident>) {
    vars.clear();
    for cons in rule.consequences.values() {
        for arg in cons.args.iter().map(|a| a.args.values()).flatten() {
            if let AtomArg::Var(var) = arg {
                vars.insert(*var);
            }
        }
    }
    for ante in rule.tail.iter().map(|t| t.antecedents.values()).flatten() {
        for arg in ante.atom().args.iter().map(|a| a.args.values()).flatten() {
            if let AtomArg::Var(var) = arg {
                vars.insert(*var);
            }
        }
    }
}

/// Generates a new set of iterators for the given [`Rule`].
///
/// # Arguments
/// - `hbase`: The Herbrand base to spawn new iterators with.
/// - `rule`: A [`Rule`] to search for variables and such.
/// - `vars`: The variables that we will find in this rule.
/// - `iters`: A new set of iterators to spawn.
fn refresh_iters<'h>(hbase: &'h IndexSet<Ident>, rule: &'_ Rule, vars: &'_ mut IndexSet<Ident>, iters: &'_ mut Vec<VarAssign<'h>>) -> Option<Rule> {
    // Find the (unique!) variables in the rule and decide if we're cloning or borrowing the rule
    find_vars_in_rule(&rule, vars);

    // Create iterators for that. We stack the iterators to emulate doing a binary-counting-like search-space.
    iters.clear();
    iters.reserve(vars.len());
    for i in 0..vars.len() {
        // We scale from essentially doing `111111...333333`, to `111222...222333`, to `123123...123123`
        iters.push(std::iter::repeat(std::iter::repeat(hbase.iter()).take(1 << (vars.len() - 1 - i)).flatten()).take(1 << i).flatten());
    }

    // Define what to return
    if vars.len() > 0 { Some(rule.clone()) } else { None }
}

/// Repopulates the given rule with the given mapping.
///
/// The values are assigned in-order as the variables are encountered. Since this iteration over the rule is deterministic, so should the assignment be.
///
/// # Arguments
/// - `rule`: The original rule that knows where variables are.
/// - `vars`: Defines the names of variables. Given as an [`IndexSet`] for speedier search, while the order is important to match with the assignment.
/// - `values`: The values mapping for the given `vars`.
/// - `gen_rule`: The rule to repopulate.
fn repopulate_rule(rule: &Rule, vars: &IndexSet<Ident>, values: &[Ident], gen_rule: &mut Rule) {
    for (c, cons) in rule.consequences.values().enumerate() {
        for (a, arg) in cons.args.iter().map(|a| a.args.values()).flatten().enumerate() {
            if matches!(arg, AtomArg::Var(_)) {
                // Find this variable's index in the mapping
                // SAFETY: We can unwrap here because we assume the caller has given us a mapping for this rule.
                let idx: usize = vars.get_index_of(arg.ident()).expect("Found variable in rule that was not in mapping");
                gen_rule.consequences[c].args.as_mut().unwrap().args[a] = AtomArg::Atom(values[idx]);
            }
        }
    }
    for (t, ante) in rule.tail.iter().map(|t| t.antecedents.values()).flatten().enumerate() {
        for (a, arg) in ante.atom().args.iter().map(|a| a.args.values()).flatten().enumerate() {
            if matches!(arg, AtomArg::Var(_)) {
                // Find this variable's index in the mapping
                // SAFETY: We can unwrap here because we assume the caller has given us a mapping for this rule.
                let idx: usize = vars.get_index_of(arg.ident()).expect("Found variable in rule that was not in mapping");
                gen_rule.tail.as_mut().unwrap().antecedents[t].atom_mut().args.as_mut().unwrap().args[a] = AtomArg::Atom(values[idx]);
            }
        }
    }
}

/// Find the next mapping given the set of iterators.
///
/// # Arguments
/// - `rule`: Some rule to use to generate new iterators if it proves necessary.
/// - `iters`: The iterators to pass. We assume that by some clever usage of [`std::iter::repeat`], any binary-like counting is embedded.
/// - `assign`: The assignment to populate.
///
/// # Returns
/// Whether we found a next mapping. If false, this means that we ran out of mappings to generate.
fn get_next_mapping(iters: &mut Vec<VarAssign>, assign: &mut Vec<Ident>) -> bool {
    assign.clear();
    assign.reserve(iters.len());
    for iter in iters {
        match iter.next() {
            Some(next) => assign.push(*next),
            None => return false,
        }
    }
    true
}





/***** LIBRARY *****/
/// Defines an iterator over the "Herbrand Base" of a program.
///
/// Because we will use this to create the Herbrand instantiation of a [`Spec`] (see the [`HerbrandInstantiationIterator`]), and because $Datalog^\neg$ atoms cannot have arguments with arity > 0, this only produces:
/// - All atoms in the spec with arity 0.
pub struct HerbrandBaseIterator<'s> {
    /// The set of rules to iterate over.
    rules: std::slice::Iter<'s, Rule>,
    /// The current antecedents to iterate over.
    ants:  Box<dyn 's + Iterator<Item = &'s Literal>>,
    /// The current consequences to iterate over.
    cons:  Box<dyn 's + Iterator<Item = &'s Atom>>,
    /// Finally, the current arguments to iterate over.
    args:  Box<dyn 's + Iterator<Item = Ident>>,
}
impl<'s> HerbrandBaseIterator<'s> {
    /// Constructor for a HerbrandBaseIterator.
    ///
    /// # Arguments
    /// - `spec`: Some [`Spec`] to iterate over.
    ///
    /// # Returns
    /// An HerbrandBaseIterator that will produce all constants (i.e., arity-0, grounded atoms) in the `spec`.
    pub fn new(spec: &'s Spec) -> Self {
        // Build ourselves with nothing in it yet but a rule
        Self { rules: spec.rules.iter(), ants: Box::new(None.into_iter()), cons: Box::new(None.into_iter()), args: Box::new(None.into_iter()) }
    }
}
impl<'s> Iterator for HerbrandBaseIterator<'s> {
    type Item = Ident;

    fn next(&mut self) -> Option<Self::Item> {
        // Go through the iterators one-by-one
        self.args
            .next()
            .or_else(|| {
                // If we got here, that means there's no more arguments for the parent atom; get the next consequent
                self.cons
                    .next()
                    .map(|c| {
                        if let Some(args) = &c.args {
                            // Not gonna return this one, not a constant. Instead, try the args again
                            self.args = Box::new(args.args.values().filter_map(|a| if let AtomArg::Atom(a) = a { Some(*a) } else { None }));
                            self.next()
                        } else {
                            Some(c.ident)
                        }
                    })
                    .flatten()
            })
            .or_else(|| {
                // If we got here, that means there's no more arguments for the parent atom _or_ consequences; get the next antecedent
                self.ants
                    .next()
                    .map(|a| {
                        if let Some(args) = &a.atom().args {
                            // Not gonna return this one, not a constant. Instead, try the args again
                            self.args = Box::new(args.args.values().filter_map(|a| if let AtomArg::Atom(a) = a { Some(*a) } else { None }));
                            self.next()
                        } else {
                            Some(a.atom().ident)
                        }
                    })
                    .flatten()
            })
            .or_else(|| {
                // If we got here, that means there's no more consequents or antecedents for this rule. Move to the next.
                let rule: &'s Rule = self.rules.next()?;
                self.cons = Box::new(rule.consequences.values());
                self.ants = Box::new(rule.tail.iter().map(|t| t.antecedents.values()).flatten());
                self.next()
            })
    }
}

/// Defines an iterator over all the rules in the _concretized_ version of the given [`Spec`].
///
/// Concretely, this produces the spec rule-by-rule, except that all rules with variables will be replicated for every possible instantiation of those variables. For $Datalog^\neg$, this simply means all possible combinations of atoms given in the spec with arity 0 is tried.
#[derive(Clone, Debug)]
pub struct HerbrandInstantiationIterator<'h, 's> {
    /// An iterator producing rules from the [`Spec`].
    rules: std::slice::Iter<'s, Rule>,
    /// The Herbrand base given for the spec.
    hbase: &'h IndexSet<Ident>,

    /// The original rule we're currently considering and an optional clone to modify with concrete instances. Is [`None`] if the rule has no variables.
    rule:   Option<(&'s Rule, Option<Rule>)>,
    /// Defines a buffer for storing which variables occur in the `rule` above.
    vars:   IndexSet<Ident>,
    /// Defines a buffer for iterating all possible value assignments of the `rule`` above.
    iters:  Vec<VarAssign<'h>>,
    /// Defines a buffer for storing the current value assignment for the `vars` above.
    assign: Vec<Ident>,
}
impl<'h, 's> HerbrandInstantiationIterator<'h, 's> {
    /// Constructor for a HerbrandInstantiationIterator.
    ///
    /// # Arguments
    /// - `spec`: Some [`Spec`] to iterate over.
    /// - `hbase`: A Herbrand Base of the given spec. This is not computed automatically, because then this iterator would become self-referential.
    ///
    /// # Returns
    /// An HerbrandInstantiationIterator that will produce all concrete rules in the given `spec`.
    pub fn new(spec: &'s Spec, hbase: &'h IndexSet<Ident>) -> Self {
        // Nothing to do if the Herbrand Base is empty
        if hbase.is_empty() {
            return Self { rules: spec.rules.iter(), hbase, rule: None, vars: IndexSet::new(), iters: vec![], assign: vec![] };
        }

        // Find the number of variables in the rule and generate the Herbrand Base iterators for that.
        let mut rules = spec.rules.iter();
        match rules.next() {
            Some(rule) => {
                // Find the (unique!) variables in the rule & generate iterators accordingly
                let mut vars: IndexSet<_> = IndexSet::new();
                let mut iters: Vec<_> = Vec::new();
                let gen_rule: Option<Rule> = refresh_iters(hbase, rule, &mut vars, &mut iters);

                // OK done
                Self { rules, hbase, rule: Some((rule, gen_rule)), iters, vars, assign: Vec::new() }
            },
            None => {
                // Nothing to do anyway
                Self { rules, hbase, rule: None, vars: IndexSet::new(), iters: vec![], assign: vec![] }
            },
        }
    }

    /// Returns the next element in the iterator.
    ///
    /// This is not implemented as an [`Iterator`], because this struct returns an optimized, already-allocated [`Rule`] with a reference to `self` (and [`Iterator`] does not GAT).
    ///
    /// # Returns
    /// A reference to the concrete rule, or else [`None`] if we had all rules.
    pub fn next(&mut self) -> Option<&Rule> {
        match self.rule.take() {
            Some((rule, None)) => {
                // It's one-time only; so get the next rule, then return the original reference
                let new_rule: &'s Rule = match self.rules.next() {
                    Some(rule) => rule,
                    None => {
                        // Only the old rule left to return...
                        return Some(rule);
                    },
                };

                // Refresh the iterators
                self.rule = Some((new_rule, refresh_iters(self.hbase, new_rule, &mut self.vars, &mut self.iters)));

                // OK, old rule here we go
                Some(rule)
            },
            Some((rule, Some(mut gen_rule))) => {
                // It's for generate rules; so get the next variable mapping
                if !get_next_mapping(&mut self.iters, &mut self.assign) {
                    // Done with self rule. Time to get the new one.
                    let new_rule: &'s Rule = match self.rules.next() {
                        Some(rule) => rule,
                        None => {
                            // Really done! Nothing more to do
                            self.rule = None;
                            return None;
                        },
                    };

                    // Find the (unique!) variables in the rule and decide if we're cloning or borrowing the rule
                    self.rule = Some((new_rule, refresh_iters(self.hbase, new_rule, &mut self.vars, &mut self.iters)));
                }

                // Populate the rule with the given variable -> value assignment
                repopulate_rule(rule, &self.vars, &self.assign, &mut gen_rule);
                self.rule = Some((rule, Some(gen_rule)));
                Some(unsafe { self.rule.as_ref().unwrap_unchecked().1.as_ref().unwrap_unchecked() })
            },
            // Actually never occurs except for the first rule because the borrow checker doesn't allow us setting the `rule` again mid-function, very sad
            None => None,
        }
    }
}
