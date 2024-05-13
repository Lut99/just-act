//  LOCAL.rs
//    by Lut99
//
//  Created:
//    18 Apr 2024, 15:27:35
//  Last edited:
//    13 May 2024, 15:42:11
//  Auto updated?
//    Yes
//
//  Description:
//!   Defines the sets that define an agent's partial knowledge.
//!
//!   On one hand, [`Statements`] allows an agent to inspect the messages it
//!   knows have been stated, and create stated messages itself.
//!
//!   On the other, [`Actions`] allows an agent to inspect enacted statements
//!   and create enactments itself.
//!
//!   [`LocalView`] combines these two for a more succinct interface.
//

use std::error::Error;

use crate::global::GlobalView;
use crate::set::Set;
use crate::wire::{Action, AuditableAction, Message};


/***** LIBRARY *****/
/// Combines both an [`Actions`] and a [`Statements`] into one convenient interface.
pub trait LocalView: Actions + Statements {}



/// Defines the interface that agents use to inspect- and create actions.
///
/// Conceptually, one can think of this as a _localized_ set that contains all the enacted statements that _a particular agent_ knows of.
///
/// Together with [`Statements`], this forms the agent's _potentially partial_ knowledge of the system.
/// ```
pub trait Actions: Set<Self::Enactment> {
    /// The type of actions stored in this set.
    type Enactment: Action;
    /// The type of actions that can become enacted actions.
    type Action: Action;
    /// Some type that allows agents to decide where their action goes.
    type Target;
    /// The type of error emitted by `enact`.
    type Error: Error;


    /// Enacts a statement.
    ///
    /// This _may_ cause the action to appear in this Actions, but only if the `target` includes ourselves.
    ///
    /// # Arguments
    /// - `target`: Some `Self::Target` that determines which other agents can see the message.
    /// - `act`: Some `Action` that carries the to-be-enacted statement.
    ///
    /// # Errors
    /// This function is allowed to fail if the broadcasting of the statement failed.
    fn enact(&mut self, target: Self::Target, msg: Self::Action) -> Result<(), Self::Error>;
}

/// Defines an extension to [`Actions`] that make them auditable.
pub trait AuditableActions: Actions
where
    Self::Enactment: AuditableAction,
{
    /// Something that explains why this Action did not succeed an audit.
    type Explanation;

    /// Audits all actions in this set.
    ///
    /// In particular, will check if all [justification](Action::justification())a are valid according to the policy language and all of its embedded messages have been stated.
    ///
    /// # Arguments
    /// - `global`: Some [`GlobalView`] that can be used to check which statements are agreements and what the current time is.
    /// - `stmts`: Some [`Statements`] that can be used to check which messages are stated.
    ///
    /// # Errors
    /// If one of the actions did not match the requirements of the audit, then an [`Action::Explanation`] is returned with why this is the case.
    fn audit<G, S>(&self, global: &G, stmts: &S) -> Result<(), Self::Explanation>
    where
        G: GlobalView,
        S: Statements;
}

/// Defines the interface that agents use to inspect- and create statements.
///
/// Conceptually, one can think of this as a _localized_ set that contains all the statements that _a particular agent_ knows of.
///
/// Together with [`Actions`], this forms the agent's _potentially partial_ knowledge of the system.
pub trait Statements: Set<Self::Statement> {
    /// The type of statements stored in this set.
    type Statement: Message;
    /// The type of messages that can become statements.
    type Message: Message;
    /// Some type that allows agents to decide where their message goes.
    type Target;
    /// The type of error emitted by `state`.
    type Error: Error;


    /// States a message.
    ///
    /// This _may_ cause the statement to appear in this Statements, but only if the `target` includes ourselves.
    ///
    /// # Arguments
    /// - `target`: Some `Self::Target` that determines which other agents can see the message.
    /// - `msg`: Some `Message` that will be stated.
    ///
    /// # Errors
    /// This function is allowed to fail if the broadcasting of the statement failed.
    fn state(&mut self, target: Self::Target, msg: Self::Message) -> Result<(), Self::Error>;
}
