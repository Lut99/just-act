//  WORLD.rs
//    by Lut99
//
//  Created:
//    15 Apr 2024, 15:11:55
//  Last edited:
//    18 Apr 2024, 15:33:09
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

use crate::message::{Action, Message};
use crate::set::MessageSet;


/***** LIBRARY *****/
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
