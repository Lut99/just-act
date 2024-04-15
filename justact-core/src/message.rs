//  MESSAGE.rs
//    by Lut99
//
//  Created:
//    15 Apr 2024, 14:59:05
//  Last edited:
//    15 Apr 2024, 16:57:49
//  Auto updated?
//    Yes
//
//  Description:
//!   Defines the [`Message`]-trait, which represents some piece of policy
//!   information shared by agents. It also includes [`MessageSet`]s,
//!   which represents a particular collection of them.
//

use crate::policy::Policy;


/***** LIBRARY *****/
/// Provides the abstraction for a message that is sent between agents.
pub trait Message {
    /// Defines how authors look like.
    ///
    /// Usually, this aligns with [`Agent::Identifier`](crate::agent::Agent::Identifier) in order to match agents with (their own) messages.
    type Author;

    /// Returns the author of the message.
    ///
    /// # Returns
    /// An author of type [`Self::Author`](Message::Author) that represents who sent the message.
    fn author(&self) -> Self::Author;
}

/// Defines a _meaningful_ collection of messages.
///
/// This is a particular set of messages that can be interpreted as a [`Policy`].
///
/// This is meaningfully different from a [`MessageCollection`], as that does not impose such a semantic cohesion on its elements.
pub trait MessageSet {
    /// The type of messages which are contained in this MessageSet.
    type Message: Message;
    /// The type of policy extracted from this set.
    type Policy<'s>: 's + Policy
    where
        Self: 's;


    /// Returns some policy from the fragments contained in the messages of this set.
    ///
    /// # Returns
    /// A new policy of type [`Self::Policy`](MessageSet::Policy) that is the extracted policy.
    fn extract<'s>(&'s self) -> Self::Policy<'s>;
}

/// Defines a _meaningless_ collection of messages.
///
/// This is a particular set of messages that are grouped together by chance or necessecity, _not_ because they form a coherent policy.
///
/// This is meaningfully different from a [`MessageSet`], as that _does_ impose such a semantic cohesion on its elements.
pub trait MessageCollection {}

/// Defines a justified enactment.
///
/// This is simply a stand-in for a tuple of a _basis_ (agreement), _justification_ and _enactment_, all three [`MessageSet`]s.
pub trait Action {
    /// The type of MessageSet out of which this Action is built.
    type MessageSet: MessageSet;


    /// Returns the _basis_ of this action.
    ///
    /// This is usually the agreement which was commonly agreed to at the time the action was taken.
    ///
    /// # Returns
    /// A `Self::MessageSet` describing the basis of the action.
    fn basis(&self) -> &Self::MessageSet;

    /// Returns the _justification_ of this action.
    ///
    /// This is the part of the action that takes care to keep the _basis_ valid which taking the _enactment_ into account.
    ///
    /// # Returns
    /// A `Self::MessageSet` describing the justification of the action.
    fn justification(&self) -> &Self::MessageSet;

    /// Returns the _enactment_ of this action.
    ///
    /// Defines the effects of the action in policy land.
    ///
    /// # Returns
    /// A `Self::MessageSet` describing the basis of the action.
    fn enact(&self) -> &Self::MessageSet;
}
