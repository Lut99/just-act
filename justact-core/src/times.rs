//  TIMES.rs
//    by Lut99
//
//  Created:
//    21 May 2024, 16:34:11
//  Last edited:
//    21 May 2024, 16:47:13
//  Auto updated?
//    Yes
//
//  Description:
//!   Implements the globally synchronized set of timestamps, including
//!   which one is the current one.
//

use std::error::Error;


/***** LIBRARY *****/
/// Defines what it means for something to be a timestamp (i.e., a [`Times::Time`]).
pub trait Time: Ord {}



/// Implements an abstract set of timestamps, including information about the current one.
///
/// This is a _globally synchronized_ set, meaning that the framework requires agents to be in
/// agreement at all times about this set's contents.
pub trait Times {
    /// The type of timestamp that is contained within this set.
    type Time: Time;
    /// The (set of) error(s) that may occur when running [`Self::advance_to()`](Times::advance_to()).
    type Error: Error;


    /// Returns the timestamp which is the current one.
    ///
    /// Any information about past or future can be deduced from which is the current timestamp, plus [`Self::Time`](Times::Time)'s [`Ord`]-implementation.
    ///
    /// # Returns
    /// A reference to the current timestamp.
    fn current(&self) -> &Self::Time;

    /// Pushes a new timestamp to be the current one.
    ///
    /// # Arguments
    /// - `timestamp`: The new timestamp to advance to.
    ///
    /// # Errors
    /// Whether this succeeds or not is entirely based on the underlying implementation. In
    /// particular, this function might fail if agents failed to reach consensus, not all agents
    /// could be synchronized, etc.
    ///
    /// However, one should assume that _if_ this function fails, the current time has not
    /// advanced.
    fn advance_to(&mut self, timestamp: Self::Time) -> Result<(), Self::Error>;
}
