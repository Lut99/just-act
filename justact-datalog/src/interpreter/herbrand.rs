//  HERBRAND.rs
//    by Lut99
//
//  Created:
//    21 Mar 2024, 10:55:27
//  Last edited:
//    25 Mar 2024, 17:18:56
//  Auto updated?
//    Yes
//
//  Description:
//!   Implements iterators for the Herbrand instantiation of a program.
//

use std::fmt::Debug;
use std::iter::{Cloned, Flatten, Repeat, Take};

use ast_toolkit_span::Span;
use indexmap::{IndexMap, IndexSet};

use crate::ast::{Atom, AtomArg, Ident, Literal, Rule, Spec};
use crate::log::{debug, trace};


/***** TESTS *****/
#[cfg(all(test, feature = "derive"))]
mod tests {
    use ast_toolkit_punctuated::Punctuated;
    use ast_toolkit_span::Span;
    use justact_datalog_derive::datalog;

    use super::*;
    use crate::ast::{AtomArgs, Comma, Parens};


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
    fn test_herbrand_const_iter() {
        #[cfg(feature = "log")]
        setup_logger();

        // Check empty specs
        let empty: Spec = datalog! { #![crate] };
        let mut iter = HerbrandConstIter::new(&empty);
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);

        // Check with a few atoms
        let cons: Spec = datalog! {
            #![crate]

            foo. bar. bar. baz.
        };
        let mut iter = HerbrandConstIter::new(&cons);
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
        let mut iter = HerbrandConstIter::new(&funcs);
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
        let mut iter = HerbrandConstIter::new(&rules);
        assert_eq!(iter.next(), Some(make_ident("foo")));
        assert_eq!(iter.next(), Some(make_ident("bar")));
        assert_eq!(iter.next(), Some(make_ident("quz")));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_herbrand_base_iter() {
        #[cfg(feature = "log")]
        setup_logger();

        // Check empty specs
        let empty: Spec = datalog! { #![crate] };
        let consts: IndexSet<Ident> = empty.constants().collect();
        let mut iter = HerbrandBaseIter::new(&empty, &consts);
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);

        // Check with a few atoms
        let cons: Spec = datalog! {
            #![crate]

            foo. bar. bar. baz.
        };
        let consts: IndexSet<Ident> = cons.constants().collect();
        let mut iter = HerbrandBaseIter::new(&cons, &consts);
        assert_eq!(iter.next(), Some(make_atom("foo", None)));
        assert_eq!(iter.next(), Some(make_atom("bar", None)));
        assert_eq!(iter.next(), Some(make_atom("baz", None)));
        assert_eq!(iter.next(), None);

        // Check with functions
        let funcs: Spec = datalog! {
            #![crate]

            foo(bar). bar(baz, quz). baz(quz).
        };
        let consts: IndexSet<Ident> = funcs.constants().collect();
        let mut iter = HerbrandBaseIter::new(&funcs, &consts);
        assert_eq!(iter.next(), Some(make_atom("bar", [])));
        assert_eq!(iter.next(), Some(make_atom("baz", [])));
        assert_eq!(iter.next(), Some(make_atom("quz", [])));
        assert_eq!(iter.next(), Some(make_atom("foo", ["bar"])));
        assert_eq!(iter.next(), Some(make_atom("foo", ["baz"])));
        assert_eq!(iter.next(), Some(make_atom("foo", ["quz"])));
        assert_eq!(iter.next(), Some(make_atom("bar", ["bar", "bar"])));
        assert_eq!(iter.next(), Some(make_atom("bar", ["bar", "baz"])));
        assert_eq!(iter.next(), Some(make_atom("bar", ["bar", "quz"])));
        assert_eq!(iter.next(), Some(make_atom("bar", ["baz", "bar"])));
        assert_eq!(iter.next(), Some(make_atom("bar", ["baz", "baz"])));
        assert_eq!(iter.next(), Some(make_atom("bar", ["baz", "quz"])));
        assert_eq!(iter.next(), Some(make_atom("bar", ["quz", "bar"])));
        assert_eq!(iter.next(), Some(make_atom("bar", ["quz", "baz"])));
        assert_eq!(iter.next(), Some(make_atom("bar", ["quz", "quz"])));
        assert_eq!(iter.next(), Some(make_atom("baz", ["bar"])));
        assert_eq!(iter.next(), Some(make_atom("baz", ["baz"])));
        assert_eq!(iter.next(), Some(make_atom("baz", ["quz"])));
        assert_eq!(iter.next(), None);

        // Check with rules
        let rules: Spec = datalog! {
            #![crate]

            foo. bar.
            baz(X) :- quz.
        };
        let consts: IndexSet<Ident> = rules.constants().collect();
        let mut iter = HerbrandBaseIter::new(&rules, &consts);
        assert_eq!(iter.next(), Some(make_atom("foo", None)));
        assert_eq!(iter.next(), Some(make_atom("bar", None)));
        assert_eq!(iter.next(), Some(make_atom("quz", None)));
        assert_eq!(iter.next(), Some(make_atom("baz", Some("foo"))));
        assert_eq!(iter.next(), Some(make_atom("baz", Some("bar"))));
        assert_eq!(iter.next(), Some(make_atom("baz", Some("quz"))));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_herbrand_rule_inst_iter() {
        #[cfg(feature = "log")]
        setup_logger();

        // Check with a few atoms
        let cons: Spec = datalog! {
            #![crate]

            foo. bar. bar. baz.
        };
        let consts: IndexSet<Ident> = cons.constants().collect();
        let rules: Vec<Rule> = cons.rules.iter().flat_map(|r| HerbrandRuleInstIter::new(&consts, r)).collect();
        assert_eq!(rules, vec![
            datalog! { #![crate] foo. }.rules.swap_remove(0),
            datalog! { #![crate] bar. }.rules.swap_remove(0),
            datalog! { #![crate] bar. }.rules.swap_remove(0),
            datalog! { #![crate] baz. }.rules.swap_remove(0),
        ]);

        // Check with functions
        let funcs: Spec = datalog! {
            #![crate]

            foo(bar). bar(baz, quz). baz(quz).
        };
        let consts: IndexSet<Ident> = funcs.constants().collect();
        let rules: Vec<Rule> = funcs.rules.iter().map(|r| HerbrandRuleInstIter::new(&consts, r)).flatten().collect();
        assert_eq!(rules, vec![
            datalog! { #![crate] foo(bar). }.rules.swap_remove(0),
            datalog! { #![crate] bar(baz, quz). }.rules.swap_remove(0),
            datalog! { #![crate] baz(quz). }.rules.swap_remove(0),
        ]);

        // Check with rules
        let rules: Spec = datalog! {
            #![crate]

            foo. bar.
            baz(X) :- quz.
        };
        let consts: IndexSet<Ident> = rules.constants().collect();
        let rules: Vec<Rule> = rules.rules.iter().map(|r| HerbrandRuleInstIter::new(&consts, r)).flatten().collect();
        assert_eq!(rules, vec![
            datalog! { #![crate] foo. }.rules.swap_remove(0),
            datalog! { #![crate] bar. }.rules.swap_remove(0),
            datalog! { #![crate] baz(foo) :- quz. }.rules.swap_remove(0),
            datalog! { #![crate] baz(bar) :- quz. }.rules.swap_remove(0),
            datalog! { #![crate] baz(quz) :- quz. }.rules.swap_remove(0),
        ]);
    }

    #[test]
    fn test_herbrand_inst_iter() {
        #[cfg(feature = "log")]
        setup_logger();

        // Check empty specs
        let empty: Spec = datalog! { #![crate] };
        let consts: IndexSet<Ident> = empty.constants().collect();
        let mut iter = HerbrandInstIter::new(&empty, &consts);
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);
        assert_eq!(iter.next(), None);

        // Check with a few atoms
        let cons: Spec = datalog! {
            #![crate]

            foo. bar. bar. baz.
        };
        let consts: IndexSet<Ident> = cons.constants().collect();
        let mut iter = HerbrandInstIter::new(&cons, &consts);
        assert_eq!(iter.next(), Some(datalog! { #![crate] foo. }.rules.swap_remove(0)));
        assert_eq!(iter.next(), Some(datalog! { #![crate] bar. }.rules.swap_remove(0)));
        assert_eq!(iter.next(), Some(datalog! { #![crate] bar. }.rules.swap_remove(0)));
        assert_eq!(iter.next(), Some(datalog! { #![crate] baz. }.rules.swap_remove(0)));
        assert_eq!(iter.next(), None);

        // Check with functions
        let funcs: Spec = datalog! {
            #![crate]

            foo(bar). bar(baz, quz). baz(quz).
        };
        let consts: IndexSet<Ident> = funcs.constants().collect();
        let mut iter = HerbrandInstIter::new(&funcs, &consts);
        assert_eq!(iter.next(), Some(datalog! { #![crate] foo(bar). }.rules.swap_remove(0)));
        assert_eq!(iter.next(), Some(datalog! { #![crate] bar(baz, quz). }.rules.swap_remove(0)));
        assert_eq!(iter.next(), Some(datalog! { #![crate] baz(quz). }.rules.swap_remove(0)));
        assert_eq!(iter.next(), None);

        // Check with rules
        let rules: Spec = datalog! {
            #![crate]

            foo. bar.
            baz(X) :- quz.
        };
        let consts: IndexSet<Ident> = rules.constants().collect();
        let mut iter = HerbrandInstIter::new(&rules, &consts);
        assert_eq!(iter.next(), Some(datalog! { #![crate] foo. }.rules.swap_remove(0)));
        assert_eq!(iter.next(), Some(datalog! { #![crate] bar. }.rules.swap_remove(0)));
        assert_eq!(iter.next(), Some(datalog! { #![crate] baz(foo) :- quz. }.rules.swap_remove(0)));
        assert_eq!(iter.next(), Some(datalog! { #![crate] baz(bar) :- quz. }.rules.swap_remove(0)));
        assert_eq!(iter.next(), Some(datalog! { #![crate] baz(quz) :- quz. }.rules.swap_remove(0)));
        assert_eq!(iter.next(), None);
    }

    // #[test]
    // fn test_herbrand_instantiation_iterator() {
    //     #[track_caller]
    //     fn rule_assert(lhs: Option<&Rule>, rhs: Option<&Rule>) {
    //         // let slhs: String = match lhs {
    //         //     Some(lhs) => format!("   lhs > '{lhs}'"),
    //         //     None => "   lhs !".into(),
    //         // };
    //         // let srhs: String = match rhs {
    //         //     Some(rhs) => format!("   rhs > '{rhs}'"),
    //         //     None => "   rhs !".into(),
    //         // };
    //         // println!("Comparing\n{slhs}\n{srhs}\n");
    //         if lhs != rhs {
    //             let slhs: String = match lhs {
    //                 Some(lhs) => format!("   lhs > '{lhs}'"),
    //                 None => "   lhs !".into(),
    //             };
    //             let srhs: String = match rhs {
    //                 Some(rhs) => format!("   rhs > '{rhs}'"),
    //                 None => "   rhs !".into(),
    //             };
    //             panic!("Rules are not as expected\n{slhs}\n{srhs}\n");
    //         }
    //     }

    //     // Check empty specs
    //     let empty: Spec = datalog! { #![crate] };
    //     let hbase: IndexSet<Cow<Atom>> = empty.herbrand_base().collect();
    //     let mut iter = HerbrandInstantiationIterator::new(&empty, &hbase);
    //     rule_assert(iter.next(), None);
    //     rule_assert(iter.next(), None);
    //     rule_assert(iter.next(), None);

    //     // Check with a few atoms
    //     let cons: Spec = datalog! {
    //         #![crate]

    //         foo. bar. bar. baz.
    //     };
    //     let hbase: IndexSet<Cow<Atom>> = cons.herbrand_base().collect();
    //     let mut iter = HerbrandInstantiationIterator::new(&cons, &hbase);
    //     rule_assert(iter.next(), Some(&datalog! { #![crate] foo. }.rules[0]));
    //     rule_assert(iter.next(), Some(&datalog! { #![crate] bar. }.rules[0]));
    //     rule_assert(iter.next(), Some(&datalog! { #![crate] bar. }.rules[0]));
    //     rule_assert(iter.next(), Some(&datalog! { #![crate] baz. }.rules[0]));
    //     rule_assert(iter.next(), None);

    //     // Check with functions
    //     let funcs: Spec = datalog! {
    //         #![crate]

    //         foo(bar). bar(baz, quz). baz(quz).
    //     };
    //     let hbase: IndexSet<Cow<Atom>> = funcs.herbrand_base().collect();
    //     let mut iter = HerbrandInstantiationIterator::new(&funcs, &hbase);
    //     rule_assert(iter.next(), Some(&datalog! { #![crate] foo(bar). }.rules[0]));
    //     rule_assert(iter.next(), Some(&datalog! { #![crate] bar(baz, quz). }.rules[0]));
    //     rule_assert(iter.next(), Some(&datalog! { #![crate] baz(quz). }.rules[0]));
    //     rule_assert(iter.next(), None);

    //     // Check with rules
    //     let rules: Spec = datalog! {
    //         #![crate]

    //         foo. bar.
    //         baz(X) :- quz.
    //     };
    //     let hbase: IndexSet<Cow<Atom>> = rules.herbrand_base().collect();
    //     let mut iter = HerbrandInstantiationIterator::new(&rules, &hbase);
    //     rule_assert(iter.next(), Some(&datalog! { #![crate] foo. }.rules[0]));
    //     rule_assert(iter.next(), Some(&datalog! { #![crate] bar. }.rules[0]));
    //     rule_assert(iter.next(), Some(&datalog! { #![crate] baz(foo) :- quz. }.rules[0]));
    //     rule_assert(iter.next(), Some(&datalog! { #![crate] baz(bar) :- quz. }.rules[0]));
    //     rule_assert(iter.next(), Some(&datalog! { #![crate] baz(quz) :- quz. }.rules[0]));
    //     rule_assert(iter.next(), None);

    //     // Check with rules, where we do grounded variables _after_ normal ones
    //     let rules: Spec = datalog! {
    //         #![crate]

    //         baz(X) :- quz.
    //         foo. bar.
    //     };
    //     let hbase: IndexSet<Cow<Atom>> = rules.herbrand_base().collect();
    //     let mut iter = HerbrandInstantiationIterator::new(&rules, &hbase);
    //     rule_assert(iter.next(), Some(&datalog! { #![crate] baz(quz) :- quz. }.rules[0]));
    //     rule_assert(iter.next(), Some(&datalog! { #![crate] baz(foo) :- quz. }.rules[0]));
    //     rule_assert(iter.next(), Some(&datalog! { #![crate] baz(bar) :- quz. }.rules[0]));
    //     rule_assert(iter.next(), Some(&datalog! { #![crate] foo. }.rules[0]));
    //     rule_assert(iter.next(), Some(&datalog! { #![crate] bar. }.rules[0]));
    //     rule_assert(iter.next(), None);

    //     // Longer rules
    //     let multi_rules: Spec = datalog! {
    //         #![crate]

    //         foo. bar. baz(foo, bar).
    //         quz(X, Y) :- baz(X, Y).
    //     };
    //     let hbase: IndexSet<Cow<Atom>> = multi_rules.herbrand_base().collect();
    //     let mut iter = HerbrandInstantiationIterator::new(&multi_rules, &hbase);
    //     rule_assert(iter.next(), Some(&datalog! { #![crate] foo. }.rules[0]));
    //     rule_assert(iter.next(), Some(&datalog! { #![crate] bar. }.rules[0]));
    //     rule_assert(iter.next(), Some(&datalog! { #![crate] baz(foo, bar). }.rules[0]));
    //     rule_assert(iter.next(), Some(&datalog! { #![crate] quz(foo, foo) :- baz(foo, foo). }.rules[0]));
    //     rule_assert(iter.next(), Some(&datalog! { #![crate] quz(foo, bar) :- baz(foo, bar). }.rules[0]));
    //     rule_assert(iter.next(), Some(&datalog! { #![crate] quz(bar, foo) :- baz(bar, foo). }.rules[0]));
    //     rule_assert(iter.next(), Some(&datalog! { #![crate] quz(bar, bar) :- baz(bar, bar). }.rules[0]));
    //     rule_assert(iter.next(), None);

    //     // Longer rules
    //     let multi_rules: Spec = datalog! {
    //         #![crate]

    //         foo. bar. baz(foo, bar).
    //         quz(X, Y, Z) :- baz(X), baz(bar), quz(Z).
    //     };
    //     let hbase: IndexSet<Cow<Atom>> = multi_rules.herbrand_base().collect();
    //     let mut iter = HerbrandInstantiationIterator::new(&multi_rules, &hbase);
    //     rule_assert(iter.next(), Some(&datalog! { #![crate] foo. }.rules[0]));
    //     rule_assert(iter.next(), Some(&datalog! { #![crate] bar. }.rules[0]));
    //     rule_assert(iter.next(), Some(&datalog! { #![crate] baz(foo, bar). }.rules[0]));
    //     rule_assert(iter.next(), Some(&datalog! { #![crate] quz(foo, foo, foo) :- baz(foo), baz(bar), quz(foo). }.rules[0]));
    //     rule_assert(iter.next(), Some(&datalog! { #![crate] quz(foo, foo, bar) :- baz(foo), baz(bar), quz(bar). }.rules[0]));
    //     rule_assert(iter.next(), Some(&datalog! { #![crate] quz(foo, bar, foo) :- baz(foo), baz(bar), quz(foo). }.rules[0]));
    //     rule_assert(iter.next(), Some(&datalog! { #![crate] quz(foo, bar, bar) :- baz(foo), baz(bar), quz(bar). }.rules[0]));
    //     rule_assert(iter.next(), Some(&datalog! { #![crate] quz(bar, foo, foo) :- baz(bar), baz(bar), quz(foo). }.rules[0]));
    //     rule_assert(iter.next(), Some(&datalog! { #![crate] quz(bar, foo, bar) :- baz(bar), baz(bar), quz(bar). }.rules[0]));
    //     rule_assert(iter.next(), Some(&datalog! { #![crate] quz(bar, bar, foo) :- baz(bar), baz(bar), quz(foo). }.rules[0]));
    //     rule_assert(iter.next(), Some(&datalog! { #![crate] quz(bar, bar, bar) :- baz(bar), baz(bar), quz(bar). }.rules[0]));
    //     rule_assert(iter.next(), None);

    //     // Longer rules
    //     let multi_rules: Spec = datalog! {
    //         #![crate]

    //         foo. bar. baz. baz(foo, bar).
    //         quz(X, Y) :- baz(X, Y).
    //     };
    //     let hbase: IndexSet<Cow<Atom>> = multi_rules.herbrand_base().collect();
    //     let mut iter = HerbrandInstantiationIterator::new(&multi_rules, &hbase);
    //     rule_assert(iter.next(), Some(&datalog! { #![crate] foo. }.rules[0]));
    //     rule_assert(iter.next(), Some(&datalog! { #![crate] bar. }.rules[0]));
    //     rule_assert(iter.next(), Some(&datalog! { #![crate] baz. }.rules[0]));
    //     rule_assert(iter.next(), Some(&datalog! { #![crate] baz(foo, bar). }.rules[0]));
    //     rule_assert(iter.next(), Some(&datalog! { #![crate] quz(foo, foo) :- baz(foo, foo). }.rules[0]));
    //     rule_assert(iter.next(), Some(&datalog! { #![crate] quz(foo, bar) :- baz(foo, bar). }.rules[0]));
    //     rule_assert(iter.next(), Some(&datalog! { #![crate] quz(foo, baz) :- baz(foo, baz). }.rules[0]));
    //     rule_assert(iter.next(), Some(&datalog! { #![crate] quz(bar, foo) :- baz(bar, foo). }.rules[0]));
    //     rule_assert(iter.next(), Some(&datalog! { #![crate] quz(bar, bar) :- baz(bar, bar). }.rules[0]));
    //     rule_assert(iter.next(), Some(&datalog! { #![crate] quz(bar, baz) :- baz(bar, baz). }.rules[0]));
    //     rule_assert(iter.next(), Some(&datalog! { #![crate] quz(baz, foo) :- baz(baz, foo). }.rules[0]));
    //     rule_assert(iter.next(), Some(&datalog! { #![crate] quz(baz, bar) :- baz(baz, bar). }.rules[0]));
    //     rule_assert(iter.next(), Some(&datalog! { #![crate] quz(baz, baz) :- baz(baz, baz). }.rules[0]));
    //     rule_assert(iter.next(), None);
    // }
}





/***** HELPER FUNCTIONS *****/
// /// Generates a new set of iterators for the given [`Rule`].
// ///
// /// # Arguments
// /// - `hbase`: The Herbrand base to spawn new iterators with.
// /// - `rule`: A [`Rule`] to search for variables and such.
// /// - `vars`: The variables that we will find in this rule.
// /// - `iters`: A new set of iterators to spawn.
// fn refresh_iters<'h, 's>(
//     hbase: &'h IndexSet<Cow<'s, Atom>>,
//     rule: &'_ Rule,
//     vars: &'_ mut IndexMap<Ident, RepeatIterator<ConstantIterator<indexmap::set::Iter<'h, Cow<'s, Atom>>>>>,
// ) -> Option<Rule> {
//     /// Generates a new iterator for the `i`th variable.
//     ///
//     /// # Arguments
//     /// - `hbase`: The Herbrand base that we eventually iterator over.
//     /// - `n_vars`: The total number of variables we're quantifying over.
//     /// - `i`: The i'the variable to generate.
//     fn create_repeat_iter<'h, 's>(
//         hbase: &'h IndexSet<Cow<'s, Atom>>,
//         n_vars: usize,
//         i: usize,
//     ) -> RepeatIterator<ConstantIterator<indexmap::set::Iter<'h, Cow<'s, Atom>>>> {
//         // We scale from essentially doing `111111...333333`, to `111222...222333`, to `123123...123123`
//         //
//         // Some examples:
//         // ```plain
//         // 123, three variables:
//         // 111111111222222222333333333      (outer = 1, inner = 9)
//         // 111222333111222333111222333      (outer = 3, inner = 3)
//         // 123123123123123123123123123      (outer = 9, inner = 1)
//         //
//         // 12, four variables
//         // 1111111122222222                 (outer = 1, inner = 8)
//         // 1111222211112222                 (outer = 2, inner = 4)
//         // 1122112211221122                 (outer = 4, inner = 2)
//         // 1212121212121212                 (outer = 8, inner = 1)
//         //
//         // 1234, two variables
//         // 1111222233334444                 (outer = 1, inner = 4)
//         // 1234123412341234                 (outer = 4, inner = 1)
//         // ```
//         // From this we can observe that the outer grows exponentially over the Herbrand base size, whereas the inner grows inverse exponentially.
//         RepeatIterator::new(hbase.iter().constants(), hbase.len().pow((n_vars - 1 - i) as u32), hbase.len().pow(i as u32))
//     }


//     // Find the (unique!) variables in the rule and decide if we're cloning or borrowing the rule
//     vars.clear();
//     for cons in rule.consequences.values() {
//         for arg in cons.args.iter().map(|a| a.args.values()).flatten() {
//             if let AtomArg::Var(var) = arg {
//                 // Spawn the variable, but do not initialize the iterator yet (we don't know the total number of variables)
//                 vars.insert(*var, RepeatIterator::empty(hbase.iter().constants()));
//             }
//         }
//     }
//     for ante in rule.tail.iter().map(|t| t.antecedents.values()).flatten() {
//         for arg in ante.atom().args.iter().map(|a| a.args.values()).flatten() {
//             if let AtomArg::Var(var) = arg {
//                 // Spawn the variable, but do not initialize the iterator yet (we don't know the total number of variables)
//                 vars.insert(*var, RepeatIterator::empty(hbase.iter().constants()));
//             }
//         }
//     }

//     // We defer initializing the iterators until now, because we need to know how many variables there are
//     let vars_len: usize = vars.len();
//     for (i, iter) in vars.values_mut().enumerate() {
//         *iter = create_repeat_iter(hbase, vars_len, i);
//     }

//     // Define what to return
//     if vars.len() > 0 { Some(rule.clone()) } else { None }
// }

// /// Repopulates the given rule with the given mapping.
// ///
// /// The values are assigned in-order as the variables are encountered. Since this iteration over the rule is deterministic, so should the assignment be.
// ///
// /// # Arguments
// /// - `rule`: The original rule that knows where variables are.
// /// - `vars`: Defines the names of variables. Given as an [`IndexSet`] for speedier search, while the order is important to match with the assignment.
// /// - `values`: The values mapping for the given `vars`.
// /// - `gen_rule`: The rule to repopulate.
// fn repopulate_rule(
//     rule: &Rule,
//     vars: &IndexMap<Ident, RepeatIterator<ConstantIterator<indexmap::set::Iter<Cow<Atom>>>>>,
//     values: &[Ident],
//     gen_rule: &mut Rule,
// ) {
//     for (c, cons) in rule.consequences.values().enumerate() {
//         for (a, arg) in cons.args.iter().map(|a| a.args.values()).flatten().enumerate() {
//             if matches!(arg, AtomArg::Var(_)) {
//                 // Find this variable's index in the mapping
//                 // SAFETY: We can unwrap here because we assume the caller has given us a mapping for this rule.
//                 let idx: usize = vars.get_index_of(arg.ident()).expect("Found variable in rule that was not in mapping");
//                 gen_rule.consequences[c].args.as_mut().unwrap().args[a] = AtomArg::Atom(values[idx]);
//             }
//         }
//     }
//     for (t, ante) in rule.tail.iter().map(|t| t.antecedents.values()).flatten().enumerate() {
//         for (a, arg) in ante.atom().args.iter().map(|a| a.args.values()).flatten().enumerate() {
//             if matches!(arg, AtomArg::Var(_)) {
//                 // Find this variable's index in the mapping
//                 // SAFETY: We can unwrap here because we assume the caller has given us a mapping for this rule.
//                 let idx: usize = vars.get_index_of(arg.ident()).expect("Found variable in rule that was not in mapping");
//                 gen_rule.tail.as_mut().unwrap().antecedents[t].atom_mut().args.as_mut().unwrap().args[a] = AtomArg::Atom(values[idx]);
//             }
//         }
//     }
// }

// /// Find the next mapping given the set of iterators.
// ///
// /// # Arguments
// /// - `rule`: Some rule to use to generate new iterators if it proves necessary.
// /// - `iters`: The iterators to pass. We assume that by some clever usage of [`std::iter::repeat`], any binary-like counting is embedded.
// /// - `assign`: The assignment to populate.
// ///
// /// # Returns
// /// Whether we found a next mapping. If false, this means that we ran out of mappings to generate.
// fn get_next_mapping(iters: &mut IndexMap<Ident, RepeatIterator<ConstantIterator<indexmap::set::Iter<Cow<Atom>>>>>, assign: &mut Vec<Ident>) -> bool {
//     assign.clear();
//     assign.reserve(iters.len());
//     for iter in iters.values_mut() {
//         match iter.next() {
//             Some(next) => assign.push(next),
//             None => return false,
//         }
//     }
//     true
// }





/***** ITERATOR TRAITS *****/
// /// Allows the inline `.repeat_m_n()` to be used for the [`RepeatIterator`].
// pub trait RepeatMN: Clone + Iterator {
//     /// Returns the same iterator but repeated as a whole (outer, `M`) and with every element repeater (inner, `N`).
//     ///
//     /// # Arguments
//     /// - `m`: The number of times the outer iterator (including element repeats!) is repeated.
//     /// - `n`: The number of times the inner elements are repeated.
//     ///
//     /// # Returns
//     /// A [`RepeatIterator`] wrapping Self to filter out non-constant (i.e., arity > 0) atoms.
//     fn repeat_m_n(self, m: usize, n: usize) -> RepeatIterator<Self>;
// }
// impl<'s, T> RepeatMN for T
// where
//     T: Clone + Iterator,
//     T::Item: Clone,
// {
//     fn repeat_m_n(self, m: usize, n: usize) -> RepeatIterator<Self> { RepeatIterator::new(self, n, m) }
// }

/// Allows the inline `.constants()` to be used for the [`ConstantIterator`].
pub trait Constants {
    /// Returns the same iterator, but with any constants filtered out.
    ///
    /// # Returns
    /// A [`ConstantIterator`] wrapping Self to filter out non-constant (i.e., arity > 0) atoms.
    fn constants(&self) -> HerbrandConstIter;
}
impl Constants for Spec {
    #[inline]
    fn constants(&self) -> HerbrandConstIter { HerbrandConstIter::new(self) }
}

/// Allows the inline `.herbrand_base()` to be used for a [`Spec`].
pub trait HerbrandBase {
    /// Returns an iterator over the Herbrand base of the specification.
    ///
    /// # Returns
    /// A [`HerbrandBaseIter`] that does what it says on the tin.
    fn herbrand_base<'s, 'c>(&'s self, consts: &'c IndexSet<Ident>) -> HerbrandBaseIter<'s, 'c>;
}
impl HerbrandBase for Spec {
    #[inline]
    fn herbrand_base<'s, 'c>(&'s self, consts: &'c IndexSet<Ident>) -> HerbrandBaseIter<'s, 'c> { HerbrandBaseIter::new(self, consts) }
}

// /// Allows the inline `.herbrand_instantiation()` to be used for a [`Spec`].
// pub trait HerbrandInstantiation {
//     /// Returns an iterator over the Herbrand instantiation of the specification.
//     ///
//     /// # Arguments
//     /// - `hbase`: Some (already computed) Herbrand Base to iterate over.
//     ///
//     /// # Returns
//     /// A [`HerbrandInstantiationIterator`] that does what it says on the tin.
//     fn herbrand_instantiation<'s, 'h>(&'s self, hbase: &'h IndexSet<Cow<'s, Atom>>) -> HerbrandInstantiationIterator<'h, 's>;
// }
// impl HerbrandInstantiation for Spec {
//     #[inline]
//     fn herbrand_instantiation<'s, 'h>(&'s self, hbase: &'h IndexSet<Cow<'s, Atom>>) -> HerbrandInstantiationIterator<'h, 's> {
//         HerbrandInstantiationIterator::new(self, hbase)
//     }
// }





/***** HELPER ITERATORS *****/
/// Given an iterator, repeats elements in two directions:
/// - In the "inner" direction, every element is repeated N times; and
/// - In the "outer" direction, the whole iterator (including "inner"-repeats) is repeated M times.
#[derive(Clone, Debug)]
struct RepeatIterator<I: Clone + Iterator> {
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
    fn new(iter: I, inner: usize, outer: usize) -> Self {
        let mut iter = std::iter::repeat(iter).take(outer).flatten();
        let next: Option<I::Item> = iter.next();
        Self { iter, next, n: inner, i: 0 }
    }
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



/// Iterates over all combinations of constants for the given atom.
///
/// This version iterates over *\*all\** combinations, including ones that would be illegal given the rule. It is a more fair representation of the HerbrandBase. For a more optimized version, see the [`FairVarAssignIter`].
struct VarAssignIter<'c, 's> {
    /// The set of constants to iterate over.
    consts: &'c IndexSet<Ident>,
    /// Some [`Atom`] that acts as a buffer of what to return.
    atom:   &'s Atom,
    /// The iterators used to produce the next atom. Mapped by variable to deal with unique cases
    iters:  Vec<RepeatIterator<Cloned<indexmap::set::Iter<'c, Ident>>>>,
}
impl<'c, 's> VarAssignIter<'c, 's> {
    /// Static helper function that find new iterators for the given atom.
    ///
    /// # Arguments
    /// - `consts`: A set of constants that is quantified over to assign to variables.
    /// - `atom`: The atom to find iterators for.
    ///
    /// # Returns
    /// A new vector with found iterators.
    fn find_iters(consts: &'c IndexSet<Ident>, atom: &'s Atom, iters: &mut Vec<RepeatIterator<Cloned<indexmap::set::Iter<'c, Ident>>>>) {
        iters.clear();

        // Analyze the atom to find where we need to repeat the same atom, and where we need to populate variables
        let n_vars: usize = atom.args.as_ref().map(|a| a.args.len()).unwrap_or(0);
        for i in 0..n_vars {
            // Apply a classic grow-double strategy to the iterators
            if iters.len() >= iters.capacity() {
                iters.reserve(iters.len() * 2);
            }

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
            iters.push(RepeatIterator::new(consts.iter().cloned(), consts.len().pow((n_vars - 1 - i) as u32), consts.len().pow(i as u32)));
        }
    }
}
impl<'c, 's> VarAssignIter<'c, 's> {
    /// Constructor for the VarAssignIter.
    ///
    /// # Arguments
    /// - `consts`: A set of constants that is quantified over to assign to variables.
    /// - `atom`: The [`Atom`] to iterate over and find new variable assignments.
    ///
    /// # Returns
    /// A new VarAssignIter.
    fn new(consts: &'c IndexSet<Ident>, atom: &'s Atom) -> Self {
        // Find the iterators using the neat builtin function
        let mut iters = Vec::new();
        Self::find_iters(consts, atom, &mut iters);

        // OK, done!
        Self { consts, atom, iters }
    }

    /// Refreshes the VarAssignIter with a new atom to iterate over.
    ///
    /// # Arguments
    /// - `atom`: The new [`Atom`] to iterate over and find new variable assignments.
    fn refresh(&mut self, atom: &'s Atom) {
        self.atom = atom;
        Self::find_iters(self.consts, self.atom, &mut self.iters);
    }
}
impl<'c, 's> Iterator for VarAssignIter<'c, 's> {
    type Item = Atom;

    #[inline]
    fn next(&mut self) -> Option<Atom> {
        let mut atom: Atom = self.atom.clone();
        for (arg, iter) in atom.args.iter_mut().map(|a| a.args.values_mut()).flatten().zip(self.iters.iter_mut()) {
            let assign: Ident = iter.next()?;
            *arg = AtomArg::Atom(assign);
        }
        Some(atom)
    }
}



/// Auxillary struct for the [`FairVarAssignIter`] that represents one of possible iterators.
enum FairAssignIter<'c> {
    /// Represents an atom we don't have to quantify. Always repeats the same value.
    Atom(Take<Repeat<Ident>>),
    /// Represents an atom before we know how many.
    AtomPlaceholder(Ident),
    /// Represents a variable that we _do_ quantify.
    Var(RepeatIterator<Cloned<indexmap::set::Iter<'c, Ident>>>),
    /// Represents a placeholder for a variable, i.e., when we haven't yet found how many variables there are in total.
    VarPlaceholder(Ident),
}
impl<'c> Iterator for FairAssignIter<'c> {
    type Item = Ident;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Atom(a) => a.next(),
            Self::AtomPlaceholder(_) => unreachable!(),
            Self::Var(v) => v.next(),
            Self::VarPlaceholder(_) => unreachable!(),
        }
    }
}





/***** LIBRARY *****/
/// Defines an iterator over only the constants in a _Herbrand base_ of a program.
///
/// The Herbrand base is defined as all constants (i.e., atoms of arity 0) plus all atoms with arity > 0, where variables are substituted for all possible constants.
///
/// For some programs, the Herbrand base is infinite. This is not the case for $Datalog^\neg$, because it doesn't allow nesting and types atoms by arity.
///
/// This specific iterator only iterates over the constants in a program. See the [`HerbrandBaseIter`] for a more comprehensive iterator that also takes non-constant atoms into account.
pub struct HerbrandConstIter<'s> {
    /// The set of rules to iterate over.
    rules: std::slice::Iter<'s, Rule>,
    /// The current antecedents to iterate over.
    ants:  Box<dyn 's + Iterator<Item = &'s Literal>>,
    /// The current consequences to iterate over.
    cons:  Box<dyn 's + Iterator<Item = &'s Atom>>,
    /// Finally, the current arguments to iterate over.
    args:  Box<dyn 's + Iterator<Item = Ident>>,
}
impl<'s> HerbrandConstIter<'s> {
    /// Constructor for a HerbrandConstIter.
    ///
    /// # Arguments
    /// - `spec`: Some [`Spec`] to iterate over.
    ///
    /// # Returns
    /// An HerbrandConstIter that will produce all constants (i.e., arity-0, grounded atoms) in the `spec`.
    pub fn new(spec: &'s Spec) -> Self {
        debug!(
            "Created new HerbrandConstIter\n\nSpec:\n{}\n{}\n{}\n",
            (0..80).map(|_| '-').collect::<String>(),
            spec.rules.iter().map(|r| format!("   {r}")).collect::<Vec<String>>().join("\n"),
            (0..80).map(|_| '-').collect::<String>()
        );

        // Build ourselves with nothing in it yet but a rule
        Self { rules: spec.rules.iter(), ants: Box::new(None.into_iter()), cons: Box::new(None.into_iter()), args: Box::new(None.into_iter()) }
    }
}
impl<'s> Iterator for HerbrandConstIter<'s> {
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
                        // Prep the arguments for continued iteration
                        if let Some(args) = &c.args {
                            self.args = Box::new(args.args.values().filter_map(|a| if let AtomArg::Atom(a) = a { Some(*a) } else { None }));
                            return self.next();
                        }
                        // Return the consequent itself
                        Some(c.ident)
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
                            return self.next();
                        }
                        // Return the antecedent itself
                        Some(a.atom().ident)
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



/// Defines an iterator over the _Herbrand base_ of a program.
///
/// The Herbrand base is defined as all constants (i.e., atoms of arity 0) plus all atoms with arity > 0, where variables are substituted for all possible constants.
///
/// For some programs, the Herbrand base is infinite. This is not the case for $Datalog^\neg$, because it doesn't allow nesting and types atoms by arity.
///
/// This iterator relies on the [`HerbrandConstIter`] iterator to provide the "all possible constants" part.
pub struct HerbrandBaseIter<'s, 'c> {
    /// The set of rules to iterate over.
    rules: std::slice::Iter<'s, Rule>,
    /// The set of constants that we use to quantify.
    consts: &'c IndexSet<Ident>,
    /// The constants providing the first set of constants.
    consts_iter: std::iter::Fuse<indexmap::set::Iter<'c, Ident>>,
    /// The list of consequences for the current rule.
    cons: Box<dyn 's + Iterator<Item = &'s Atom>>,
    /// The list of antecedents for the current rule.
    ants: Box<dyn 's + Iterator<Item = &'s Literal>>,
    /// An assignment of variables for the current atom we're iterating over.
    vars: Option<VarAssignIter<'c, 's>>,
}
impl<'s, 'c> HerbrandBaseIter<'s, 'c> {
    /// Constructor for a HerbrandBaseIter.
    ///
    /// # Arguments
    /// - `spec`: Some [`Spec`] to iterate over.
    /// - `consts`: An [`IndexSet`] of the found, unique constants in the given [`Spec`].
    ///
    /// # Returns
    /// An HerbrandBaseIter that will produce all constants (i.e., arity-0, grounded atoms) in the `spec`.
    pub fn new(spec: &'s Spec, consts: &'c IndexSet<Ident>) -> Self {
        debug!(
            "Created new HerbrandBaseIter\n\nSpec:\n{}\n{}\n{}\n\nConstants:\n{}\n{}\n{}\n",
            (0..80).map(|_| '-').collect::<String>(),
            spec.rules.iter().map(|r| format!("   {r}")).collect::<Vec<String>>().join("\n"),
            (0..80).map(|_| '-').collect::<String>(),
            (0..80).map(|_| '-').collect::<String>(),
            consts.iter().map(|c| format!("   {c}")).collect::<Vec<String>>().join("\n"),
            (0..80).map(|_| '-').collect::<String>()
        );

        // Build ourselves with nothing in it (as long as there are rules, next() takes care of everything)
        Self {
            rules: spec.rules.iter(),
            consts,
            consts_iter: consts.iter().fuse(),
            cons: Box::new(None.into_iter()),
            ants: Box::new(None.into_iter()),
            vars: None,
        }
    }
}
impl<'s, 'c> Iterator for HerbrandBaseIter<'s, 'c> {
    type Item = Atom;

    fn next(&mut self) -> Option<Atom> {
        // Try to drain the constants first
        if let Some(next) = self.consts_iter.next() {
            trace!("Returning constant '{next}'");
            return Some(Atom { ident: *next, args: None });
        }

        // Try to pop the next variable assignment first
        if let Some(assign) = self.vars.as_mut().map(|v| v.next()).flatten() {
            trace!("Returning assigned atom '{assign}'");
            return Some(assign);
        }

        // If there are none, try to get the next as a consequent
        // Note that we search until we find one with arguments
        if let Some(cons) = self.cons.next().filter(|c| c.args.is_some()) {
            trace!("Moved to consequence '{cons}'");

            // Re-spawn the variable list, either as a new list or by refreshing the old one
            match &mut self.vars {
                Some(vars) => vars.refresh(cons),
                None => self.vars = Some(VarAssignIter::new(self.consts, cons)),
            }
            return self.next();
        }

        // Antecedents, at least??
        // Note that we search until we find one with arguments
        if let Some(ante) = self.ants.next().filter(|a| a.atom().args.is_some()) {
            trace!("Moved to antecedent '{ante}'");

            // Re-spawn the variable list, either as a new list or by refreshing the old one
            match &mut self.vars {
                Some(vars) => vars.refresh(ante.atom()),
                None => self.vars = Some(VarAssignIter::new(self.consts, ante.atom())),
            }
            return self.next();
        }

        // Then do the next rule
        let rule: &'s Rule = self.rules.next()?;
        trace!("Moved to rule '{rule}'");
        self.cons = Box::new(rule.consequences.values());
        self.ants = Box::new(rule.tail.iter().map(|t| t.antecedents.values()).flatten());
        self.next()
    }
}



/// Defines an iterator over the _Herbrand instantiation_ of a particular rule.
///
/// The Herbrand instantiation is defined as all rules that are the same as the given rule, but with all substitutions possible of variables with grounded atoms.
///
/// For some programs, the Herbrand instantiation is infinite. This is not the case for $Datalog^\neg$, because it doesn't allow nesting and types atoms by arity. Further, only possible constants have to be quantified due to the impossibility of nesting.
///
/// This iterator relies on the [`HerbrandConstIter`] iterator to provide the "all possible constants" part.
struct HerbrandRuleInstIter<'c, 's> {
    /// The set of constants to iterate over.
    consts: &'c IndexSet<Ident>,
    /// Some [`Rule`] that acts as a buffer of what to return.
    rule:   &'s Rule,
    /// An auxillary buffer used to keep track of which variables we've seen.
    dups:   IndexMap<Ident, Option<RepeatIterator<Cloned<indexmap::set::Iter<'c, Ident>>>>>,
    /// The iterators used to produce the next atom. Mapped by variable to deal with unique cases
    iters:  Vec<FairAssignIter<'c>>,
}
impl<'c, 's> HerbrandRuleInstIter<'c, 's> {
    /// Static helper function that find new iterators for the given rule.
    ///
    /// # Arguments
    /// - `consts`: A set of constants that is quantified over to assign to variables.
    /// - `rule`: The rule to find iterators for.
    ///
    /// # Returns
    /// A new vector with found iterators.
    fn find_iters(
        consts: &'c IndexSet<Ident>,
        rule: &'s Rule,
        dups: &mut IndexMap<Ident, Option<RepeatIterator<Cloned<indexmap::set::Iter<'c, Ident>>>>>,
        iters: &mut Vec<FairAssignIter<'c>>,
    ) {
        dups.clear();
        iters.clear();

        // Analyze the rule to find where we need to repeat the same atom, and where we need to populate variables
        // EDITOR'S NOTE: Looks horrible, but is just a chain of both the consequents and antecedents in the rule.
        for arg in rule.consequences.values().map(|a| a.args.iter().map(|a| a.args.values()).flatten()).flatten().chain(
            rule.tail.iter().map(|t| t.antecedents.values().map(|a| a.atom().args.iter().map(|a| a.args.values()).flatten()).flatten()).flatten(),
        ) {
            // Apply a classic grow-double strategy to the iterators
            if iters.len() >= iters.capacity() {
                iters.reserve(iters.len() * 2);
            }

            // Either push this as an atom iterator (if it is one), or a placeholder for the variable iterator.
            match arg {
                AtomArg::Atom(a) => iters.push(FairAssignIter::AtomPlaceholder(*a)),
                AtomArg::Var(v) => {
                    // Apply a classic grow-double strategy to the duplicates
                    if dups.len() >= dups.capacity() {
                        dups.reserve(dups.len() * 2);
                    }

                    // Now register this variable in both
                    dups.insert(*v, None);
                    iters.push(FairAssignIter::VarPlaceholder(*v));
                },
            }
        }

        // If there is no arguments whatsoever in this rule, then instead generate a single result
        if iters.is_empty() {
            trace!("Rule '{rule}' has no arguments, inserting phony once iterator");
            iters.push(FairAssignIter::Atom(
                std::iter::repeat(Ident { value: Span::new("HerbrandRuleInstIter::find_iters::phony", "Hello there!") }).take(1),
            ));
            return;
        }

        // If we found all variables, now go again to generate the iterators for the variables in a symbol-respecting manner
        let n_vars: usize = dups.len();
        for (i, iter) in iters.iter_mut().enumerate() {
            // Nothing to do for atoms
            let v: Ident = match iter {
                FairAssignIter::Atom(_) => unreachable!(),
                FairAssignIter::AtomPlaceholder(a) => {
                    // Overwrite it with a repeat iter that is bounded to the correct length
                    *iter = FairAssignIter::Atom(std::iter::repeat(*a).take(if n_vars > 1 { n_vars } else { 1 }));
                    continue;
                },
                FairAssignIter::Var(_) => unreachable!(),
                FairAssignIter::VarPlaceholder(v) => *v,
            };

            // Either fetch or generate the iterator for this variable
            let dup: &mut Option<_> = dups.get_mut(&v).expect("Got variable in arguments that is not in duplicate list");
            *iter = match dup {
                Some(iter) => FairAssignIter::Var(iter.clone()),
                None => {
                    // Generate it first

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
                    *dup = Some(RepeatIterator::new(consts.iter().cloned(), consts.len().pow((n_vars - 1 - i) as u32), consts.len().pow(i as u32)));

                    // OK, return it
                    FairAssignIter::Var(dup.clone().expect("I literally just created the assignment and it's None. Kinda weird ngl"))
                },
            };
        }
    }
}
impl<'c, 's> HerbrandRuleInstIter<'c, 's> {
    /// Constructor for the HerbrandRuleInstIter.
    ///
    /// # Arguments
    /// - `consts`: A set of constants that is quantified over to assign to variables.
    /// - `rule`: The [`Rule`] to iterate over and find new variable assignments.
    ///
    /// # Returns
    /// A new HerbrandRuleInstIter.
    fn new(consts: &'c IndexSet<Ident>, rule: &'s Rule) -> Self {
        debug!(
            "Created new HerbrandRuleInstIter\n\nConstants:\n{}\n{}\n{}\n\nRule:\n{}\n   {}\n{}\n",
            (0..80).map(|_| '-').collect::<String>(),
            consts.iter().map(|c| format!("   {c}")).collect::<Vec<String>>().join("\n"),
            (0..80).map(|_| '-').collect::<String>(),
            (0..80).map(|_| '-').collect::<String>(),
            rule,
            (0..80).map(|_| '-').collect::<String>(),
        );

        // Find the iterators using the neat helper function
        let mut dups = IndexMap::new();
        let mut iters = Vec::new();
        Self::find_iters(consts, rule, &mut dups, &mut iters);

        // OK, done!
        Self { consts, rule, dups, iters }
    }

    /// Refreshes the HerbrandRuleInstIter with a new rule to iterate over.
    ///
    /// # Arguments
    /// - `rule`: The new [`Rule`] to iterate over and find new variable assignments.
    fn refresh(&mut self, rule: &'s Rule) {
        self.rule = rule;
        Self::find_iters(self.consts, self.rule, &mut self.dups, &mut self.iters);
    }
}
impl<'c, 's> Iterator for HerbrandRuleInstIter<'c, 's> {
    type Item = Rule;

    #[inline]
    fn next(&mut self) -> Option<Rule> {
        let mut rule: Rule = self.rule.clone();

        // Prepare an iterator over all the rule's arguments
        // EDITOR'S NOTE: Looks horrible, but is just a chain of both the consequents and antecedents in the rule. Oh yeah and the zip'ed iterators themselves.
        let mut args = rule.consequences.values_mut().map(|a| a.args.iter_mut().map(|a| a.args.values_mut()).flatten()).flatten().chain(
            rule.tail
                .iter_mut()
                .map(|t| t.antecedents.values_mut().map(|a| a.atom_mut().args.iter_mut().map(|a| a.args.values_mut()).flatten()).flatten())
                .flatten(),
        );

        // Now iterate over the iters primarily, as they are tuned to catch empty rules
        for iter in &mut self.iters {
            // Get the next argument and assignment
            let arg: &mut AtomArg = args.next()?;
            let assign: Ident = iter.next()?;
            *arg = AtomArg::Atom(assign);
        }
        Some(rule)
    }
}

/// Defines an iterator over the _Herbrand instantiation_ of a program.
///
/// This is simply the base program, but then with additional rules such that all variables are replaced with all possible contents.
///
/// For some programs, the Herbrand base is infinite. This is not the case for $Datalog^\neg$, because it doesn't allow nesting and types atoms by arity.
///
/// This iterator relies on the [`HerbrandConstIter`] iterator to provide the "all possible constants" part.
pub struct HerbrandInstIter<'s, 'c> {
    /// The set of rules to iterate over.
    rules:  std::slice::Iter<'s, Rule>,
    /// The set of constants that we use to quantify.
    consts: &'c IndexSet<Ident>,
    /// An assignment of variables for the current atom we're iterating over.
    vars:   Option<HerbrandRuleInstIter<'c, 's>>,
}
impl<'s, 'c> HerbrandInstIter<'s, 'c> {
    /// Constructor for a HerbrandInstIter.
    ///
    /// # Arguments
    /// - `spec`: Some [`Spec`] to iterate over.
    /// - `consts`: An [`IndexSet`] of the found, unique constants in the given [`Spec`].
    ///
    /// # Returns
    /// An HerbrandInstIter that will produce all rules in the `spec` but concretized.
    pub fn new(spec: &'s Spec, consts: &'c IndexSet<Ident>) -> Self {
        debug!(
            "Created new HerbrandInstIter\n\nSpec:\n{}\n{}\n{}\n\nConstants:\n{}\n{}\n{}\n",
            (0..80).map(|_| '-').collect::<String>(),
            spec.rules.iter().map(|r| format!("   {r}")).collect::<Vec<String>>().join("\n"),
            (0..80).map(|_| '-').collect::<String>(),
            (0..80).map(|_| '-').collect::<String>(),
            consts.iter().map(|c| format!("   {c}")).collect::<Vec<String>>().join("\n"),
            (0..80).map(|_| '-').collect::<String>()
        );

        // Build ourselves with nothing in it (as long as there are rules, next() takes care of everything)
        Self { rules: spec.rules.iter(), consts, vars: None }
    }
}
impl<'s, 'c> Iterator for HerbrandInstIter<'s, 'c> {
    type Item = Rule;

    fn next(&mut self) -> Option<Rule> {
        // Attempt to get the next mapping
        if let Some(rule) = self.vars.as_mut().map(|v| v.next()).flatten() {
            return Some(rule);
        }

        // Otherwise, attempt to find the next rule
        let rule: &'s Rule = self.rules.next()?;
        match &mut self.vars {
            Some(vars) => vars.refresh(rule),
            None => self.vars = Some(HerbrandRuleInstIter::new(self.consts, rule)),
        }
        self.next()
    }
}
