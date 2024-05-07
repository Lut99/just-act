//  DATALOG.rs
//    by Lut99
//
//  Created:
//    15 Apr 2024, 15:56:10
//  Last edited:
//    07 May 2024, 16:36:26
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
pub struct Policy<'f, 's> {
    /// The spec that we wrap in this policy.
    pub spec: Spec<'f, 's>,
}

impl<'f, 's> Deref for Policy<'f, 's> {
    type Target = Spec<'f, 's>;

    #[inline]
    fn deref(&self) -> &Self::Target { &self.spec }
}
impl<'f, 's> DerefMut for Policy<'f, 's> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.spec }
}
impl<'f, 's> Display for Policy<'f, 's> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult { write!(f, "{}", self.spec) }
}
impl<'f, 's> justact::Policy for Policy<'f, 's> {
    type Explanation = Interpretation<'f, 's>;

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

impl<'f, 's> FromIterator<Policy<'f, 's>> for Policy<'f, 's> {
    #[inline]
    fn from_iter<T: IntoIterator<Item = Policy<'f, 's>>>(iter: T) -> Self {
        let mut rules: Vec<Rule> = Vec::new();
        for policy in iter {
            rules.extend(policy.spec.rules);
        }
        Policy { spec: Spec { rules } }
    }
}
impl<'f, 's> From<Spec<'f, 's>> for Policy<'f, 's> {
    #[inline]
    fn from(value: Spec<'f, 's>) -> Self { Self { spec: value } }
}
impl<'f, 's> From<Policy<'f, 's>> for Spec<'f, 's> {
    #[inline]
    fn from(value: Policy<'f, 's>) -> Self { value.spec }
}
