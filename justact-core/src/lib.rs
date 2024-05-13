//  LIB.rs
//    by Lut99
//
//  Created:
//    13 Mar 2024, 15:36:47
//  Last edited:
//    13 May 2024, 15:40:43
//  Auto updated?
//    Yes
//
//  Description:
//!   Implements the core framework part of the JustAct framework. For
//!   more details, see the paper:
//


// Declare submodules
pub mod agent;
pub mod auxillary;
pub mod global;
pub mod local;
pub mod policy;
pub mod set;
pub mod wire;

// Bring it into this namespace
pub use agent::{Agent, AgentPoll, RationalAgent};
pub use global::{Agreements, GlobalView, Times};
pub use local::{Actions, AuditableActions, LocalView, Statements};
pub use policy::{ExtractablePolicy, Policy};
pub use set::Set;
pub use wire::{Action, Agreement, AuditableAction, Message, MessageSet};
