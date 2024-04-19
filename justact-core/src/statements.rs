//  STATEMENTS.rs
//    by Lut99
//
//  Created:
//    18 Apr 2024, 15:27:35
//  Last edited:
//    19 Apr 2024, 14:06:52
//  Auto updated?
//    Yes
//
//  Description:
//!   Defines [`Statements`], the interface for agents to emit messages,
//!   enact actions and reason about what others stated.
//

use std::error::Error;
use std::future::Future;

use crate::message::{Action, Message};


/***** LIBRARY *****/
/// Defines the interface that agents use to inspect stated messages.
pub trait Statements {
    /// A type that is used to refer to messages.
    type Id;
    /// A type that explains why a particular action did not succeed an audit.
    type Explanation<'s>: 's
    where
        Self: 's;

    /// The type of messages that can be stated.
    type Statement<'s>: 's + Message
    where
        Self: 's;
    /// An iterator for stated `Self::Statement`s.
    type StatedIter<'s>: 's + Iterator
    where
        Self: 's;
    /// An iterator for enacted actions.
    type EnactedIter<'s>: 's + Iterator
    where
        Self: 's;


    /// Audits all actions [enacted](Statements::enact()) that this agent can reach.
    ///
    /// # Errors
    /// If any of the actions does not succeed the audit, then this returns a `Self::Error` with which one and why this is the case.
    fn audit<'s>(&'s self) -> Result<(), Self::Explanation<'s>>;


    /// Checks if a message with the given ID has been stated.
    ///
    /// # Arguments
    /// - `id`: Some identifier to check for if it has been stated.
    ///
    /// # Returns
    /// True if it has, or false if it hasn't.
    #[inline]
    fn is_stated(&self, id: Self::Id) -> bool { self.get_stated(id).is_some() }

    /// Returns a stated message by ID.
    ///
    /// # Arguments
    /// - `id`: Some identifier to check for if it has been stated.
    ///
    /// # Returns
    /// The referred message if it was stated and this agent had access to it, or else [`None`].
    fn get_stated<'s>(&'s self, id: Self::Id) -> Option<Self::Statement<'s>>;

    /// Returns the number of (stated) messages, or at least, the number that are reachable by this agent.
    ///
    /// # Returns
    /// The number of (stated) messages.
    fn n_stated(&self) -> usize;

    /// Returns whether any messages are stated at all, as far as this agent knows.
    ///
    /// # Returns
    /// True if [`Statements::n_stated()`] is 0, or false otherwise.
    #[inline]
    fn any_stated(&self) -> bool { self.n_stated() > 0 }

    /// Returns an iterator over all stated messages.
    ///
    /// # Returns
    /// A `Self::ActionIter` that iterates over the statements to which this agent has access.
    fn stated<'s>(&'s self) -> Self::StatedIter<'s>;

    /// Returns the number of (enacted) actions, or at least, the number that are reachable by this agent.
    ///
    /// # Returns
    /// The number of (enacted) actions.
    fn n_enacted(&self) -> usize;

    /// Returns whether any actions are enacted at all, as far as this agent knows.
    ///
    /// # Returns
    /// True if [`Statements::n_enacted()`] is 0, or false otherwise.
    #[inline]
    fn any_enacted(&self) -> bool { self.n_enacted() > 0 }

    /// Returns an iterator over all enacted actions.
    ///
    /// # Returns
    /// A `Self::ActionIter` that iterates over the enacted actions to which this agent has access.
    fn enacted<'s>(&'s self) -> Self::EnactedIter<'s>;
}



/// Defines the interface that agents use to state messages.
///
/// This version implements a synchronous version of the Stating. If you want to use `async` instead, refer to [`StatingAsync`].
pub trait Stating {
    /// The type of errors emitted by this pool.
    type Error: Error;
    /// The type of scope that allows agents to specify where their messages end up.
    type Scope;

    /// The type of messages that can be transferred with this pool.
    type Message: Message;
    /// The type of actions that can be transferred with this pool.
    type Action: Action;


    /// Allows an agent to broadcast a message into the world.
    ///
    /// The Agent should provide a scope to determine where the message ends up. For some implementations, only one scope may be possible.
    ///
    /// # Arguments
    /// - `msg`: Some [`Message`] to emit. Should already be signed by the agent.
    /// - `scope`: Some kind of `Self::Scope` that determines where the message ends up.
    ///
    /// # Errors
    /// This function may error if it failed to send the message somehow.
    fn state(&mut self, msg: Self::Message, scope: Self::Scope) -> Result<(), Self::Error>;

    /// Allows an agent to broadcast a justified effect into the world.
    ///
    /// The Agent should provide a scope to determine where the message ends up. For some implementations, only one scope may be possible.
    ///
    /// # Arguments
    /// - `act`: Some [`Action`] to enact. Should already be signed and justified by the agent.
    /// - `scope`: Some kind of `Self::Scope` that determines where the message ends up.
    ///
    /// # Errors
    /// This function may error if it failed to send the message somehow.
    fn enact(&mut self, act: Self::Action, scope: Self::Scope) -> Result<(), Self::Error>;
}



/// Defines the interface that agents use to emit messages, enact actions and reason about what others stated.
///
/// This version implements an `async` version of the Stating. If you don't need to use `async`, consider using [`Stating`] instead.
pub trait StatingAsync {
    /// The type of errors emitted by this pool.
    type Error: Error;
    /// The type of scope that allows agents to specify where their messages end up.
    type Scope;

    /// The type of messages that can be transferred with this pool.
    type Message: Message;
    /// The type of actions that can be transferred with this pool.
    type Action: Action;


    /// Allows an agent to broadcast a message into the world.
    ///
    /// The Agent should provide a scope to determine where the message ends up. For some implementations, only one scope may be possible.
    ///
    /// # Arguments
    /// - `msg`: Some [`Message`] to emit. Should already be signed by the agent.
    /// - `scope`: Some kind of `Self::Scope` that determines where the message ends up.
    ///
    /// # Errors
    /// This function may error if it failed to send the message somehow.
    fn state_async<'s>(&'s mut self, msg: Self::Message, scope: Self::Scope) -> impl 's + Future<Output = Result<(), Self::Error>>;

    /// Allows an agent to broadcast a justified effect into the world.
    ///
    /// The Agent should provide a scope to determine where the message ends up. For some implementations, only one scope may be possible.
    ///
    /// # Arguments
    /// - `act`: Some [`Action`] to enact. Should already be signed and justified by the agent.
    /// - `scope`: Some kind of `Self::Scope` that determines where the message ends up.
    ///
    /// # Errors
    /// This function may error if it failed to send the message somehow.
    fn enact_async<'s>(&'s mut self, act: Self::Action, scope: Self::Scope) -> impl 's + Future<Output = Result<(), Self::Error>>;
}
