//  MESSAGE.rs
//    by Lut99
//
//  Created:
//    15 Apr 2024, 14:59:05
//  Last edited:
//    26 Apr 2024, 11:31:21
//  Auto updated?
//    Yes
//
//  Description:
//!   Defines the [`Message`]-trait, which represents some piece of policy
//!   information shared by agents.
//

use std::borrow::Cow;

use crate::set::MessageSet;
use crate::statements::Statements;


/***** LIBRARY *****/
/// Provides the abstraction for a message that is sent between agents.
pub trait Message {
    /// Defines how message identifiers look like.
    type Identifier;
    /// Defines how authors look like.
    type Author;

    /// Returns some identifier of the message.
    ///
    /// # Returns
    /// An identifier of type [`Self::Id`](Message::Id).
    fn id(&self) -> Self::Identifier;

    /// Returns the author of the message.
    ///
    /// # Returns
    /// An author of type [`Self::Author`](Message::Author) that represents who sent the message.
    fn author(&self) -> Self::Author;
}

// Implement over some pointer-like types
impl<'a, T: Clone + Message> Message for &'a T {
    type Author = T::Author;
    type Identifier = T::Identifier;

    #[inline]
    fn author(&self) -> Self::Author { T::author(self) }

    #[inline]
    fn id(&self) -> Self::Identifier { T::id(self) }
}

impl<'a, T: Clone + Message> Message for Cow<'a, T> {
    type Author = T::Author;
    type Identifier = T::Identifier;

    #[inline]
    fn author(&self) -> Self::Author { T::author(self) }

    #[inline]
    fn id(&self) -> Self::Identifier { T::id(self) }
}



/// Defines a justified enactment.
///
/// This is simply a stand-in for a tuple of a _basis_ (agreement), _justification_ and _enactment_, all three [`MessageSet`]s.
pub trait Action {
    /// Something that explains why this Action did not succeed an audit.
    type Explanation;
    /// The type of Message out of which this Action is built.
    type Message<'s>: 's + Message
    where
        Self: 's;
    /// The type of MessageSet out of which this Action is built.
    type MessageSet<'s>: 's + MessageSet
    where
        Self: 's;


    /// Returns the _basis_ of this action.
    ///
    /// This is usually the agreement which was commonly agreed to at the time the action was taken.
    ///
    /// # Returns
    /// A `Self::MessageSet` describing the basis of the action.
    fn basis<'s>(&'s self) -> Self::Message<'s>;

    /// Returns the _justification_ of this action.
    ///
    /// This is the part of the action that takes care to keep the _basis_ valid which taking the _enactment_ into account.
    ///
    /// # Returns
    /// A `Self::MessageSet` describing the justification of the action.
    /// TODO: Includes basis and enactment
    fn justification<'s>(&'s self) -> Self::MessageSet<'s>;

    /// Returns the _enactment_ of this action.
    ///
    /// Defines the effects of the action in policy land.
    ///
    /// # Returns
    /// A `Self::MessageSet` describing the basis of the action.
    fn enactment<'s>(&'s self) -> Self::Message<'s>;


    /// Audits this action.
    ///
    /// In particular, will check if the [justification](Action::justification()) is valid according to the policy language and all of its embedded messages have been stated.
    ///
    /// # Arguments
    /// - `stmts`: Some [`Statements`] that can be used to check if messages are stated.
    ///
    /// # Errors
    /// If this action did not match the requirements of the audit, then an [`Action::Explanation`] is returned with why this is the case.
    fn audit<'s, S>(&'s self, stmts: &S) -> Result<(), Self::Explanation>
    where
        S: Statements<Id = <Self::Message<'s> as Message>::Identifier>;
}
