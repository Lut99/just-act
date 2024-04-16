//  DATALOG.rs
//    by Lut99
//
//  Created:
//    15 Apr 2024, 15:56:10
//  Last edited:
//    16 Apr 2024, 16:32:29
//  Auto updated?
//    Yes
//
//  Description:
//!   Exposes $Datalog^\neg$ for use in JustAct.
//

use std::borrow::Cow;
use std::fmt::{Display, Formatter, Result as FResult};

use datalog::ast::{Atom, Ident, Rule, Span};
use datalog::interpreter::alternating_fixpoint;
use datalog::interpreter::interpretation::Interpretation;
pub use datalog::*;
use justact_core::policy as justact;


/***** LIBRARY *****/
/// Wraps a $Datalog^\neg$-policy [`Spec`] into something usable by the framework.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Policy<'s>(pub Vec<Cow<'s, Rule>>);
impl<'s> Policy<'s> {
    /// Runs the alternating fixpoint semantics for real, producing an [`Interpretation`] of the policy.
    ///
    /// # Returns
    /// An [`Interpretation`] that denotes which facts were derived.
    #[inline]
    pub fn alternating_fixpoint(&self) -> Interpretation {
        alternating_fixpoint(self.0.iter().map(|r| r.as_ref())).expect("Got too many variables in policy")
    }

    /// Returns a clone of this Policy that refers all rules to this policy.
    ///
    /// # Returns
    /// A [`Policy<'s>`].
    #[inline]
    pub fn as_borrow(&self) -> Policy {
        let rules: Vec<Cow<Rule>> = self.0.iter().map(|r| Cow::Borrowed(r.as_ref())).collect();
        Policy(rules)
    }

    /// Returns a clone of this Policy that owns all of its rules.
    ///
    /// # Returns
    /// A [`Policy<'static>`].
    #[inline]
    pub fn to_owned(&self) -> Policy<'static> {
        let rules: Vec<Cow<'static, Rule>> = self.0.iter().map(|r| Cow::Owned(r.clone().into_owned())).collect();
        Policy(rules)
    }
}

impl<'s> Display for Policy<'s> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        for rule in &self.0 {
            writeln!(f, "{rule}")?;
        }
        Ok(())
    }
}
impl<'s> justact::Policy for Policy<'s> {
    type Explanation = Interpretation;

    fn check_validity(&self) -> Result<(), Self::Explanation> {
        // Run the interpreter and see if we derive `error`.
        let int: Interpretation = self.alternating_fixpoint();
        let error_truth: Option<bool> = int.closed_world_truth(&Atom {
            ident: Ident { value: Span::new("<justact_policy::datalog::Policy::is_valid() generated>", "error") },
            args:  None,
        });

        // Check if it's OK
        if error_truth == Some(false) { Ok(()) } else { Err(int) }
    }
}
