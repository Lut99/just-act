//  INTERPRETER.rs
//    by Lut99
//
//  Created:
//    13 Mar 2024, 17:54:05
//  Last edited:
//    14 Mar 2024, 16:59:11
//  Auto updated?
//    Yes
//
//  Description:
//!   Evaluates a given $Datalog^\neg$ AST.
//

use std::collections::{BTreeSet, HashSet};

use ast_toolkit_punctuated::Punctuated;
use ast_toolkit_span::Spannable;

use crate::ast::{Atom, AtomArg, Comma, Literal, Rule, RuleAntecedents, Spec};


/***** TESTS *****/
#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn test_knowledge_base_iterator() {
        let kb: HashSet<String> = HashSet::new();
        assert_eq!(KnowledgeBaseIterator::new(&kb, 0).next(), None);
        assert_eq!(KnowledgeBaseIterator::new(&kb, 1).next(), None);
        assert_eq!(KnowledgeBaseIterator::new(&kb, 2).next(), None);
        assert_eq!(KnowledgeBaseIterator::new(&kb, 3).next(), None);
        assert_eq!(KnowledgeBaseIterator::new(&kb, 4).next(), None);



        let kb: HashSet<String> = HashSet::from(["foo".into(), "bar".into()]);

        let mut iter = KnowledgeBaseIterator::new(&kb, 0);
        assert_eq!(iter.next(), None);

        let mut iter = KnowledgeBaseIterator::new(&kb, 1);
        assert_eq!(iter.next(), Some([&String::from("foo")].as_slice()));
        assert_eq!(iter.next(), Some([&String::from("bar")].as_slice()));
        assert_eq!(iter.next(), None);

        let mut iter = KnowledgeBaseIterator::new(&kb, 2);
        assert_eq!(iter.next(), Some([&String::from("foo"), &String::from("foo")].as_slice()));
        assert_eq!(iter.next(), Some([&String::from("foo"), &String::from("bar")].as_slice()));
        assert_eq!(iter.next(), Some([&String::from("bar"), &String::from("foo")].as_slice()));
        assert_eq!(iter.next(), Some([&String::from("bar"), &String::from("bar")].as_slice()));
        assert_eq!(iter.next(), None);

        let mut iter = KnowledgeBaseIterator::new(&kb, 3);
        assert_eq!(iter.next(), Some([&String::from("foo"), &String::from("foo"), &String::from("foo")].as_slice()));
        assert_eq!(iter.next(), Some([&String::from("foo"), &String::from("foo"), &String::from("bar")].as_slice()));
        assert_eq!(iter.next(), Some([&String::from("foo"), &String::from("bar"), &String::from("foo")].as_slice()));
        assert_eq!(iter.next(), Some([&String::from("foo"), &String::from("bar"), &String::from("bar")].as_slice()));
        assert_eq!(iter.next(), Some([&String::from("bar"), &String::from("foo"), &String::from("foo")].as_slice()));
        assert_eq!(iter.next(), Some([&String::from("bar"), &String::from("foo"), &String::from("bar")].as_slice()));
        assert_eq!(iter.next(), Some([&String::from("bar"), &String::from("bar"), &String::from("foo")].as_slice()));
        assert_eq!(iter.next(), Some([&String::from("bar"), &String::from("bar"), &String::from("bar")].as_slice()));
        assert_eq!(iter.next(), None);



        let kb: HashSet<String> = HashSet::from(["foo".into(), "bar".into(), "baz".into()]);

        let mut iter = KnowledgeBaseIterator::new(&kb, 2);
        assert_eq!(iter.next(), Some([&String::from("foo"), &String::from("foo")].as_slice()));
        assert_eq!(iter.next(), Some([&String::from("foo"), &String::from("bar")].as_slice()));
        assert_eq!(iter.next(), Some([&String::from("foo"), &String::from("baz")].as_slice()));
        assert_eq!(iter.next(), Some([&String::from("bar"), &String::from("foo")].as_slice()));
        assert_eq!(iter.next(), Some([&String::from("bar"), &String::from("bar")].as_slice()));
        assert_eq!(iter.next(), Some([&String::from("bar"), &String::from("baz")].as_slice()));
        assert_eq!(iter.next(), Some([&String::from("baz"), &String::from("foo")].as_slice()));
        assert_eq!(iter.next(), Some([&String::from("baz"), &String::from("bar")].as_slice()));
        assert_eq!(iter.next(), Some([&String::from("baz"), &String::from("baz")].as_slice()));
        assert_eq!(iter.next(), None);
    }
}





