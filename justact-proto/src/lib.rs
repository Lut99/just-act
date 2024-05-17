//  LIB.rs
//    by Lut99
//
//  Created:
//    15 Apr 2024, 16:13:37
//  Last edited:
//    17 May 2024, 11:43:59
//  Auto updated?
//    Yes
//
//  Description:
//!   Provides an implementation of a simple demo environment that simulates agents without threads or any of that fancy jazz.
//

// Declare modules
pub mod global;
pub mod interface;
pub mod local;
pub mod set;
pub mod simulation;
pub mod sync;
pub mod wire;

// Use some of it in the global namespace
pub use simulation::*;
