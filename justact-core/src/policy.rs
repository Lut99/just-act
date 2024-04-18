//  POLICY.rs
//    by Lut99
//
//  Created:
//    15 Apr 2024, 15:11:07
//  Last edited:
//    18 Apr 2024, 14:33:12
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
    /// The type of the object that explains why this policy is invalid.
    type Explanation;


    /// Checks if this policy is valid.
    ///
    /// This can differ wildy per policy language what this means.
    ///
    /// # Errors
    /// If the policy is not valid, then this returns some kind of `Self::Explanation` explaining why it wasn't.
    fn check_validity(&self) -> Result<(), Self::Explanation>;
}
