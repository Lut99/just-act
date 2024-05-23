//  AGENT.rs
//    by Lut99
//
//  Created:
//    15 Apr 2024, 14:52:41
//  Last edited:
//    23 May 2024, 13:42:08
//  Auto updated?
//    Yes
//
//  Description:
//!   Defines the [`Agent`]-trait, which defines how the simulator
//!   interacts with agents in the system.
//

use std::error::Error;

use crate::auxillary::Identifiable;


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
pub trait Agent: Identifiable<'static> {}

/// Extends an [`Agent`] with the capacity to think, i.e., do something.
///
/// This is effectively the trait that unifies everything into a concrete implementation. Its associated types force the implementer to get concrete about everything.
pub trait RationalAgent: Agent {
    /// The type of errors raised by reasoning.
    type Error: Error;


    /// Runs the underlying Agent code for one run.
    ///
    /// This effectively "runs" the agent itself. This allows it to inspect any statements, enactments, agreements and/or times, as well as create them.
    ///
    /// # Returns
    /// An [`AgentPoll`]-type that determines what the runtime should do with this agent.
    ///
    /// # Errors
    /// Only fatal errors that prevent the Agent from participating in the system should cause this function to error. Examples are failures to properly attach to some remote registry or queue.
    fn poll(&mut self) -> Result<AgentPoll, Self::Error>;
}
