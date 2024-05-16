//  GLOBAL.rs
//    by Lut99
//
//  Created:
//    13 May 2024, 19:28:04
//  Last edited:
//    16 May 2024, 17:43:31
//  Auto updated?
//    Yes
//
//  Description:
//!   Defines concrete implementations for globally synchronized sets and (possibly) their elements.
//

use std::error::Error;

use error_trace::trace;
use justact_core::global::{Agreements as _, Times as _};
use justact_core::{global as justact, Set as _};
use log::{error, warn};

use crate::set::Set;
use crate::sync::{SyncVote, Synchronizer};
use crate::wire::{Agreement, MessageSet};


/***** AUXILLARY *****/
/// Represents a single timestep in the simulation.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Timestamp(pub u128);





/***** LIBRARY *****/
/// Represents a sychronized view on the agreements that exist.
///
/// # Generics
/// - `S`: The [`Synchronizer`] used to synchronise any updates.
#[derive(Clone, Debug)]
pub struct Agreements<S> {
    /// The of (accepted) agreements.
    agrmnts:      Set<Agreement>,
    /// The synchronizer used to push updated.
    synchronizer: S,
}
impl<S> Agreements<S> {
    /// Constructor for the Agreements.
    ///
    /// # Arguments
    /// - `synchronizer`: The [`Synchronizer`] to use for reaching a consensus on which agreements are there.
    ///
    /// # Returns
    /// A new Agreements ready for use in the simulation.
    #[inline]
    pub fn new(synchronizer: S) -> Self { Self { agrmnts: Set::empty(), synchronizer } }

    /// Returns a new Agreements that is scoped to the given agent.
    ///
    /// # Arguments
    /// - `agent`: The identifier of the agent to scope to.
    ///
    /// # Returns
    /// A new [`AgreementsView`] that will only returns messages for the given scope.
    ///
    /// Note that, if the given `agent` is unknown, the resulting `AgreementsView` will not return any statements.
    #[inline]
    pub fn scope<'s>(&'s mut self, agent: &'static str) -> AgreementsView<'s, S> { AgreementsView { agrmnts: self, agent } }
}

/// Represents a particular agent's interpretation of the Agreements.
pub struct AgreementsView<'a, S> {
    /// The main Agreements they have a view on.
    agrmnts: &'a mut Agreements<S>,
    /// The agent from which we are viewing.
    agent:   &'static str,
}
impl<'a, S: Synchronizer<Agreement>> AgreementsView<'a, S> {
    /// Called by an agent to participate in the synchronization of the times.
    ///
    /// Agents should call this function regularly in order to participate in consent schemes and be up-to-date on the current time.
    ///
    /// # Arguments
    /// - `decision`: Some kind of closure that makes decisions in the chosen synchronization scheme `S`.
    pub fn synchronize<E: Error>(&mut self, decision: impl FnMut(&'static str, &Agreement) -> SyncVote<E>) -> Result<(), E> {
        // Run the vote, pull changes
        self.agrmnts.synchronizer.vote(self.agent, decision)?;
        while let Some(agrmnt) = self.agrmnts.synchronizer.process() {
            self.agrmnts.agrmnts.add(agrmnt);
        }
        Ok(())
    }
}
impl<'s, S: Synchronizer<Agreement>> justact_core::Set<Agreement> for AgreementsView<'s, S>
where
    S::Error: 'static,
{
    type Item<'s2> = <Set<Agreement> as justact_core::Set<Agreement>>::Item<'s2>
    where
        Self: 's2;
    type Iter<'s2> = <Set<Agreement> as justact_core::Set<Agreement>>::Iter<'s2> where Self: 's2;

    #[inline]
    fn add(&mut self, new_elem: Agreement) -> bool {
        // Else, treat as a normal agree
        if let Err(err) = self.agree(new_elem.timestamp, new_elem.msgs) {
            error!("{}", trace!(("Failed to agree on a message set"), err));
            warn!("The failure to synchronize agreements is not handled. Call `AgreementsView::agree()` to do so.");
        }
        false
    }

    #[inline]
    fn iter<'s2>(&'s2 self) -> Self::Iter<'s2> { self.agrmnts.agrmnts.iter() }

    #[inline]
    fn len(&self) -> usize { self.agrmnts.agrmnts.len() }
}
impl<'s, S: Synchronizer<Agreement>> justact::Agreements for AgreementsView<'s, S>
where
    S::Error: 'static,
{
    type Agreement = Agreement;
    type Time = Timestamp;
    type MessageSet = MessageSet<'static>;
    type Error = S::Error;

    #[inline]
    fn agree(&mut self, time: Self::Time, msgs: Self::MessageSet) -> Result<(), Self::Error> {
        // Simply start the voting, agents have to do the rest (through `Self::synchronize()`).
        self.agrmnts.synchronizer.start(self.agent, Agreement { timestamp: time, msgs })
    }
}



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
impl<'t, S: Synchronizer<Timestamp>> TimesView<'t, S> {
    /// Called by an agent to participate in the synchronization of the times.
    ///
    /// Agents should call this function regularly in order to participate in consent schemes and be up-to-date on the current time.
    ///
    /// # Arguments
    /// - `decision`: Some kind of closure that makes decisions in the chosen synchronization scheme `S`.
    pub fn synchronize<E: Error>(&mut self, decision: impl FnMut(&'static str, &Timestamp) -> SyncVote<E>) -> Result<(), E> {
        // Run the vote, pull changes
        self.times.synchronizer.vote(self.agent, decision)?;
        while let Some(new_current) = self.times.synchronizer.process() {
            self.times.current = new_current;
        }
        Ok(())
    }
}
impl<'t, S: Synchronizer<Timestamp>> justact_core::Set<Timestamp> for TimesView<'t, S>
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
impl<'t, S: Synchronizer<Timestamp>> justact::Times for TimesView<'t, S>
where
    S::Error: 'static,
{
    type Time = Timestamp;
    type Error = S::Error;

    #[inline]
    fn advance(&mut self, time: Self::Time) -> Result<(), Self::Error> {
        // Simply start the voting, agents have to do the rest (through `Self::synchronize()`).
        self.times.synchronizer.start(self.agent, time)
    }

    #[inline]
    fn current(&self) -> Self::Time { self.times.current }
}
