//  LIB.rs
//    by Lut99
//
//  Created:
//    13 Mar 2024, 15:36:47
//  Last edited:
//    13 Mar 2024, 17:53:32
//  Auto updated?
//    Yes
//
//  Description:
//!   Implements the core framework part of the JustAct framework. For
//!   more details, see the paper:
//


/***** LIBRARY *****/
/// Defines an agent in the system, at least abstractly.
pub trait Agent {}



/// Provides the abstraction for a message that is sent between agents.
pub trait Message {
    /// Defines how authors look like.
    type Author: Agent;

    /// Returns the author of the message.
    ///
    /// # Returns
    /// An author of type [`Self::Author`](Message::Author) that represents who sent the message.
    fn author(&self) -> Self::Author;
}

/// Defines a collection of messages.
pub trait MessageSet {
    /// The type of policy extracted from this set.
    type Policy: Policy;

    /// Returns some policy from the fragments contained in the messages of this set.
    ///
    /// # Returns
    /// A new policy of type [`Self::Policy`](MessageSet::Policy) that is the extracted policy.
    fn extract(&self) -> Self::Policy;
}



/// Defines how a Policy looks like.
pub trait Policy {
    /// Examines if this policy is valid.
    ///
    /// # Returns
    /// True if it is, or false if it isn't.
    fn is_valid(&self) -> bool;
}
