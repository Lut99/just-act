//  TIMES.rs
//    by Lut99
//
//  Created:
//    26 Apr 2024, 11:31:44
//  Last edited:
//    26 Apr 2024, 11:41:25
//  Auto updated?
//    Yes
//
//  Description:
//!   Defines the framework's notion of the set of all [`Times`].
//

use std::error::Error;


/***** LIBRARY *****/
/// Defines the set of times that are known at runtime.
///
/// This allows one to dynamically decide which agreement applies at which moment.
pub trait Times {
    /// The type of each individual time.
    type Time;
    /// The type of errors returned by the various functions.
    type Error: Error;


    /// Checks if the given time is Known.
    ///
    /// # Arguments
    /// - `time`: Some [`Self::Time`](Times::Time) that we check to see if it was once applicable.
    ///
    /// # Returns
    /// True if it was, or false if it wasn't.
    ///
    /// # Errors
    /// This function should error if it failed to update itself.
    fn contains(&mut self, time: &Self::Time) -> Result<bool, Self::Error>;

    /// Returns the _current_ time.
    ///
    /// # Returns
    /// A [`Self::Time`](Times::Time) which represents the time that should be assigned to new actions.
    ///
    /// # Errors
    /// This function should error if it failed to update itself.
    fn current(&mut self) -> Result<&Self::Time, Self::Error>;

    /// Moves the current time to the given one.
    ///
    /// # Arguments
    /// - `time`: Some [`Self::Time`](Times::Time) to move to.
    fn advance(&self, time: Self::Time);
}
