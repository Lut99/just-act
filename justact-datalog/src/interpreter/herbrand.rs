//  HERBRAND.rs
//    by Lut99
//
//  Created:
//    21 Mar 2024, 10:55:27
//  Last edited:
//    22 Mar 2024, 18:00:06
//  Auto updated?
//    Yes
//
//  Description:
//!   Implements iterators for the Herbrand instantiation of a program.
//

use std::borrow::Cow;
use std::iter::{Flatten, Repeat, Take};

use indexmap::{IndexMap, IndexSet};

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
    fn test_refresh_iters() {
        // Test a single variable
        let hbase: IndexSet<Ident> = IndexSet::from([make_ident("foo"), make_ident("bar"), make_ident("baz")]);
        let mut vars: IndexMap<Ident, RepeatIterator<indexmap::set::Iter<Ident>>> = IndexMap::new();
        refresh_iters(&hbase, &datalog! { #![crate] foo(X) :- bar(X). }.rules[0], &mut vars);
        assert_eq!(vars.len(), 1);
        assert_eq!(vec![vars[0].next()], vec![Some(&make_ident("foo"))]);
        assert_eq!(vec![vars[0].next()], vec![Some(&make_ident("bar"))]);
        assert_eq!(vec![vars[0].next()], vec![Some(&make_ident("baz"))]);
        assert_eq!(vec![vars[0].next()], vec![None]);

        // Test two distinct variables
        refresh_iters(&hbase, &datalog! { #![crate] foo(X) :- bar(Y). }.rules[0], &mut vars);
        assert_eq!(vars.len(), 2);
        assert_eq!(vec![vars[0].next(), vars[1].next()], vec![Some(&make_ident("foo")), Some(&make_ident("foo"))]);
        assert_eq!(vec![vars[0].next(), vars[1].next()], vec![Some(&make_ident("foo")), Some(&make_ident("bar"))]);
        assert_eq!(vec![vars[0].next(), vars[1].next()], vec![Some(&make_ident("foo")), Some(&make_ident("baz"))]);
        assert_eq!(vec![vars[0].next(), vars[1].next()], vec![Some(&make_ident("bar")), Some(&make_ident("foo"))]);
        assert_eq!(vec![vars[0].next(), vars[1].next()], vec![Some(&make_ident("bar")), Some(&make_ident("bar"))]);
        assert_eq!(vec![vars[0].next(), vars[1].next()], vec![Some(&make_ident("bar")), Some(&make_ident("baz"))]);
        assert_eq!(vec![vars[0].next(), vars[1].next()], vec![Some(&make_ident("baz")), Some(&make_ident("foo"))]);
        assert_eq!(vec![vars[0].next(), vars[1].next()], vec![Some(&make_ident("baz")), Some(&make_ident("bar"))]);
        assert_eq!(vec![vars[0].next(), vars[1].next()], vec![Some(&make_ident("baz")), Some(&make_ident("baz"))]);
        assert_eq!(vec![vars[0].next(), vars[1].next()], vec![None, None]);

        // Test mixed distinct and the same
        refresh_iters(&hbase, &datalog! { #![crate] foo(X, Y) :- bar(Y, Z). }.rules[0], &mut vars);
        assert_eq!(vars.len(), 3);
        assert_eq!(vec![vars[0].next(), vars[1].next(), vars[2].next()], vec![
            Some(&make_ident("foo")),
            Some(&make_ident("foo")),
            Some(&make_ident("foo"))
        ]);
        assert_eq!(vec![vars[0].next(), vars[1].next(), vars[2].next()], vec![
            Some(&make_ident("foo")),
            Some(&make_ident("foo")),
            Some(&make_ident("bar"))
        ]);
        assert_eq!(vec![vars[0].next(), vars[1].next(), vars[2].next()], vec![
            Some(&make_ident("foo")),
            Some(&make_ident("foo")),
            Some(&make_ident("baz"))
        ]);
        assert_eq!(vec![vars[0].next(), vars[1].next(), vars[2].next()], vec![
            Some(&make_ident("foo")),
            Some(&make_ident("bar")),
            Some(&make_ident("foo"))
        ]);
        assert_eq!(vec![vars[0].next(), vars[1].next(), vars[2].next()], vec![
            Some(&make_ident("foo")),
            Some(&make_ident("bar")),
            Some(&make_ident("bar"))
        ]);
        assert_eq!(vec![vars[0].next(), vars[1].next(), vars[2].next()], vec![
            Some(&make_ident("foo")),
            Some(&make_ident("bar")),
            Some(&make_ident("baz"))
        ]);
        assert_eq!(vec![vars[0].next(), vars[1].next(), vars[2].next()], vec![
            Some(&make_ident("foo")),
            Some(&make_ident("baz")),
            Some(&make_ident("foo"))
        ]);
        assert_eq!(vec![vars[0].next(), vars[1].next(), vars[2].next()], vec![
            Some(&make_ident("foo")),
            Some(&make_ident("baz")),
            Some(&make_ident("bar"))
        ]);
        assert_eq!(vec![vars[0].next(), vars[1].next(), vars[2].next()], vec![
            Some(&make_ident("foo")),
            Some(&make_ident("baz")),
            Some(&make_ident("baz"))
        ]);
        assert_eq!(vec![vars[0].next(), vars[1].next(), vars[2].next()], vec![
            Some(&make_ident("bar")),
            Some(&make_ident("foo")),
            Some(&make_ident("foo"))
        ]);
        assert_eq!(vec![vars[0].next(), vars[1].next(), vars[2].next()], vec![
            Some(&make_ident("bar")),
            Some(&make_ident("foo")),
            Some(&make_ident("bar"))
        ]);
        assert_eq!(vec![vars[0].next(), vars[1].next(), vars[2].next()], vec![
            Some(&make_ident("bar")),
            Some(&make_ident("foo")),
            Some(&make_ident("baz"))
        ]);
        assert_eq!(vec![vars[0].next(), vars[1].next(), vars[2].next()], vec![
            Some(&make_ident("bar")),
            Some(&make_ident("bar")),
            Some(&make_ident("foo"))
        ]);
        assert_eq!(vec![vars[0].next(), vars[1].next(), vars[2].next()], vec![
            Some(&make_ident("bar")),
            Some(&make_ident("bar")),
            Some(&make_ident("bar"))
        ]);
        assert_eq!(vec![vars[0].next(), vars[1].next(), vars[2].next()], vec![
            Some(&make_ident("bar")),
            Some(&make_ident("bar")),
            Some(&make_ident("baz"))
        ]);
        assert_eq!(vec![vars[0].next(), vars[1].next(), vars[2].next()], vec![
            Some(&make_ident("bar")),
            Some(&make_ident("baz")),
            Some(&make_ident("foo"))
        ]);
        assert_eq!(vec![vars[0].next(), vars[1].next(), vars[2].next()], vec![
            Some(&make_ident("bar")),
            Some(&make_ident("baz")),
            Some(&make_ident("bar"))
        ]);
        assert_eq!(vec![vars[0].next(), vars[1].next(), vars[2].next()], vec![
            Some(&make_ident("bar")),
            Some(&make_ident("baz")),
            Some(&make_ident("baz"))
        ]);
        assert_eq!(vec![vars[0].next(), vars[1].next(), vars[2].next()], vec![
            Some(&make_ident("baz")),
            Some(&make_ident("foo")),
            Some(&make_ident("foo"))
        ]);
        assert_eq!(vec![vars[0].next(), vars[1].next(), vars[2].next()], vec![
            Some(&make_ident("baz")),
            Some(&make_ident("foo")),
            Some(&make_ident("bar"))
        ]);
        assert_eq!(vec![vars[0].next(), vars[1].next(), vars[2].next()], vec![
            Some(&make_ident("baz")),
            Some(&make_ident("foo")),
            Some(&make_ident("baz"))
        ]);
        assert_eq!(vec![vars[0].next(), vars[1].next(), vars[2].next()], vec![
            Some(&make_ident("baz")),
            Some(&make_ident("bar")),
            Some(&make_ident("foo"))
        ]);
        assert_eq!(vec![vars[0].next(), vars[1].next(), vars[2].next()], vec![
            Some(&make_ident("baz")),
            Some(&make_ident("bar")),
            Some(&make_ident("bar"))
        ]);
        assert_eq!(vec![vars[0].next(), vars[1].next(), vars[2].next()], vec![
            Some(&make_ident("baz")),
            Some(&make_ident("bar")),
            Some(&make_ident("baz"))
        ]);
        assert_eq!(vec![vars[0].next(), vars[1].next(), vars[2].next()], vec![
            Some(&make_ident("baz")),
            Some(&make_ident("baz")),
            Some(&make_ident("foo"))
        ]);
        assert_eq!(vec![vars[0].next(), vars[1].next(), vars[2].next()], vec![
            Some(&make_ident("baz")),
            Some(&make_ident("baz")),
            Some(&make_ident("bar"))
        ]);
        assert_eq!(vec![vars[0].next(), vars[1].next(), vars[2].next()], vec![
            Some(&make_ident("baz")),
            Some(&make_ident("baz")),
            Some(&make_ident("baz"))
        ]);
        assert_eq!(vec![vars[0].next(), vars[1].next(), vars[2].next()], vec![None, None, None]);
    }

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
        let mut iter = ConstantIterator::new(HerbrandBaseIterator::new(&cons));
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
        let mut iter = ConstantIterator::new(HerbrandBaseIterator::new(&funcs));
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
        let mut iter = ConstantIterator::new(HerbrandBaseIterator::new(&rules));
        assert_eq!(iter.next(), Some(make_ident("foo")));
        assert_eq!(iter.next(), Some(make_ident("bar")));
        assert_eq!(iter.next(), Some(make_ident("quz")));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_herbrand_instantiation_iterator() {
        #[track_caller]
        fn rule_assert(lhs: Option<&Rule>, rhs: Option<&Rule>) {
            // let slhs: String = match lhs {
            //     Some(lhs) => format!("   lhs > '{lhs}'"),
            //     None => "   lhs !".into(),
            // };
            // let srhs: String = match rhs {
            //     Some(rhs) => format!("   rhs > '{rhs}'"),
            //     None => "   rhs !".into(),
            // };
            // println!("Comparing\n{slhs}\n{srhs}\n");
            if lhs != rhs {
                let slhs: String = match lhs {
                    Some(lhs) => format!("   lhs > '{lhs}'"),
                    None => "   lhs !".into(),
                };
                let srhs: String = match rhs {
                    Some(rhs) => format!("   rhs > '{rhs}'"),
                    None => "   rhs !".into(),
                };
                panic!("Rules are not as expected\n{slhs}\n{srhs}\n");
            }
        }


        // Check empty specs
        let empty: Spec = datalog! { #![crate] };
        let mut iter = HerbrandInstantiationIterator::new(&empty, &empty.herbrand_base().collect());
        rule_assert(iter.next(), None);
        rule_assert(iter.next(), None);
        rule_assert(iter.next(), None);

        // Check with a few atoms
        let cons: Spec = datalog! {
            #![crate]

            foo. bar. bar. baz.
        };
        let mut iter = HerbrandInstantiationIterator::new(&cons, &cons.herbrand_base().collect());
        rule_assert(iter.next(), Some(&datalog! { #![crate] foo. }.rules[0]));
        rule_assert(iter.next(), Some(&datalog! { #![crate] bar. }.rules[0]));
        rule_assert(iter.next(), Some(&datalog! { #![crate] bar. }.rules[0]));
        rule_assert(iter.next(), Some(&datalog! { #![crate] baz. }.rules[0]));
        rule_assert(iter.next(), None);

        // Check with functions
        let funcs: Spec = datalog! {
            #![crate]

            foo(bar). bar(baz, quz). baz(quz).
        };
        let mut iter = HerbrandInstantiationIterator::new(&funcs, &funcs.herbrand_base().collect());
        rule_assert(iter.next(), Some(&datalog! { #![crate] foo(bar). }.rules[0]));
        rule_assert(iter.next(), Some(&datalog! { #![crate] bar(baz, quz). }.rules[0]));
        rule_assert(iter.next(), Some(&datalog! { #![crate] baz(quz). }.rules[0]));
        rule_assert(iter.next(), None);

        // Check with rules
        let rules: Spec = datalog! {
            #![crate]

            foo. bar.
            baz(X) :- quz.
        };
        let mut iter = HerbrandInstantiationIterator::new(&rules, &rules.herbrand_base().collect());
        rule_assert(iter.next(), Some(&datalog! { #![crate] foo. }.rules[0]));
        rule_assert(iter.next(), Some(&datalog! { #![crate] bar. }.rules[0]));
        rule_assert(iter.next(), Some(&datalog! { #![crate] baz(foo) :- quz. }.rules[0]));
        rule_assert(iter.next(), Some(&datalog! { #![crate] baz(bar) :- quz. }.rules[0]));
        rule_assert(iter.next(), Some(&datalog! { #![crate] baz(quz) :- quz. }.rules[0]));
        rule_assert(iter.next(), None);

        // Check with rules, where we do grounded variables _after_ normal ones
        let rules: Spec = datalog! {
            #![crate]

            baz(X) :- quz.
            foo. bar.
        };
        let mut iter = HerbrandInstantiationIterator::new(&rules, &rules.herbrand_base().collect());
        rule_assert(iter.next(), Some(&datalog! { #![crate] baz(quz) :- quz. }.rules[0]));
        rule_assert(iter.next(), Some(&datalog! { #![crate] baz(foo) :- quz. }.rules[0]));
        rule_assert(iter.next(), Some(&datalog! { #![crate] baz(bar) :- quz. }.rules[0]));
        rule_assert(iter.next(), Some(&datalog! { #![crate] foo. }.rules[0]));
        rule_assert(iter.next(), Some(&datalog! { #![crate] bar. }.rules[0]));
        rule_assert(iter.next(), None);

        // Longer rules
        let multi_rules: Spec = datalog! {
            #![crate]

            foo. bar. baz(foo, bar).
            quz(X, Y) :- baz(X, Y).
        };
        let mut iter = HerbrandInstantiationIterator::new(&multi_rules, &multi_rules.herbrand_base().collect());
        rule_assert(iter.next(), Some(&datalog! { #![crate] foo. }.rules[0]));
        rule_assert(iter.next(), Some(&datalog! { #![crate] bar. }.rules[0]));
        rule_assert(iter.next(), Some(&datalog! { #![crate] baz(foo, bar). }.rules[0]));
        rule_assert(iter.next(), Some(&datalog! { #![crate] quz(foo, foo) :- baz(foo, foo). }.rules[0]));
        rule_assert(iter.next(), Some(&datalog! { #![crate] quz(foo, bar) :- baz(foo, bar). }.rules[0]));
        rule_assert(iter.next(), Some(&datalog! { #![crate] quz(bar, foo) :- baz(bar, foo). }.rules[0]));
        rule_assert(iter.next(), Some(&datalog! { #![crate] quz(bar, bar) :- baz(bar, bar). }.rules[0]));
        rule_assert(iter.next(), None);

        // Longer rules
        let multi_rules: Spec = datalog! {
            #![crate]

            foo. bar. baz(foo, bar).
            quz(X, Y, Z) :- baz(X), baz(bar), quz(Z).
        };
        let hbase: IndexSet<Ident> = HerbrandBaseIterator::new(&multi_rules).constants().collect();
        let mut iter = HerbrandInstantiationIterator::new(&multi_rules, &hbase);
        rule_assert(iter.next(), Some(&datalog! { #![crate] foo. }.rules[0]));
        rule_assert(iter.next(), Some(&datalog! { #![crate] bar. }.rules[0]));
        rule_assert(iter.next(), Some(&datalog! { #![crate] baz(foo, bar). }.rules[0]));
        rule_assert(iter.next(), Some(&datalog! { #![crate] quz(foo, foo, foo) :- baz(foo), baz(bar), quz(foo). }.rules[0]));
        rule_assert(iter.next(), Some(&datalog! { #![crate] quz(foo, foo, bar) :- baz(foo), baz(bar), quz(bar). }.rules[0]));
        rule_assert(iter.next(), Some(&datalog! { #![crate] quz(foo, bar, foo) :- baz(foo), baz(bar), quz(foo). }.rules[0]));
        rule_assert(iter.next(), Some(&datalog! { #![crate] quz(foo, bar, bar) :- baz(foo), baz(bar), quz(bar). }.rules[0]));
        rule_assert(iter.next(), Some(&datalog! { #![crate] quz(bar, foo, foo) :- baz(bar), baz(bar), quz(foo). }.rules[0]));
        rule_assert(iter.next(), Some(&datalog! { #![crate] quz(bar, foo, bar) :- baz(bar), baz(bar), quz(bar). }.rules[0]));
        rule_assert(iter.next(), Some(&datalog! { #![crate] quz(bar, bar, foo) :- baz(bar), baz(bar), quz(foo). }.rules[0]));
        rule_assert(iter.next(), Some(&datalog! { #![crate] quz(bar, bar, bar) :- baz(bar), baz(bar), quz(bar). }.rules[0]));
        rule_assert(iter.next(), None);

        // Longer rules
        let multi_rules: Spec = datalog! {
            #![crate]

            foo. bar. baz. baz(foo, bar).
            quz(X, Y) :- baz(X, Y).
        };
        let hbase: IndexSet<Cow<Atom>> = HerbrandBaseIterator::new(&multi_rules).constants().collect();
        let mut iter = HerbrandInstantiationIterator::new(&multi_rules, &hbase);
        rule_assert(iter.next(), Some(&datalog! { #![crate] foo. }.rules[0]));
        rule_assert(iter.next(), Some(&datalog! { #![crate] bar. }.rules[0]));
        rule_assert(iter.next(), Some(&datalog! { #![crate] baz. }.rules[0]));
        rule_assert(iter.next(), Some(&datalog! { #![crate] baz(foo, bar). }.rules[0]));
        rule_assert(iter.next(), Some(&datalog! { #![crate] quz(foo, foo) :- baz(foo, foo). }.rules[0]));
        rule_assert(iter.next(), Some(&datalog! { #![crate] quz(foo, bar) :- baz(foo, bar). }.rules[0]));
        rule_assert(iter.next(), Some(&datalog! { #![crate] quz(foo, baz) :- baz(foo, baz). }.rules[0]));
        rule_assert(iter.next(), Some(&datalog! { #![crate] quz(bar, foo) :- baz(bar, foo). }.rules[0]));
        rule_assert(iter.next(), Some(&datalog! { #![crate] quz(bar, bar) :- baz(bar, bar). }.rules[0]));
        rule_assert(iter.next(), Some(&datalog! { #![crate] quz(bar, baz) :- baz(bar, baz). }.rules[0]));
        rule_assert(iter.next(), Some(&datalog! { #![crate] quz(baz, foo) :- baz(baz, foo). }.rules[0]));
        rule_assert(iter.next(), Some(&datalog! { #![crate] quz(baz, bar) :- baz(baz, bar). }.rules[0]));
        rule_assert(iter.next(), Some(&datalog! { #![crate] quz(baz, baz) :- baz(baz, baz). }.rules[0]));
        rule_assert(iter.next(), None);
    }
}





/***** HELPER FUNCTIONS *****/
/// Generates a new set of iterators for the given [`Rule`].
///
/// # Arguments
/// - `hbase`: The Herbrand base to spawn new iterators with.
/// - `rule`: A [`Rule`] to search for variables and such.
/// - `vars`: The variables that we will find in this rule.
/// - `iters`: A new set of iterators to spawn.
fn refresh_iters<'h>(
    hbase: &'h IndexSet<Ident>,
    rule: &'_ Rule,
    vars: &'_ mut IndexMap<Ident, RepeatIterator<indexmap::set::Iter<'h, Ident>>>,
) -> Option<Rule> {
    /// Generates a new iterator for the `i`th variable.
    ///
    /// # Arguments
    /// - `hbase`: The Herbrand base that we eventually iterator over.
    /// - `n_vars`: The total number of variables we're quantifying over.
    /// - `i`: The i'the variable to generate.
    fn create_repeat_iter<'h>(hbase: &'h IndexSet<Ident>, n_vars: usize, i: usize) -> RepeatIterator<indexmap::set::Iter<'h, Ident>> {
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
        RepeatIterator::new(hbase.iter(), hbase.len().pow((n_vars - 1 - i) as u32), hbase.len().pow(i as u32))
    }


    // Find the (unique!) variables in the rule and decide if we're cloning or borrowing the rule
    vars.clear();
    for cons in rule.consequences.values() {
        for arg in cons.args.iter().map(|a| a.args.values()).flatten() {
            if let AtomArg::Var(var) = arg {
                // Spawn the variable, but do not initialize the iterator yet (we don't know the total number of variables)
                vars.insert(*var, RepeatIterator::empty(hbase.iter()));
            }
        }
    }
    for ante in rule.tail.iter().map(|t| t.antecedents.values()).flatten() {
        for arg in ante.atom().args.iter().map(|a| a.args.values()).flatten() {
            if let AtomArg::Var(var) = arg {
                // Spawn the variable, but do not initialize the iterator yet (we don't know the total number of variables)
                vars.insert(*var, RepeatIterator::empty(hbase.iter()));
            }
        }
    }

    // We defer initializing the iterators until now, because we need to know how many variables there are
    let vars_len: usize = vars.len();
    for (i, iter) in vars.values_mut().enumerate() {
        *iter = create_repeat_iter(hbase, vars_len, i);
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
fn repopulate_rule(rule: &Rule, vars: &IndexMap<Ident, RepeatIterator<indexmap::set::Iter<Ident>>>, values: &[Ident], gen_rule: &mut Rule) {
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
fn get_next_mapping(iters: &mut IndexMap<Ident, RepeatIterator<indexmap::set::Iter<Ident>>>, assign: &mut Vec<Ident>) -> bool {
    assign.clear();
    assign.reserve(iters.len());
    for iter in iters.values_mut() {
        match iter.next() {
            Some(next) => assign.push(*next),
            None => return false,
        }
    }
    true
}





/***** AUXILLARY *****/
/// Allows the inline `.repeat_m_n()` to be used for the [`RepeatIterator`].
pub trait RepeatMN: Clone + Iterator {
    /// Returns the same iterator but repeated as a whole (outer, `M`) and with every element repeater (inner, `N`).
    ///
    /// # Arguments
    /// - `m`: The number of times the outer iterator (including element repeats!) is repeated.
    /// - `n`: The number of times the inner elements are repeated.
    ///
    /// # Returns
    /// A [`RepeatIterator`] wrapping Self to filter out non-constant (i.e., arity > 0) atoms.
    fn repeat_m_n(self, m: usize, n: usize) -> RepeatIterator<Self>;
}
impl<'s, T> RepeatMN for T
where
    T: Clone + Iterator,
    T::Item: Clone,
{
    fn repeat_m_n(self, m: usize, n: usize) -> RepeatIterator<Self> { RepeatIterator::new(self, n, m) }
}

/// Allows the inline `.constants()` to be used for the [`ConstantIterator`].
pub trait Constants {
    /// Returns the same iterator, but with any constants filtered out.
    ///
    /// # Returns
    /// A [`ConstantIterator`] wrapping Self to filter out non-constant (i.e., arity > 0) atoms.
    fn constants(self) -> ConstantIterator<Self>;
}
impl<'s, T: Iterator<Item = Cow<'s, Atom>>> Constants for T {
    #[inline]
    fn constants(self) -> ConstantIterator<Self> { ConstantIterator::new(self) }
}

/// Allows the inline `.herbrand_base()` to be used for a [`Spec`].
pub trait HerbrandBase {
    /// Returns an iterator over the Herbrand base of the specification.
    ///
    /// # Returns
    /// A [`HerbrandBaseIterator`] that does what it says on the tin.
    fn herbrand_base(&self) -> HerbrandBaseIterator;
}
impl HerbrandBase for Spec {
    #[inline]
    fn herbrand_base(&self) -> HerbrandBaseIterator { HerbrandBaseIterator::new(self) }
}

/// Allows the inline `.herbrand_instantiation()` to be used for a [`Spec`].
pub trait HerbrandInstantiation {
    /// Returns an iterator over the Herbrand instantiation of the specification.
    ///
    /// # Arguments
    /// - `hbase`: Some (already computed) Herbrand Base to iterate over.
    ///
    /// # Returns
    /// A [`HerbrandInstantiationIterator`] that does what it says on the tin.
    fn herbrand_instantiation<'s, 'h>(&'s self, hbase: &'h IndexSet<Ident>) -> HerbrandInstantiationIterator<'h, 's>;
}
impl HerbrandInstantiation for Spec {
    #[inline]
    fn herbrand_instantiation<'s, 'h>(&'s self, hbase: &'h IndexSet<Ident>) -> HerbrandInstantiationIterator<'h, 's> {
        HerbrandInstantiationIterator::new(self, hbase)
    }
}





/***** LIBRARY *****/
/// Given an iterator, repeats elements in two directions:
/// - In the "inner" direction, every element is repeated N times; and
/// - In the "outer" direction, the whole iterator (including "inner"-repeats) is repeated M times.
#[derive(Clone, Debug)]
pub struct RepeatIterator<I: Clone + Iterator> {
    /// The iterator itself.
    iter: Flatten<Take<Repeat<I>>>,
    /// The next individual element that we repeat.
    next: Option<I::Item>,
    /// The number of times we should repeat the inner element.
    n:    usize,
    /// The number of times we've repeated the inner element.
    i:    usize,
}
impl<I: Clone + Iterator> RepeatIterator<I> {
    /// Constructor for the RepeatIter that creates it.
    ///
    /// # Arguments
    /// - `iter`: The iterator (who's elements) to repeat.
    /// - `inner`: The number of times to repeat each individual element (N).
    /// - `outer`: The number of times to repeat the whole iterator (including `inner` repeats) (M).
    ///
    /// # Returns
    /// A new RepeatIter instance.
    #[inline]
    pub fn new(iter: I, inner: usize, outer: usize) -> Self {
        let mut iter = std::iter::repeat(iter).take(outer).flatten();
        let next: Option<I::Item> = iter.next();
        Self { iter, next, n: inner, i: 0 }
    }

    /// An as-cheaply-as-possible constructor that creates it as an empty iterator.
    ///
    /// This is used to have something convenient before initializing the "real" RepeatIter later.
    ///
    /// # Arguments
    /// - `iter`: Some stand-in iterator of compatible type.
    ///
    /// # Returns
    /// A new RepeatIter instance that will always yield [`None`].
    #[inline]
    pub fn empty(iter: I) -> Self { Self { iter: std::iter::repeat(iter).take(0).flatten(), next: None, n: 0, i: 0 } }
}
impl<I: Clone + Iterator> Iterator for RepeatIterator<I>
where
    I: Clone + Iterator,
    I::Item: Clone,
{
    type Item = I::Item;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        // Base case: catch n == 0
        if self.n == 0 {
            return None;
        }

        // See if we need to repeat
        self.i += 1;
        if self.i < self.n {
            // Return a clone
            self.next.clone()
        } else if self.i == self.n {
            // Get the one to return
            let next: Option<I::Item> = self.next.take();

            // Get the next one
            self.i = 0;
            self.next = self.iter.next();

            // Return
            next
        } else {
            // Out-of-bounds; n is probably 0
            None
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        match self.iter.size_hint() {
            (min, Some(max)) => (min * self.n, Some(max * self.n)),
            (min, None) => (min * self.n, None),
        }
    }
}

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
    type Item = Cow<'s, Atom>;

    fn next(&mut self) -> Option<Self::Item> {
        // Go through the iterators one-by-one
        self.args
            .next()
            .map(|a| Cow::Owned(Atom { ident: a, args: None }))
            .or_else(|| {
                // If we got here, that means there's no more arguments for the parent atom; get the next consequent
                self.cons
                    .next()
                    .map(|c| {
                        // Prep the arguments for continued iteration
                        if let Some(args) = &c.args {
                            self.args = Box::new(args.args.values().filter_map(|a| if let AtomArg::Atom(a) = a { Some(*a) } else { None }));
                        }
                        // Return the consequent itself
                        Some(Cow::Borrowed(c))
                    })
                    .flatten()
            })
            .or_else(|| {
                // If we got here, that means there's no more arguments for the parent atom _or_ consequences; get the next antecedent
                self.ants
                    .next()
                    .map(|a| {
                        // Prep the arguments for continued iteration
                        if let Some(args) = &a.atom().args {
                            // Not gonna return this one, not a constant. Instead, try the args again
                            self.args = Box::new(args.args.values().filter_map(|a| if let AtomArg::Atom(a) = a { Some(*a) } else { None }));
                        }
                        // Return the antecedent itself
                        Some(Cow::Borrowed(a.atom()))
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

/// Given an iterator over [`Atom`]s, only returns constants (i.e., atoms with arity 0) as [`Ident`]s.
pub struct ConstantIterator<I: ?Sized> {
    /// Some iterator over Atoms.
    iter: I,
}
impl<I> ConstantIterator<I> {
    /// Constructor for the ConstantIterator.
    ///
    /// # Arguments
    /// - `iter`: Some iterator to filter all non-constants out of.
    ///
    /// # Returns
    /// A new ConstantIterator.
    #[inline]
    pub fn new(iter: I) -> Self { Self { iter } }
}
impl<'s, I: Iterator<Item = Cow<'s, Atom>>> Iterator for ConstantIterator<I> {
    type Item = Cow<'s, Atom>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(next) = self.iter.next() {
            // Check if it's a constant
            if next.args.is_some() {
                continue;
            }

            // Return the identifier
            return Some(next);
        }

        // Out of stuff
        None
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) { (0, self.iter.size_hint().1) }
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
    vars:   IndexMap<Ident, RepeatIterator<indexmap::set::Iter<'h, Ident>>>,
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
            return Self { rules: spec.rules.iter(), hbase, rule: None, vars: IndexMap::new(), assign: vec![] };
        }

        // Find the number of variables in the rule and generate the Herbrand Base iterators for that.
        let mut rules = spec.rules.iter();
        match rules.next() {
            Some(rule) => {
                // Find the (unique!) variables in the rule & generate iterators accordingly
                let mut vars: IndexMap<_, _> = IndexMap::new();
                let gen_rule: Option<Rule> = refresh_iters(hbase, rule, &mut vars);

                // OK done
                Self { rules, hbase, rule: Some((rule, gen_rule)), vars, assign: Vec::new() }
            },
            None => {
                // Nothing to do anyway
                Self { rules, hbase, rule: None, vars: IndexMap::new(), assign: vec![] }
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
                self.rule = Some((new_rule, refresh_iters(self.hbase, new_rule, &mut self.vars)));

                // OK, old rule here we go
                Some(rule)
            },
            Some((rule, Some(mut gen_rule))) => {
                // It's for generate rules; so get the next variable mapping
                if !get_next_mapping(&mut self.vars, &mut self.assign) {
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
                    self.rule = Some((new_rule, refresh_iters(self.hbase, new_rule, &mut self.vars)));
                    // NOTE: Without recursion, we will fall through to doing a repopulate case with a grounded rule in case the `new_rule` is grounded
                    return self.next();
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
