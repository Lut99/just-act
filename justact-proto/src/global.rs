//  GLOBAL.rs
//    by Lut99
//
//  Created:
//    13 May 2024, 19:28:04
//  Last edited:
//    13 May 2024, 19:29:10
//  Auto updated?
//    Yes
//
//  Description:
//!   Defines concrete implementations for globally synchronized sets and (possibly) their elements.
//


/***** LIBRARY *****/
/// Represents a single time in the simulation.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Timestamp(pub u128);
