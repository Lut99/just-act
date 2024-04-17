//  WORLD.rs
//    by Lut99
//
//  Created:
//    15 Apr 2024, 15:11:55
//  Last edited:
//    17 Apr 2024, 16:30:43
//  Auto updated?
//    Yes
//
//  Description:
//!   Defines some more implementation-focused structs that determines how
//!   the world of agents looks like.
//

use std::error::Error;
use std::fmt::Display;
use std::future::Future;

use crate::collection::Collection;
use crate::message::{Action, Message, MessageSet};


/***** LIBRARY *****/
/// Defines how [`Agent`](crate::agent::Agent)s can communicate with each other.
///
/// In particular, it defines a set of [`Message`]s that the agent knows of. Or can learn.
///
/// Importantly, MessagePools may be scoped in that they give different information per-agent.
///
/// This version of the MessagePool is **synchronous**. See [`MessagePoolAsync`] for an `async` version.
pub trait MessagePool {
    /// The type of errors emitted by this pool.
    type Error: Error;
    /// The type of scope that allows agents to specify where their messages end up.
    type Scope;

    /// The type of messages that can be transferred with this pool.
    type Message: Message;
    /// The type of a meaningful collection of messages.
    type MessageCollection<'s>: 's + Collection<Self::Message>
    where
        Self: 's;

    /// The type of actions that can be transferred with this pool.
    type Action: Action;
    /// The type of a collection of actions.
    type ActionCollection<'s>: 's + Collection<Self::Action>
    where
        Self: 's;


    /// Allows an agent to broadcast a message into the world.
    ///
    /// The Agent should provide a scope to determine where the message ends up. For some implementations, only one scope may be possible.
    ///
    /// # Arguments
    /// - `message`: Some [`Message`] to emit. Should already be signed by the agent.
    /// - `scope`: Some kind of `Self::Scope` that determines where the message ends up.
    ///
    /// # Errors
    /// This function may error if it failed to send the message somehow.
    fn emit(&mut self, message: Self::Message, scope: Self::Scope) -> Result<(), Self::Error>;

    /// Allows an agent to broadcast a justified effect into the world.
    ///
    /// The Agent should provide a scope to determine where the message ends up. For some implementations, only one scope may be possible.
    ///
    /// # Arguments
    /// - `act`: Some [`Action`] to emit. Should already be signed and justified by the agent.
    /// - `scope`: Some kind of `Self::Scope` that determines where the message ends up.
    ///
    /// # Errors
    /// This function may error if it failed to send the message somehow.
    fn enact(&mut self, act: Self::Action, scope: Self::Scope) -> Result<(), Self::Error>;

    /// Allows an agent to extract the set of messages in this pool that this agent has access to.
    ///
    /// # Returns
    /// A set of messages that represents all the messages the agents knows of. This is _not_ a [`MessageSet`](crate::message::MessageSet), because there is no semantic meaning to them being combined.
    ///
    /// # Errors
    /// This function may error if it failed to construct this set, or update it.
    fn all_messages<'s>(&'s mut self) -> Self::MessageCollection<'s>;

    /// Allows an agent to extract all actions in this pool that this agent has access to.
    ///
    /// # Returns
    /// A set of actions that represents all the actions the agents knows of.
    ///
    /// # Errors
    /// This function may error if it failed to construct this set, or update it.
    fn all_actions<'s>(&'s mut self) -> Self::ActionCollection<'s>;
}

/// Defines how [`Agent`](crate::agent::Agent)s can communicate with each other.
///
/// In particular, it defines a set of [`Message`]s that the agent knows of. Or can learn.
///
/// Importantly, MessagePools may be scoped in that they give different information per-agent.
///
/// This version of the MessagePool is **asynchronous**, i.e., `async`. See [`MessagePool`] for a non-`async` version.
pub trait MessagePoolAsync {
    /// The type of errors emitted by this pool.
    type Error: Error;
    /// The type of scope that allows agents to specify where their messages end up.
    type Scope;

    /// The type of messages that can be transferred with this pool.
    type Message: Message;
    /// The type of a meaningful collection of messages.
    type MessageCollection<'s>: 's + Collection<Self::Message>
    where
        Self: 's;

    /// The type of actions that can be transferred with this pool.
    type Action: Action;
    /// The type of a collection of actions.
    type ActionCollection<'s>: 's + Collection<Self::Action>
    where
        Self: 's;


    /// Allows an agent to broadcast a message into the world.
    ///
    /// The Agent should provide a scope to determine where the message ends up. For some implementations, only one scope may be possible.
    ///
    /// # Arguments
    /// - `message`: Some [`Message`] to emit. Should already be signed by the agent.
    /// - `scope`: Some kind of `Self::Scope` that determines where the message ends up.
    ///
    /// # Errors
    /// This function may error if it failed to send the message somehow.
    fn emit_async<'s>(&'s mut self, message: Self::Message, scope: Self::Scope) -> impl 's + Future<Output = Result<(), Self::Error>>;

    /// Allows an agent to broadcast a justified effect into the world.
    ///
    /// The Agent should provide a scope to determine where the message ends up. For some implementations, only one scope may be possible.
    ///
    /// # Arguments
    /// - `act`: Some [`Action`] to emit. Should already be signed and justified by the agent.
    /// - `scope`: Some kind of `Self::Scope` that determines where the message ends up.
    ///
    /// # Errors
    /// This function may error if it failed to send the message somehow.
    fn enact_async<'s>(&'s mut self, act: Self::Action, scope: Self::Scope) -> impl 's + Future<Output = Result<(), Self::Error>>;

    /// Allows an agent to extract the set of messages in this pool that this agent has access to.
    ///
    /// # Returns
    /// A set of messages that represents all the messages the agents knows of. This is _not_ a [`MessageSet`](crate::message::MessageSet), because there is no semantic meaning to them being combined.
    ///
    /// # Errors
    /// This function may error if it failed to construct this set, or update it.
    fn all_messages_async<'s>(&'s mut self) -> impl 's + Future<Output = Result<Self::MessageCollection<'s>, Self::Error>>;

    /// Allows an agent to extract all actions in this pool that this agent has access to.
    ///
    /// # Returns
    /// A set of actions that represents all the actions the agents knows of.
    ///
    /// # Errors
    /// This function may error if it failed to construct this set, or update it.
    fn all_actions_async<'s>(&'s mut self) -> impl 's + Future<Output = Result<Self::ActionCollection<'s>, Self::Error>>;
}



