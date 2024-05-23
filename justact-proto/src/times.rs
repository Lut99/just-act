//  TIMES.rs
//    by Lut99
//
//  Created:
//    23 May 2024, 17:36:27
//  Last edited:
//    23 May 2024, 17:42:35
//  Auto updated?
//    Yes
//
//  Description:
//!   Implements (various) global views on timestamps.
//

use std::error::Error;
use std::fmt::{Display, Formatter, Result as FResult};

use justact_core::times::{Times as JATimes, Timestamp};


/***** ERRORS *****/
/// Determines the possible errors for the [`TimesDicatator`] set.
#[derive(Debug)]
pub enum TimesDictatorError {
    /// The agent attempting to advance the time was not the dictator.
    NotTheDictator { agent: String, dictator: String },
}
impl Display for TimesDictatorError {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        use TimesDictatorError::*;
        match self {
            NotTheDictator { agent, dictator } => {
                write!(f, "Agent '{agent}' failed to advance the time because they are not the dictator ('{dictator}' is)")
            },
        }
    }
}
impl Error for TimesDictatorError {}





/***** LIBRARY *****/
/// Provides agents with a global view on the current time.
///
/// This variation synchronizes time if and only if it's a particular agent claiming it.
#[derive(Debug)]
pub struct TimesDictator {
    /// This agent
    agent:    String,
    /// The only agent allowed to make changes.
    dictator: String,

    /// The statements that this agent knows of.
    current: Timestamp,
    /// A queue of statements that this agent pushed.
    pub(crate) queue: Vec<Timestamp>,
}
impl JATimes for TimesDictator {
    type Error = TimesDictatorError;

    #[inline]
    fn current(&self) -> Timestamp {
        // See if the agent pushed a more recent one; else, take the one at creation time
        if let Some(time) = self.queue.last() { *time } else { self.current }
    }

    #[inline]
    fn advance_to(&mut self, timestamp: Timestamp) -> Result<(), Self::Error> {
        // Do not advance if we're not the dictator
        if self.agent == self.dictator {
            self.queue.push(timestamp);
            Ok(())
        } else {
            Err(TimesDictatorError::NotTheDictator { agent: self.agent.clone(), dictator: self.dictator.clone() })
        }
    }
}
