//  WORLD.rs
//    by Lut99
//
//  Created:
//    15 Apr 2024, 15:11:55
//  Last edited:
//    15 Apr 2024, 16:15:15
//  Auto updated?
//    Yes
//
//  Description:
//!   Defines some more implementation-focused structs that determines how
//!   the world of agents looks like.
//

use std::future::Future;

use crate::message::MessageCollection;


/***** LIBRARY *****/
/// Defines how [`Agent`](crate::agent::Agent)s can communicate with each other.
///
/// In particular, it defines a set of [`Message`]s that the agent knows of. Or can learn.
///
/// Importantly, MessagePools may be scoped in that they give different information per-agent.
///
/// This version of the MessagePool is **synchronous**. See [`MessagePoolAsync`] for an `async` version.
pub trait MessagePool {
    /// The type of a meaningful collection of messages.
    type Collection<'s>: 's + MessageCollection
    where
        Self: 's;


    /// Allows an agent to extract the set of messages in this pool that this agent has access to.
    ///
    /// # Returns
    /// A set of messages that represents all the messages the agents knows of. This is _not_ a [`MessageSet`](crate::message::MessageSet), because there is no semantic meaning to them being combined.
    ///
    /// # Errors
    /// This function may error if it failed to construct this set, or update it.
    fn all<'s>(&'s mut self) -> Self::Collection<'s>;
}

/// Defines how [`Agent`](crate::agent::Agent)s can communicate with each other.
///
/// In particular, it defines a set of [`Message`]s that the agent knows of. Or can learn.
///
/// Importantly, MessagePools may be scoped in that they give different information per-agent.
///
/// This version of the MessagePool is **asynchronous**, i.e., `async`. See [`MessagePool`] for a non-`async` version.
pub trait MessagePoolAsync {
    /// The type of a meaningful collection of messages.
    type Collection<'s>: 's + MessageCollection
    where
        Self: 's;


    /// Allows an agent to extract the set of messages in this pool that this agent has access to.
    ///
    /// # Returns
    /// A set of messages that represents all the messages the agents knows of. This is _not_ a [`MessageSet`](crate::message::MessageSet), because there is no semantic meaning to them being combined.
    ///
    /// # Errors
    /// This function may error if it failed to construct this set, or update it.
    fn all_async<'s>(&'s mut self) -> impl 's + Future<Output = Self::Collection<'s>>;
}



/// Defines how [`Agent`](crate::agent::Agent)s communicate with users observing the simulations.
///
/// Depending on the agent, this is also an interface for providing user-input to the agent.
///
/// This version of the Interface is **synchronous**. See [`InterfaceAsync`] for an `async` version.
pub trait Interface {}

/// Defines how [`Agent`](crate::agent::Agent)s communicate with users observing the simulations.
///
/// Depending on the agent, this is also an interface for providing user-input to the agent.
///
/// This version of the Interface is **asynchronous**, i.e., `async`. See [`Interface`] for a non-`async` version.
pub trait InterfaceAsync {}
