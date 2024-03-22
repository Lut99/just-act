//  MOD.rs
//    by Lut99
//
//  Created:
//    21 Mar 2024, 10:27:36
//  Last edited:
//    22 Mar 2024, 16:11:18
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
//

// Declare nested modules
pub mod afps;
pub mod herbrand;
pub mod interpretation;


// /***** LIBRARY *****/
// /// Evaluates a given $Datalog^\neg$ AST.
// ///
// /// Contains a knowledge base internally. That means that different interpreter instances may give different answers.
// #[derive(Clone, Debug)]
// pub struct Interpreter {
//     /// The set of facts that we know exist.
//     pub knowledge_base: IndexSet<Atom>,
// }
// impl Default for Interpreter {
//     #[inline]
//     fn default() -> Self { Self::new() }
// }
// impl Interpreter {
//     /// Constructor for the Interpreter that initializes it with an empty knowledge base.
//     ///
//     /// # Returns
//     /// A new Interpreter instance with nothing derived yet.
//     #[inline]
//     pub fn new() -> Self { Self { knowledge_base: IndexSet::new() } }

//     /// Performs "one-time" evaluation on the given specification.
//     ///
//     /// This is equivalent to creating a new interpreter and interpreting with that.
//     ///
//     /// # Arguments
//     /// - `spec`: The $Datalog^\neg$ [`Spec`]ification to evaluate.
//     ///
//     /// # Returns
//     /// A derived set of facts, as a [`HashSet<String>`].
//     ///
//     /// # Example
//     /// ```rust
//     /// use justact_ast::{datalog, Spec};
//     /// use justact_datalog::Interpeter;
//     ///
//     /// let spec: Spec = datalog!(foo.);
//     ///
//     /// // The verbose way
//     /// let mut int = Interpreter::new();
//     /// int.evaluate(&spec);
//     ///
//     /// // The short way
//     /// let short = Interpeter::evaluate_once(&spec);
//     /// assert_eq!(int.knowledge_base, short);
//     /// ```
//     #[inline]
//     pub fn evaluate_once(spec: &Spec) -> IndexSet<Atom> {
//         let mut int: Self = Self::new();
//         int.evaluate(spec);
//         int.knowledge_base
//     }

//     /// Preforms evaluation on the given specification.
//     ///
//     /// This updates the internal `knowledge_base`. You can manually inspect this.
//     ///
//     /// # Algorithm
//     /// The interpreter relies on the _well-founded semantics_ to do derivation in a way that deals more intuitively with negate antecedents.
//     ///
//     /// Concretely, the well-founded semantics works
//     ///
//     /// # Arguments
//     /// - `spec`: The $Datalog^\neg$ [`Spec`]ification to evaluate.
//     ///
//     /// # Example
//     /// ```rust
//     /// use justact_ast::{datalog, Spec};
//     /// use justact_datalog::Interpeter;
//     ///
//     /// let mut int = Interpreter::new();
//     /// int.evaluate(&datalog!(foo.));
//     /// assert!(int.holds("foo"));
//     /// ```
//     pub fn evaluate(&mut self, spec: &Spec) {
//         // //

//         // // Go thru the rules
//         // for rule in &spec.rules {
//         //     // Consider all concrete instances based on variables
//         //     let mut new_instances: IndexSet<Atom> = IndexSet::new();
//         //     for concrete_rule in RuleConcretizer::new(rule, &self.knowledge_base) {}
//         // }
//     }
// }
