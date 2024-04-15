//  POLICY.rs
//    by Lut99
//
//  Created:
//    15 Apr 2024, 15:11:07
//  Last edited:
//    15 Apr 2024, 15:11:21
//  Auto updated?
//    Yes
//
//  Description:
//!   Defines the interface between the JustAct framework as a whole and a
//!   chosen policy language.
//


/***** LIBRARY *****/
/// Defines how a Policy looks like.
pub trait Policy {
    /// Examines if this policy is valid.
    ///
    /// # Returns
    /// True if it is, or false if it isn't.
    fn is_valid(&self) -> bool;
}
