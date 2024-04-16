//  LIB.rs
//    by Lut99
//
//  Created:
//    15 Apr 2024, 16:13:37
//  Last edited:
//    16 Apr 2024, 11:07:44
//  Auto updated?
//    Yes
//
//  Description:
//!   Provides an implementation of a simple demo environment that simulates agents without threads or any of that fancy jazz.
//

// Declare modules
pub mod interface;
pub mod message;
pub mod pool;
pub mod simulation;

// Use some of it in the global namespace
pub use simulation::*;
