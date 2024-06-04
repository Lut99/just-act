//  LIB.rs
//    by Lut99
//
//  Created:
//    15 Apr 2024, 16:13:37
//  Last edited:
//    04 Jun 2024, 11:29:50
//  Auto updated?
//    Yes
//
//  Description:
//!   Provides an implementation of a simple demo environment that simulates agents without threads or any of that fancy jazz.
//

// Declare modules
pub mod agreements;
pub mod interface;
pub mod simulation;
pub mod statements;
pub mod times;

// Use some of it in the global namespace
pub use simulation::*;
