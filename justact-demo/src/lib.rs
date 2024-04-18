//  LIB.rs
//    by Lut99
//
//  Created:
//    15 Apr 2024, 16:13:37
//  Last edited:
//    18 Apr 2024, 15:57:52
//  Auto updated?
//    Yes
//
//  Description:
//!   Provides an implementation of a simple demo environment that simulates agents without threads or any of that fancy jazz.
//

// Declare modules
pub mod interface;
pub mod lang;
pub mod set;
pub mod simulation;
pub mod statements;

// Use some of it in the global namespace
pub use simulation::*;
