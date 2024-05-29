//  AGENT.rs
//    by Lut99
//
//  Created:
//    15 Apr 2024, 14:52:41
//  Last edited:
//    27 May 2024, 17:32:02
//  Auto updated?
//    Yes
//
//  Description:
//!   Defines the [`Agent`]-trait, which defines how the simulator
//!   interacts with agents in the system.
//

use std::error::Error;

use crate::agreements::Agreements;
use crate::auxillary::Identifiable;
use crate::statements::Statements;
use crate::times::Times;


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
pub trait Agent: Identifiable {}

/// Extends an [`Agent`] with the capacity to think, i.e., do something.
///
/// This is effectively the trait that unifies everything into a concrete implementation. Its associated types force the implementer to get concrete about everything.
pub trait RationalAgent: Agent {
    /// The messages exchange by the agent.
    type Message;
    /// The target used by the agent to aim for other agents.
    type Target;
    /// The type of errors raised by reasoning.
    type Error: Error;


    /// Runs the underlying Agent code for one run.
    ///
    /// This effectively "runs" the agent itself. This allows it to inspect any statements, enactments, agreements and/or times, as well as create them.
    ///
    /// # Arguments
    /// - `agrs`: A set of globally synchronized [`Agreements`] for the agent to mutate (if consensus is reached) or not.
    /// - `times`: A set of globally synchronized [`Times`] for the agent to mutate (if consensus is reached) or not.
    ///
    /// # Returns
    /// An [`AgentPoll`]-type that determines what the runtime should do with this agent.
    ///
    /// # Errors
    /// Only fatal errors that prevent the Agent from participating in the system should cause this function to error. Examples are failures to properly attach to some remote registry or queue.
    fn poll(
        &mut self,
        agrs: impl Agreements<Message = Self::Message>,
        times: impl Times,
        stmts: impl Statements<Message = Self::Message, Target = Self::Target>,
    ) -> Result<AgentPoll, Self::Error>;
}
