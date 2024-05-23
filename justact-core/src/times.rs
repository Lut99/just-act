//  TIMES.rs
//    by Lut99
//
//  Created:
//    21 May 2024, 16:34:11
//  Last edited:
//    23 May 2024, 11:54:50
//  Auto updated?
//    Yes
//
//  Description:
//!   Implements the globally synchronized set of timestamps, including
//!   which one is the current one.
//

use std::error::Error;
use std::fmt::{Display, Formatter, Result as FResult};


/***** LIBRARY *****/
/// Defines what it means for something to be a timestamp.
///
/// This implementation is provided, as we expect it to be the same across implementations.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Timestamp(pub u128);
impl Display for Timestamp {
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> FResult { write!(f, "{}", self.0) }
}



/// Implements an abstract set of timestamps, including information about the current one.
///
/// This is a _globally synchronized_ set, meaning that the framework requires agents to be in
/// agreement at all times about this set's contents.
pub trait Times {
    /// The (set of) error(s) that may occur when running [`Self::advance_to()`](Times::advance_to()).
    type Error: Error;


    /// Returns the timestamp which is the current one.
    ///
    /// Any information about past or future can be deduced from which is the current timestamp, plus [`Timestamp`]'s [`Ord`]-implementation.
    ///
    /// # Returns
    /// The current [`Timestamp`].
    fn current(&self) -> Timestamp;

    /// Pushes a new timestamp to be the current one.
    ///
    /// # Arguments
    /// - `timestamp`: The new [`Timestamp`] to advance to.
    ///
    /// # Errors
    /// Whether this succeeds or not is entirely based on the underlying implementation. In
    /// particular, this function might fail if agents failed to reach consensus, not all agents
    /// could be synchronized, etc.
    ///
    /// However, one should assume that _if_ this function fails, the current time has not
    /// advanced.
    fn advance_to(&mut self, timestamp: Timestamp) -> Result<(), Self::Error>;
}
