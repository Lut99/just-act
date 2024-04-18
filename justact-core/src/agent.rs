//  AGENT.rs
//    by Lut99
//
//  Created:
//    15 Apr 2024, 14:52:41
//  Last edited:
//    18 Apr 2024, 17:23:51
//  Auto updated?
//    Yes
//
//  Description:
//!   Defines the [`Agent`]-trait, which defines how the simulator
//!   interacts with agents in the system.
//

use std::error::Error;
use std::future::Future;

use crate::statements::{Statements, Stating, StatingAsync};


/***** AUXILLARY *****/
/// Allows an [`Agent`] to decide what happens to it after it has been polled.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum AgentPoll {
    /// The agent lives on, nothing happens.
    Alive,
    /// The agent should be terminated. Its work has been completed.
    Dead,
}
impl Default for AgentPoll {
    #[inline]
    fn default() -> Self { Self::Alive }
}





/***** LIBRARY *****/
/// Defines an agent in the system, at least abstractly.
pub trait Agent {
    /// The type of error emitted by functions for this type of Agent.
    type Error: Error;
    /// Some type of identifier that can be used to recognize this agent.
    type Identifier;


    /// Returns some identifyer for this agent that can be used to uniquely recognize it within the system.
    ///
    /// # Returns
    /// A `Self::Identifier` that represents this agent in terms of identification.
    fn id(&self) -> Self::Identifier;
}



/// Extends an [`Agent`] with the capacity to think, i.e., do something.
///
/// This version does so **synchronously**. See [`RationalAgentAsync`] for a version that is `async`.
pub trait RationalAgent: Agent {
    /// The pool that agents use to state messages / enact actions, and inspect what others did.
    type Statements: Statements + Stating;
    /// The interface type that allows the Agent to communicate with users.
    type Interface;


    /// Runs the underlying Agent code for one run.
    ///
    /// This effectively "runs" the agent itself. This receives any events on the incoming queue and outputs any on the outgoing.
    ///
    /// # Arguments
    /// - `pool`: Some [`MessagePool`] that the agent uses to learn of new messages and/or emits new messages on. Essentially, acts as a way for the agent to interact with other agents.
    /// - `interface`: An [`Interface`] that the agent uses to show logs, receives user input and/or produces errors. Essentially, acts as a way for the agent to interact with users.
    ///
    /// # Returns
    /// An [`AgentPoll`]-type that determines what the runtime should do with this agent next.
    ///
    /// # Errors
    /// Only fatal errors that prevent the Agent from participating in the system should cause this function to error. Examples are failures to emit errors to the `interface`.
    fn poll(&mut self, pool: &mut Self::Statements, interface: &mut Self::Interface) -> Result<AgentPoll, Self::Error>;
}

/// Extends an [`Agent`] with the capacity to think, i.e., do something.
///
/// This version does so **asynchronously**, i.e., `async`. See [`RationalAgent`] for a version that is not `async`.
pub trait RationalAgentAsync: Agent {
    /// The pool that agents use to state messages / enact actions, and inspect what others did.
    type StatementsAsync: Statements + StatingAsync;
    /// The interface type that allows the Agent to communicate with users.
    type InterfaceAsync;


    /// Runs the underlying Agent code for one run.
    ///
    /// This effectively "runs" the agent itself. This receives any events on the incoming queue and outputs any on the outgoing.
    ///
    /// # Arguments
    /// - `pool`: Some [`MessagePool`] that the agent uses to learn of new messages and/or emits new messages on. Essentially, acts as a way for the agent to interact with other agents.
    /// - `interface`: An [`Interface`] that the agent uses to show logs, receives user input and/or produces errors. Essentially, acts as a way for the agent to interact with users.
    ///
    /// # Returns
    /// An [`AgentPoll`]-type that determines what the runtime should do with this agent next.
    ///
    /// # Errors
    /// Only fatal errors that prevent the Agent from participating in the system should cause this function to error. Examples are failures to emit errors to the `interface`.
    fn poll_async<'s, 'p, 'i>(
        &'s mut self,
        pool: &'p mut Self::StatementsAsync,
        interface: &'i mut Self::InterfaceAsync,
    ) -> impl 's + 'p + 'i + Future<Output = Result<AgentPoll, Self::Error>>;
}
