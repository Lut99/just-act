//  DATALOG.rs
//    by Lut99
//
//  Created:
//    15 Apr 2024, 15:56:10
//  Last edited:
//    18 Apr 2024, 14:33:18
//  Auto updated?
//    Yes
//
//  Description:
//!   Exposes $Datalog^\neg$ for use in JustAct.
//

use std::fmt::{Display, Formatter, Result as FResult};
use std::ops::{Deref, DerefMut};

use datalog::ast::{Atom, Ident, Rule, Span, Spec};
use datalog::interpreter::interpretation::Interpretation;
pub use datalog::*;
use justact_core::policy as justact;


/***** LIBRARY *****/
/// Wraps a $Datalog^\neg$-policy [`Spec`] into something usable by the framework.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Policy {
    /// The spec that we wrap in this policy.
    pub spec: Spec,
}

impl Deref for Policy {
    type Target = Spec;

    #[inline]
    fn deref(&self) -> &Self::Target { &self.spec }
}
impl DerefMut for Policy {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.spec }
}
impl Display for Policy {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult { write!(f, "{}", self.spec) }
}
impl justact::Policy for Policy {
    type Explanation = Interpretation;

    fn check_validity(&self) -> Result<(), Self::Explanation> {
        // Run the interpreter and see if we derive `error`.
        let int: Interpretation = self.spec.alternating_fixpoint().unwrap();
        let error_truth: Option<bool> = int.closed_world_truth(&Atom {
            ident: Ident { value: Span::new("<justact_policy::datalog::Policy::is_valid() generated>", "error") },
            args:  None,
        });

        // Check if it's OK
        if error_truth == Some(false) { Ok(()) } else { Err(int) }
    }
}

impl FromIterator<Policy> for Policy {
    #[inline]
    fn from_iter<T: IntoIterator<Item = Policy>>(iter: T) -> Self {
        let mut rules: Vec<Rule> = Vec::new();
        for policy in iter {
            rules.extend(policy.spec.rules);
        }
        Policy { spec: Spec { rules } }
    }
}
impl From<Spec> for Policy {
    #[inline]
    fn from(value: Spec) -> Self { Self { spec: value } }
}
impl From<Policy> for Spec {
    #[inline]
    fn from(value: Policy) -> Self { value.spec }
}
