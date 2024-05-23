//  LIB.rs
//    by Lut99
//
//  Created:
//    13 Mar 2024, 15:36:47
//  Last edited:
//    23 May 2024, 11:27:11
//  Auto updated?
//    Yes
//
//  Description:
//!   Implements the core framework part of the JustAct framework. For
//!   more details, see the paper:
//


// Declare submodules
// pub mod agent;
pub mod agreements;
pub mod auxillary;
// pub mod global;
pub mod iter;
// pub mod local;
// pub mod policy;
pub mod set;
pub mod statements;
pub mod system;
pub mod times;
// pub mod wire;

// Bring it into this namespace
// pub use agent::{Agent, AgentPoll, RationalAgent};
// pub use global::{Agreements, Times};
// pub use local::{Actions, Statements};
// pub use policy::{ExtractablePolicy, Policy};
// pub use set::{Map, Set};
// pub use wire::{Action, Agreement, Message, MessageSet};
pub use system::SystemView;
