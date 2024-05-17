//  MOD.rs
//    by Lut99
//
//  Created:
//    17 May 2024, 14:20:44
//  Last edited:
//    17 May 2024, 14:54:13
//  Auto updated?
//    Yes
//
//  Description:
//!   Defines helper modules for the `paperX.rs`-examples.
//

// Declare submodules (agents, mostly)
pub mod administrator;

// Imports
use std::convert::Infallible;

pub use administrator::Administrator;
use justact_core::auxillary::Identifiable;
use justact_core::{Actions, Agent, AgentPoll, Agreements, RationalAgent, Statements, Times};
use justact_prototype::local::Target;
use justact_prototype::wire::{Action, Message};


/***** LIBRARY *****/
/// An agent abstracting over the other types.
#[derive(Debug)]
pub enum AbstractAgent {
    Administrator(Administrator),
    // Amy(Amy),
    // Anton(Anton),
    // Consortium(Consortium),
}
impl Identifiable for AbstractAgent {
    type Id = &'static str;

    #[inline]
    fn id(&self) -> &Self::Id {
        match self {
            Self::Administrator(a) => a.id(),
            // Self::Amy(a) => a.id(),
            // Self::Anton(a) => a.id(),
            // Self::Consortium(c) => c.id(),
        }
    }
}
impl Agent for AbstractAgent {}
impl RationalAgent for AbstractAgent {
    type Enactment = Action;
    type Action = Action;
    type Statement = Message;
    type Message = Message;
    type Target = Target;
    type Error = Infallible;

    fn poll<G, L>(&mut self, global: &mut G, local: &mut L) -> Result<AgentPoll, Self::Error>
    where
        G: Agreements + Times,
        L: Actions<Enactment = Self::Enactment, Action = Self::Action, Target = Self::Target>
            + Statements<Statement = Self::Statement, Message = Self::Message, Target = Self::Target>,
        Self::Error: From<<L as Actions>::Error> + From<<L as Statements>::Error>,
    {
        match self {
            Self::Administrator(a) => a.poll(global, local),
            // Self::Amy(a) => a.poll(pool),
            // Self::Anton(a) => a.poll(pool),
            // Self::Consortium(c) => c.poll(pool),
        }
    }
}
impl From<Administrator> for AbstractAgent {
    #[inline]
    fn from(value: Administrator) -> Self { Self::Administrator(value) }
}
// impl From<Amy> for AbstractAgent {
//     #[inline]
//     fn from(value: Amy) -> Self { Self::Amy(value) }
// }
// impl From<Anton> for AbstractAgent {
//     #[inline]
//     fn from(value: Anton) -> Self { Self::Anton(value) }
// }
// impl From<Consortium> for AbstractAgent {
//     #[inline]
//     fn from(value: Consortium) -> Self { Self::Consortium(value) }
// }