/***** HELPER FUNCTIONS *****/
/// Traverses an atom to find all variables in its arguments.
///
/// # Arguments
/// - `atom`: The [`Atom`] to analyze.
///
/// # Returns
/// A list of the names of variables found.
fn find_vars_in_atom<'a, F, S>(atom: &'a Atom<F, S>) -> BTreeSet<S::Slice<'a>>
where
    S: Spannable,
    S::Slice<'a>: Ord,
{
    let mut res: BTreeSet<S::Slice<'a>> = BTreeSet::new();
    for arg in atom.args.iter().map(|a| a.args.values()).flatten() {
        match arg {
            AtomArg::Atom(atom) => res.append(&mut find_vars_in_atom(atom)),
            AtomArg::Var(var) => {
                res.insert(var.value.spanned());
            },
        }
    }
    res
}

/// Traverses a rule to find all variables.
///
/// # Arguments
/// - `rule`: The [`Rule`] to analyze.
///
/// # Returns
/// A list of the names of variables found.
fn find_vars_in_rule<'a, F, S>(rule: &'a Rule<F, S>) -> BTreeSet<S::Slice<'a>>
where
    S: Spannable,
    S::Slice<'a>: Ord,
{
    // Iterate over the consequences
    let mut res: BTreeSet<S::Slice<'a>> = BTreeSet::new();
    for con in rule.consequences.values() {
        res.append(&mut find_vars_in_atom(con));
    }
    // Add the antecedents
    for ant in rule.tail.iter().map(|h| h.antecedants.values()).flatten() {
        match ant {
            Literal::Atom(atom) => res.append(&mut find_vars_in_atom(atom)),
            Literal::NegAtom(natom) => res.append(&mut find_vars_in_atom(&natom.atom)),
        }
    }
    res
}



/// Replaces all variables in an atom with a given list of concrete instances.
///
/// # Arguments
/// - `atom`: The [`Atom`] to clone.

/// Swaps all variables in a rule with a given list of concrete instances.
///
/// # Arguments
/// - `rule`: The [`Rule`] to clone.
/// - `vars`: The instantiation of all the variables.
///
/// # Returns
/// A new [`Rule`] that doesn't have any variables.
fn replace_vars_in_rule<'r, F: Clone, S: Clone>(rule: &'r Rule<F, S>, vars: impl IntoIterator<Item = &'r String>) -> Rule<F, S> {
    // Clone the consequents
    let mut consequents: Punctuated<Atom<F, S>, Comma<F, S>> = rule.consequences.clone();
    for cons in consequents.values_mut() {
        replace_vars_in_atom(cons, vars);
    }

    // Clone the antecedents, if any
    let mut tail: Option<RuleAntecedents<F, S>> = rule.tail.clone();
}





/***** AUXILLARY *****/
/// Given a HashSet, generates all possible tuples of arbitrary length that can be taken from it.
#[derive(Debug)]
pub struct KnowledgeBaseIterator<'k> {
    /// The knowledge base to iterate over.
    kb:    &'k HashSet<String>,
    /// The current set of iterators that we're iterating.
    iters: Vec<std::collections::hash_set::Iter<'k, String>>,
    /// A vector that we allocated once and then return for efficiency.
    res:   Vec<&'k String>,
}
impl<'k> KnowledgeBaseIterator<'k> {
    /// Constructor for the KnowledgeBaseIterator.
    ///
    /// # Arguments
    /// - `knowledge_base`: The knowledge base (as a [`HashSet<String>`]) to iterate over.
    /// - `n_vars`: The number of variables to iterate (i.e., the tuple length).
    ///
    /// # Returns
    /// A new KnowledgeBaseIterator ready to go.
    #[inline]
    pub fn new(knowledge_base: &'k HashSet<String>, n_vars: usize) -> Self {
        Self { kb: knowledge_base, iters: vec![knowledge_base.iter(); n_vars], res: Vec::with_capacity(n_vars) }
    }

