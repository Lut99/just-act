//  GLOBAL.rs
//    by Lut99
//
//  Created:
//    13 May 2024, 14:43:47
//  Last edited:
//    17 May 2024, 14:39:03
//  Auto updated?
//    Yes
//
//  Description:
//!   Defines the sets that define an agent's synchronized knowledge.
//!
//!   On one hand, [`Agreements`] defines the set of stated messages that all
//!   agents must agree on.
//!
//!   On the other, [`Times`] allows agents to agree on a particular time in
//!   order to agree which of the agreements is currently valid.
//!
//!   [`GlobalView`] combines these two for a more succinct interface.
//

use std::error::Error;

use crate::set::{Map, Set};
use crate::{Agreement, Message};


/***** LIBRARY *****/
/// Defines the interface that agents use to inspect- and create agreements.
///
/// Conceptually, one can think of this as a _global_, _synchronized_ set that contains all the statements that _all agents agree on_.
///
/// Together with [`Times`], this forms the agents' shared knowledge of the system.
/// ```
pub trait Agreements: Map<Self::Agreement> {
    /// The type of agreements stored in this set.
    type Agreement: Agreement;
    /// The common notion of Time in the framework.
    type Time: Ord;
    /// The type of messages that can become agreements.
    type Message: Message;
    /// The type of error emitted by `agree`.
    type Error: Error;


    /// Agrees upon a statement in order to make it an agreement.
    ///
    /// This _always_ triggers _some kind_ of consensus mechanism, and may fail if consensus was not reached.
    /// An example of a super simple consensus mechanism is one agent deciding everything, and other agents
    /// always failing to create new agreements.
    ///
    /// After this function returns, one can be sure that all agents agree on the message being an agreement,
    /// insofar the underlying consensus mechanism guarantees this.
    ///
    /// # Arguments
    /// - `when`: Some `Self::Time` that determines for which time the agreement is valid (i.e., can be used as basis in Actions).
    /// - `msg`: Some `Self::Message` that will be agreed upon.
    ///
    /// # Errors
    /// This function is allowed to fail if no consensus was reached.
    fn agree(&mut self, when: Self::Time, msg: Self::Message) -> Result<(), Self::Error>;
}

/// Defines the interface that agents use to inspect- and create times.
///
/// Conceptually, one can think of this as a _global_, _synchronized_ set that contains all the steps in time that other agents agree on.
/// Importantly, this mainly focuses on which time is the _current_ time, and which times come before it.
///
/// Together with [`Agreements`], this forms the agents' shared knowledge of the system.
pub trait Times: Set<Self::Time> {
    /// The type of times stored in this set.
    type Time: Ord;
    /// The type of error emitted by `advance`.
    type Error: Error;


    /// Advances the current time.
    ///
    /// This _always_ triggers _some kind_ of consensus mechanism, and may fail if consensus was not reached.
    /// An example of a super simple consensus mechanism is one agent deciding everything, and other agents
    /// always failing to advance the time.
    ///
    /// # Arguments
    /// - `time`: Some `Time` that will become the new current time.
    ///
    /// # Errors
    /// This function is allowed to fail if no consensus was reached.
    fn advance(&mut self, time: Self::Time) -> Result<(), Self::Error>;

    /// Returns the current time.
    ///
    /// Since `Self::Time` implements [`Ord`], the latter can be used to discover which times come before the current time, and which times after.
    ///
    /// # Returns
    /// A `Self::Time` that represents the currently, globally agreed upon time.
    fn current(&self) -> Self::Time;
}
