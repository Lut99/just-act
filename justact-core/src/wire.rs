//  WIRE.rs
//    by Lut99
//
//  Created:
//    15 Apr 2024, 14:59:05
//  Last edited:
//    14 May 2024, 10:05:35
//  Auto updated?
//    Yes
//
//  Description:
//!   Defines the [`Action`]- and [`Message`]-traits, which represent policy
//!   information on the wire.
//

use std::borrow::Cow;

use crate::auxillary::{Authored, Identifiable};
use crate::global::GlobalView;
use crate::local::Statements;
use crate::policy::ExtractablePolicy;
use crate::set::Set;


/***** LIBRARY *****/
/// Provides the abstraction for a message that is sent between agents.
pub trait Message: Authored + Identifiable {
    /// Returns the payload of the message, as raw bytes.
    ///
    /// This is typically a syntactically correct policy message.
    ///
    /// # Returns
    /// A byte slice containing the message's raw binary payload.
    /// Anything from encoding to padding is left as an exercise for the particular [`ExtractablePolicy`](crate::policy::ExtractablePolicy)-implementation.
    fn payload(&self) -> &[u8];
}

// Implement over some pointer-like types
impl<'a, T: Message> Message for &'a T {
    #[inline]
    fn payload(&self) -> &[u8] { T::payload(self) }
}
impl<'a, T: Clone + Message> Message for Cow<'a, T> {
    #[inline]
    fn payload(&self) -> &[u8] { T::payload(self) }
}

/// Defines a collection of messages.
///
/// This is a particular set of messages that can be interpreted as a [`Policy`].
///
/// Typically, MessageSets will want to implement something akin to [`From<Self::Message>`] in order to be kind to single messages.
pub trait MessageSet: Set<Self::Message> {
    /// The type of messages which are contained in this MessageSet.
    type Message: Message;


    /// Convenience alias for calling a particular [`Policy`](crate::policy::Policy)s [`extract_from()`](crate::policy::Policy::extract_from())-implementation on this MessageSet's payload.
    ///
    /// # Generics
    /// - `P`: The particular type of [`ExtractablePolicy`] to try and attempt to parse the payload as.
    ///
    /// # Returns
    /// A new instance of `P` that represents the parsed policy.
    ///
    /// # Errors
    /// This function fails if we failed to extract the policy from our payload bytes.
    #[inline]
    fn extract<'s, P>(&'s self) -> Result<P, P::ExtractError>
    where
        P: ExtractablePolicy<Self::Iter<'s>>,
    {
        // Default impl: just wrap the given policy's `extract_from()`
        P::extract_from(self.iter())
    }
}

// Implement the `MessageSet` for pointer-like types.
impl<'a, T> MessageSet for &'a T
where
    T: MessageSet,
    &'a T: Set<T::Message>,
{
    type Message = T::Message;
}
impl<'a, T> MessageSet for Cow<'a, T>
where
    T: Clone + MessageSet,
    Cow<'a, T>: Set<T::Message>,
{
    type Message = T::Message;
}



/// Defines an agreed-upon message.
pub trait Agreement {
    /// The set returned by this agreement that represents what is agreed upon.
    type MessageSet<'s>: MessageSet
    where
        Self: 's;

    /// The type of the time at which this Action can be taken.
    type Time: Ord;


    /// Returns the set of statements that was agreed upon by all of the agents.
    ///
    /// # Returns
    /// Some `Self::MessageSet` that represents the set of messages.
    fn statements<'s>(&'s self) -> Self::MessageSet<'s>;

    /// Returns some time at which this agreement can be used to enact actions.
    ///
    /// The returned time should be part of the globally synchronized [`Times`](crate::global::Times)-set.
    ///
    /// # Returns
    /// Some `Self::Time` noting when the action was enacted.
    fn applies_at(&self) -> Self::Time;
}

// Implement `Agreement` for some pointer-like types
impl<'a, T: Agreement> Agreement for &'a T {
    type MessageSet<'s> = T::MessageSet<'s> where Self: 's;
    type Time = T::Time;

    #[inline]
    fn statements<'s>(&'s self) -> Self::MessageSet<'s> { T::statements(self) }
    #[inline]
    fn applies_at(&self) -> Self::Time { T::applies_at(self) }
}

/// Defines a justified enactment.
///
/// This is simply a stand-in for a tuple of a _basis_ (agreement), _justification_ and _enactment_, all three [`MessageSet`]s.
pub trait Action {
    /// The type of the time at which this Action can be taken.
    type Time: Ord;
    /// The type of Agreement which forms the `Self::basis()` of the agreement.
    type Agreement<'s>: 's + Agreement
    where
        Self: 's;
    /// The type of MessageSet out of which this Action is built.
    type MessageSet<'s>: 's + MessageSet
    where
        Self: 's;
    /// The type of Message out of which this Action is built. In particular, this is what is returned by `Self::enacts()`.
    type Message<'s>: 's + Message
    where
        Self: 's;


    /// Returns some time at which this action was taken.
    ///
    /// The returned time should be part of the globally synchronized [`Times`](crate::global::Times)-set.
    ///
    /// # Returns
    /// Some `Self::Time` noting when the action was enacted.
    fn taken_at(&self) -> Self::Time;

    /// Returns the _basis_ of this action.
    ///
    /// This is usually the agreement which was commonly agreed to at the time the action was taken.
    ///
    /// # Returns
    /// A `Self::Agreement` describing the basis of the action.
    fn basis<'s>(&'s self) -> Self::Agreement<'s>;

    /// Returns the _justification_ of this action.
    ///
    /// This is the part of the action that takes care to keep the _basis_ valid which taking the _enactment_ into account.
    ///
    /// Note that, as per the paper, this should already include the messages returned by [`Action::basis()`] and [`Action::enacts()`].
    ///
    /// # Returns
    /// A `Self::MessageSet` describing the justification of the action.
    fn justification<'s>(&'s self) -> Self::MessageSet<'s>;

    /// Returns the _enactment_ of this action.
    ///
    /// Defines the effects of the action in policy land.
    ///
    /// # Returns
    /// A `Self::Message` describing the enacted effects of the action.
    fn enacts<'s>(&'s self) -> Self::Message<'s>;
}

/// Defines an extension to [`Action`]s that allows them to be audited.
pub trait AuditableAction: Action {
    /// Something that explains why this Action did not succeed an audit.
    type Explanation;

    /// Audits this action.
    ///
    /// In particular, will check if the [justification](Action::justification()) is valid according to the policy language and all of its embedded messages have been stated.
    ///
    /// # Arguments
    /// - `global`: Some [`GlobalView`] that can be used to check which statements are agreements and what the current time is.
    /// - `stmts`: Some [`Statements`] that can be used to check which messages are stated.
    ///
    /// # Errors
    /// If this action did not match the requirements of the audit, then an [`Action::Explanation`] is returned with why this is the case.
    fn audit<G, S>(&self, global: &G, stmts: &S) -> Result<(), Self::Explanation>
    where
        G: GlobalView,
        S: Statements;
}