    /// Gets the next sample.
    ///
    /// Note: doesn't implement [`Iterator`] because it doesn't GAT :/
    ///
    /// # Returns
    /// A new [`&[&'k String]`] that represents the current assignment of variables.
    #[inline]
    pub fn next(&mut self) -> Option<&[&'k String]> {
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
                // SAFETY: We can call unwrap() because we asserted above the knowledge base is non-empty, and we know this one's only executed at first
                self.res.push(iter.next().unwrap());
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
                    *iter = self.kb.iter();
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
#[derive(Debug)]
pub struct RuleConcretizer<'r, 'k, F, S> {
    /// The rule that needs to be concretized.
    rule: &'r Rule<F, S>,
    /// The iterator that does the actual quantification.
    iter: KnowledgeBaseIterator<'k>,
    /// The variables that we will populate.
    vars: BTreeSet<&'r str>,
}
impl<'r, 'k, F, S> RuleConcretizer<'r, 'k, F, S> {
    /// Constructor that creates a new RuleConretizer.
    ///
    /// # Arguments
    /// - `rule`: The [`Rule`] to concretize.
    /// - `knowledge_base`: The knowledge base that we quantify over.
    ///
    /// # Returns
    /// A new RuleConcretizer that implements [`Iterator`].
    pub fn new(rule: &'r Rule<F, S>, knowledge_base: &'k HashSet<String>) -> Self
    where
        S: Spannable<Slice<'r> = &'r str>,
    {
        // Scan the rule for variables
        let vars: BTreeSet<&str> = find_vars_in_rule(rule).into_iter().map(|v| v.as_ref()).collect();

        // Build self
        Self { rule, iter: KnowledgeBaseIterator::new(knowledge_base, vars.len()), vars }
    }
}
impl<'r, 'k, F, S> Iterator for RuleConcretizer<'r, 'k, F, S> {
    type Item = Rule<F, S>;

    fn next(&mut self) -> Option<Self::Item> {
        // Get the next variable mapping
        let vars: &[&String] = self.iter.next()?;
        assert_eq!(vars.len(), self.vars.len());

        // Apply it to the variables in the rule
        Some(clone_rule_as_concrete(self.rule, vars.into_iter()))
    }
}





/***** LIBRARY *****/
/// Evaluates a given $Datalog^\neg$ AST.
///
/// Contains a knowledge base internally. That means that different interpreter instances may give different answers.
#[derive(Clone, Debug)]
pub struct Interpreter {
    /// The set of facts that we know exist.
    pub knowledge_base: HashSet<String>,
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
    pub fn new() -> Self { Self { knowledge_base: HashSet::new() } }

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
    pub fn evaluate_once<'s, F, S>(spec: &'s Spec<F, S>) -> HashSet<String>
    where
        S: Spannable<Slice<'s> = &'s str>,
    {
        let mut int: Self = Self::new();
        int.evaluate(spec);
        int.knowledge_base
    }

    /// Preforms evaluation on the given specification.
    ///
    /// This updates the internal `knowledge_base`. You can manually inspect this.
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
    pub fn evaluate<'s, F, S>(&'_ mut self, spec: &'s Spec<F, S>)
    where
        S: Spannable<Slice<'s> = &'s str>,
    {
        // Go thru the rules
        for rule in &spec.rules {
            // Consider all concrete instances based on variables
            let mut new_instances: HashSet<String> = HashSet::new();
            for concrete_rule in RuleConcretizer::new(rule, &self.knowledge_base) {}
        }
    }
}
