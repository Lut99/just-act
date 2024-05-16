//  GLOBAL.rs
//    by Lut99
//
//  Created:
//    13 May 2024, 19:28:04
//  Last edited:
//    16 May 2024, 16:38:27
//  Auto updated?
//    Yes
//
//  Description:
//!   Defines concrete implementations for globally synchronized sets and (possibly) their elements.
//

use error_trace::trace;
use justact_core::global as justact;
use justact_core::global::Times as _;
use log::{error, warn};

use crate::sync::Synchronizer;


/***** AUXILLARY *****/
/// Represents a single timestep in the simulation.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Timestamp(pub u128);





/***** LIBRARY *****/
/// Represents a sychronized view on the times that exist.
///
/// To be precise, does actually not do this. Instead, it synchronizes which [`Timestamp`] is the current one. It being ordered then allows agents to decide if something is in the past or the future.
///
/// # Generics
/// - `S`: The [`Synchronizer`] used to synchronise any updates.
#[derive(Clone, Debug)]
pub struct Times<S> {
    /// The current time.
    current:      Timestamp,
    /// The synchronizer used to push updated.
    synchronizer: S,
}
impl<S> Times<S> {
    /// Constructor for the Times.
    ///
    /// # Arguments
    /// - `initial`: The initial timestap to start at.
    /// - `synchronizer`: The [`Synchronizer`] to use for reaching a consensus on the next time.
    ///
    /// # Returns
    /// A new Times ready for use in the simulation.
    #[inline]
    pub fn new(initial: Timestamp, synchronizer: S) -> Self { Self { current: initial, synchronizer } }

    /// Returns a new Times that is scoped to the given agent.
    ///
    /// # Arguments
    /// - `agent`: The identifier of the agent to scope to.
    ///
    /// # Returns
    /// A new [`TimesView`] that will only returns messages for the given scope.
    ///
    /// Note that, if the given `agent` is unknown, the resulting `TimesView` will not return any statements.
    #[inline]
    pub fn scope<'s>(&'s mut self, agent: &'static str) -> TimesView<'s, S> { TimesView { times: self, agent } }
}

/// Represents a particular agent's interpretation of the Times.
pub struct TimesView<'t, S> {
    /// The main Times they have a view on.
    times: &'t mut Times<S>,
    /// The agent from which we are viewing.
    agent: &'static str,
}
impl<'s, S: Synchronizer<Timestamp>> justact_core::Set<Timestamp> for TimesView<'s, S>
where
    S::Error: 'static,
{
    type Item<'s2> = Timestamp
    where
        Self: 's2;
    type Iter<'s2> = std::option::IntoIter<Timestamp> where Self: 's2;

    #[inline]
    fn add(&mut self, new_elem: Timestamp) -> bool {
        // Catch a simple case if the times haven't changed
        if self.current() == new_elem {
            return true;
        }

        // Else, treat as a normal advance
        if let Err(err) = self.advance(new_elem) {
            error!("{}", trace!(("Failed to advance the time"), err));
            warn!("The failure to synchronize time is not handled. Call `TimesView::advance()` to do so.");
        }
        false
    }

    #[inline]
    fn iter<'s2>(&'s2 self) -> Self::Iter<'s2> { Some(self.times.current).into_iter() }

    #[inline]
    fn len(&self) -> usize { 1 }
}
impl<'s, S: Synchronizer<Timestamp>> justact::Times for TimesView<'s, S>
where
    S::Error: 'static,
{
    type Time = Timestamp;
    type Error = S::Error;

    #[inline]
    fn advance(&mut self, time: Self::Time) -> Result<(), Self::Error> {
        // Simply start the voting
        self.times.synchronizer.start(self.agent, time)
    }

    #[inline]
    fn current(&self) -> Self::Time { self.times.current }
}
