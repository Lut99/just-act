//  MOD.rs
//    by Lut99
//
//  Created:
//    17 May 2024, 14:20:44
//  Last edited:
//    27 May 2024, 18:02:25
//  Auto updated?
//    Yes
//
//  Description:
//!   Defines helper modules for the `paperX.rs`-examples.
//

// Declare submodules (agents, mostly)
pub mod administrator;
pub mod amy;
pub mod anton;
pub mod consortium;

// Imports
use std::convert::Infallible;

pub use administrator::Administrator;
pub use amy::Amy;
pub use anton::Anton;
pub use consortium::Consortium;
use justact_core::agents::{Agent, AgentPoll, RationalAgent};
use justact_core::agreements::Agreements;
use justact_core::auxillary::Identifiable;
use justact_core::statements::Statements;
use justact_core::times::Times;
use justact_prototype::statements::{Message, Target};


/***** LIBRARY *****/
/// An agent abstracting over the other types.
#[derive(Debug)]
pub enum AbstractAgent {
    Administrator(Administrator),
    Amy(Amy),
    Anton(Anton),
    Consortium(Consortium),
}
impl Identifiable for AbstractAgent {
    type Id = str;

    #[inline]
    fn id(&self) -> &Self::Id {
        match self {
            Self::Administrator(a) => a.id(),
            Self::Amy(a) => a.id(),
            Self::Anton(a) => a.id(),
            Self::Consortium(c) => c.id(),
        }
    }
}
impl Agent for AbstractAgent {}
impl RationalAgent for AbstractAgent {
    type Message = Message;
    type Target = Target;
    type Error = Infallible;

    fn poll(
        &mut self,
        agrmnts: impl Agreements<Message = Self::Message>,
        times: impl Times,
        stmts: impl Statements<Message = Self::Message, Target = Self::Target>,
    ) -> Result<AgentPoll, Self::Error> {
        match self {
            Self::Administrator(a) => a.poll(agrmnts, times, stmts),
            Self::Amy(a) => a.poll(agrmnts, times, stmts),
            Self::Anton(a) => a.poll(agrmnts, times, stmts),
            Self::Consortium(c) => c.poll(agrmnts, times, stmts),
        }
    }
}
impl From<Administrator> for AbstractAgent {
    #[inline]
    fn from(value: Administrator) -> Self { Self::Administrator(value) }
}
impl From<Amy> for AbstractAgent {
    #[inline]
    fn from(value: Amy) -> Self { Self::Amy(value) }
}
impl From<Anton> for AbstractAgent {
    #[inline]
    fn from(value: Anton) -> Self { Self::Anton(value) }
}
impl From<Consortium> for AbstractAgent {
    #[inline]
    fn from(value: Consortium) -> Self { Self::Consortium(value) }
}
