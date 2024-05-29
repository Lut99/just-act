//  POLICY.rs
//    by Lut99
//
//  Created:
//    29 May 2024, 11:17:34
//  Last edited:
//    29 May 2024, 11:51:27
//  Auto updated?
//    Yes
//
//  Description:
//!   Defines JustAct's abstract notion of policy.
//

use std::error::Error;

use crate::auxillary::{Authored, Identifiable};
use crate::set::LocalSet;
use crate::statements::Message;


/***** LIBRARY *****/
/// Defines the framework's notion of policy.
///
/// This is usually accompanied by [`Extractable`] in order to communicate that policy can be
/// extracted from message sets.
pub trait Policy {
    /// The type of error emitted when the policy is not valid (**semantically** incorrect).
    type SemanticError: Error;

    /// Checks whether this policy is valid according to its own semantics.
    ///
    /// # Errors
    /// If the policy is not valid, this function errors. The resulting
    /// [`Self::SemanticError`](Policy::SemanticError) encodes some explanation of why the policy
    /// wasn't valid.
    fn assert_validity(&self) -> Result<(), Self::SemanticError>;
}



/// Defines the `extract()`-function for some [`Policy`].
///
/// This is what is used to get some policy from a set of messages.
///
/// # Generics
/// - `M`: Some type of message that is contained within the given `set`.
pub trait Extractor<M> {
    /// The policy that is extracted.
    type Policy<'s>: Policy
    where
        Self: 's;
    /// The type of error emitted when the policy is **syntactically** incorrect.
    type SyntaxError<'s>: Error
    where
        Self: 's;


    /// Extracts this policy from a given [`Set`] over messages.
    ///
    /// # Arguments
    /// - `set`: The [`LocalSet`] to extract from.
    ///
    /// # Returns
    /// A new instance of `Self::Policy` as encoded in the given `set`.
    ///
    /// # Errors
    /// This function should throw a [`Self::SyntaxError`](Extractor::SyntaxError) if and only if
    /// the combined messages' payloads did not make a **syntactically** correct policy.
    ///
    /// Semantic correctness is conventionally modelled by returning a legal policy, but that fails
    /// the [`Policy::assert_validity()`]-check.
    fn extract<'v, R>(set: &LocalSet<M, R>) -> Result<Self::Policy<'v>, Self::SyntaxError<'v>>
    where
        Self: Sized,
        M: Authored + Identifiable + Message<'v>;
}
