//  LIB.rs
//    by Lut99
//
//  Created:
//    13 Mar 2024, 15:36:47
//  Last edited:
//    23 May 2024, 13:39:20
//  Auto updated?
//    Yes
//
//  Description:
//!   Implements the core framework part of the JustAct framework. For
//!   more details, see the paper:
//


// Declare submodules
pub mod agents;
pub mod agreements;
pub mod auxillary;
pub mod iter;
pub mod set;
pub mod statements;
pub mod system;
pub mod times;

// Bring it into this namespace
pub use system::SystemView;
