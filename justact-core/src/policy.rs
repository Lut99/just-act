//  POLICY.rs
//    by Lut99
//
//  Created:
//    15 Apr 2024, 15:11:07
//  Last edited:
//    21 May 2024, 15:10:55
//  Auto updated?
//    Yes
//
//  Description:
//!   Defines the interface between the JustAct framework as a whole and a
//!   chosen policy language.
//

use std::error::Error;


/***** LIBRARY *****/
/// Defines how a Policy looks like.
pub trait Policy {
    /// The type of the object that explains why this policy is invalid.
    type Explanation;


    /// Checks if this policy is valid.
    ///
    /// This can differ wildy per policy language what this means.
    ///
    /// # Errors
    /// If the policy is not valid, then this returns some kind of `Self::Explanation` explaining why it wasn't.
    fn check_validity(&self) -> Result<(), Self::Explanation>;
}

/// Defines an extension to [`Policy`] that marks it as being parsed from the payload of a [`MessageSet`](crate::wire::MessageSet).
///
/// This trait mirrors the signature of [`TryFrom`] explicitly, except separately to avoid conflicts with builtin definitions for `TryFrom`.
///
/// # Generics
/// - `I`: The type of [`MessageSet::PayloadIter`](crate::wire::MessageSet::PayloadIter)ator returned by the message set, and which can be used to construct this policy.
///
/// # Example
/// Typically, one would implement this using a blanket implementation for all possible payload iterators in order to be generic over MessageSet implementations.
///
/// For example:
/// ```rust
/// use std::error::Error;
/// use std::fmt::{Display, Formatter, Result as FResult};
///
/// use justact_core::{ExtractablePolicy, Message, Policy};
///
/// #[derive(Debug)]
/// struct SimplePolicyError;
/// impl Display for SimplePolicyError {
///     fn fmt(&self, f: &mut Formatter) -> FResult {
///         write!(f, "Policy was not either 'true' or 'false'")
///     }
/// }
/// impl Error for SimplePolicyError {}
///
/// struct SimplePolicy {
///     allowed: bool,
/// };
/// impl Policy for SimplePolicy {
///     type Explanation = bool;
///
///     fn check_validity(&self) -> Result<(), Self::Explanation> {
///         if self.allowed { Ok(()) } else { Err(self.allowed) }
///     }
/// }
/// impl<I: Iterator<Item = M>, M: Message> ExtractablePolicy<I> for SimplePolicy {
///     type ExtractError = SimplePolicyError;
///
///     fn extract_from(msgs: I) -> Result<Self, Self::ExtractError> {
///         // Collect all the policy input
///         let mut input = Vec::<u8>::new();
///         for msg in msgs {
///             input.extend(msg.payload());
///         }
///
///         // Check if it what we expect
///         if &input == b"true" {
///             Ok(Self { allowed: true })
///         } else if &input == b"false" {
///             Ok(Self { allowed: false })
///         } else {
///             Err(SimplePolicyError)
///         }
///     }
/// }
/// ```
pub trait ExtractablePolicy<M>: Policy {
    /// The error that is raised if we failed to parse a policy from the MessageSet's payload.
    ///
    /// Note that this should only reflect _syntactic_ invalidity instead of _semantic_ invalidity.
    /// For example, in the case of $Datalog^\neg$, a completely invalid datalog program would raise an error:
    /// ```datalog
    /// @&(@*&#$)       # Result::Err
    /// ```
    /// However, an invalid but syntactically correct policy would not:
    /// ```datalog
    /// error.          # Result::Ok
    /// ```
    type ExtractError: Error;


    /// Parses this Policy from a [`MessageSet`](crate::wire::MessageSet)'s payload.
    ///
    /// The payload is usually given as an iterator over byte slices ([`Iterator<Item = &[u8]>`]).
    /// As a result, to be agnostic to the specific MessageSet implementation, it is recommended to implement this function as a blanket over all possible iterators.
    ///
    /// See the [main trait](ParsablePolicy) for an example.
    ///
    /// # Arguments
    /// - `msgs`: Some [`Iterator`] that yields the messages in some message set. These, together, should form a complete policy.
    ///
    /// # Returns
    /// A new instance of Self, representing the extracted policy.
    ///
    /// # Errors
    /// This function should fail if there was some _syntactic_ problem in the input that you want to notify to the user for debugging purposes.
    /// All other cases (semantic invalidity, problems with parsing that do not need reporting) should simply result in an instance of self for which [`Policy::check_validity()`] fails.
    fn extract_from(msgs: impl IntoIterator<Item = M>) -> Result<Self, Self::ExtractError>
    where
        Self: Sized;
}