/// Defines how [`Agent`](crate::agent::Agent)s communicate with users observing the simulations.
///
/// Depending on the agent, this is also an interface for providing user-input to the agent.
///
/// This version of the Interface is **synchronous**. See [`InterfaceAsync`] for an `async` version.
pub trait Interface {
    /// The error type returned by this Interface's functions.
    type Error: Error;

    /// Allows an agent to log something generic to the system or user.
    ///
    /// # Arguments
    /// - `id`: Some identifier by which the user may recognize this agent.
    /// - `msg`: The message to log.
    ///
    /// # Errors
    /// This function can error if it failed to log the statement.
    fn log(&mut self, id: &str, msg: impl Display) -> Result<(), Self::Error>;

    /// Allows an agent to log that they emit a [`Message`] to the system or user.
    ///
    /// # Arguments
    /// - `id`: Some identifier by which the user may recognize this agent.
    /// - `msg`: The [`Message`] to log.
    ///
    /// # Errors
    /// This function can error if it failed to log the statement.
    fn log_emit<M>(&mut self, id: &str, msg: &M) -> Result<(), Self::Error>
    where
        M: Display + Message,
        M::Author: Display,
        M::Id: Display;

    /// Allows an agent to log that they enacted an [`Action`] to the system or user.
    ///
    /// # Arguments
    /// - `id`: Some identifier by which the user may recognize this agent.
    /// - `act`: The [`Action`] to log.
    ///
    /// # Errors
    /// This function can error if it failed to log the statement.
    fn log_enact<'a, A>(&mut self, id: &str, act: &'a A) -> Result<(), Self::Error>
    where
        A: Display + Action,
        <A::Message as Message>::Author: Display,
        <A::Message as Message>::Id: Display,
        <A::MessageSet as MessageSet>::Policy<'a>: Display;

    /// Allows an agent to log something bad to the system or user.
    ///
    /// # Arguments
    /// - `id`: Some identifier by which the user may recognize this agent.
    /// - `msg`: The message to log.
    ///
    /// # Errors
    /// This function can error if it failed to log the statement.
    fn error(&mut self, id: &str, msg: impl Display) -> Result<(), Self::Error>;
}

/// Defines how [`Agent`](crate::agent::Agent)s communicate with users observing the simulations.
///
/// Depending on the agent, this is also an interface for providing user-input to the agent.
///
/// This version of the Interface is **asynchronous**, i.e., `async`. See [`Interface`] for a non-`async` version.
pub trait InterfaceAsync {
    /// The error type returned by this Interface's functions.
    type Error: Error;

    /// Allows an agent to log something to the system or user.
    ///
    /// # Arguments
    /// - `id`: Some identifier by which the user may recognize this agent.
    /// - `msg`: The message to log.
    ///
    /// # Errors
    /// This function can error if it failed to log the statement.
    fn log_async<'s, 'i, 'm>(&'s mut self, id: &'i str, msg: impl 'm + Display) -> impl 's + 's + 'm + Future<Output = Result<(), Self::Error>>;

    /// Allows an agent to log that they emit a [`Message`] to the system or user.
    ///
    /// # Arguments
    /// - `id`: Some identifier by which the user may recognize this agent.
    /// - `msg`: The [`Message`] to log.
    ///
    /// # Errors
    /// This function can error if it failed to log the statement.
    fn log_emit_async<'s, 'i, 'm, M>(&'s mut self, id: &'i str, msg: &'m M) -> impl 's + 'i + 'm + Future<Output = Result<(), Self::Error>>
    where
        M: Display + Message,
        M::Author: Display,
        M::Id: Display;

    /// Allows an agent to log that they enacted an [`Action`] to the system or user.
    ///
    /// # Arguments
    /// - `id`: Some identifier by which the user may recognize this agent.
    /// - `act`: The [`Action`] to log.
    ///
    /// # Errors
    /// This function can error if it failed to log the statement.
    fn log_enact_async<'s, 'i, 'a, A>(&'s mut self, id: &'i str, act: &'a A) -> impl 's + 'i + 'a + Future<Output = Result<(), Self::Error>>
    where
        A: Display + Action,
        <A::Message as Message>::Author: Display,
        <A::Message as Message>::Id: Display,
        <A::MessageSet as MessageSet>::Policy<'a>: Display;

    /// Allows an agent to log something bad to the system or user.
    ///
    /// # Arguments
    /// - `id`: Some identifier by which the user may recognize this agent.
    /// - `msg`: The message to log.
    ///
    /// # Errors
    /// This function can error if it failed to log the statement.
    fn error_async<'s1, 's2, 'd>(
        &'s1 mut self,
        id: &'s2 str,
        msg: impl 'd + Display,
    ) -> impl 's1 + 's2 + 'd + Future<Output = Result<(), Self::Error>>;
}
