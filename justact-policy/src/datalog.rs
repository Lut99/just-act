//  DATALOG.rs
//    by Lut99
//
//  Created:
//    15 Apr 2024, 15:56:10
//  Last edited:
//    15 Apr 2024, 19:07:12
//  Auto updated?
//    Yes
//
//  Description:
//!   Exposes $Datalog^\neg$ for use in JustAct.
//

use std::borrow::Cow;

use datalog::ast::{Atom, Ident, Rule, Span};
use datalog::interpreter::alternating_fixpoint;
use datalog::interpreter::interpretation::Interpretation;
pub use datalog::*;
use justact_core::policy as justact;


/***** LIBRARY *****/
/// Wraps a $Datalog^\neg$-policy [`Spec`] into something usable by the framework.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Policy<'s>(pub Vec<Cow<'s, Rule>>);
impl<'s> justact::Policy for Policy<'s> {
    fn is_valid(&self) -> bool {
        // Run the interpreter and see if we don't derive [`error`].
        let int: Interpretation = alternating_fixpoint(self.0.iter().map(|r| r.as_ref())).expect("Got too many variables in policy");
        int.closed_world_truth(&Atom {
            ident: Ident { value: Span::new("<justact_policy::datalog::Policy::is_valid() generated>", "error") },
            args:  None,
        })
        .unwrap_or(true)
    }
}
