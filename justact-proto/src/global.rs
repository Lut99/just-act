//  GLOBAL.rs
//    by Lut99
//
//  Created:
//    13 May 2024, 19:28:04
//  Last edited:
//    21 May 2024, 15:46:00
//  Auto updated?
//    Yes
//
//  Description:
//!   Defines concrete implementations for globally synchronized sets and (possibly) their elements.
//

use std::cell::RefCell;
use std::error::Error;
use std::fmt::{Display, Formatter, Result as FResult};
use std::rc::Rc;

use error_trace::trace;
use justact_core::auxillary::Identifiable;
use justact_core::global::{Agreements as _, Times as _};
use justact_core::{global as justact, Set as _};
use log::{error, warn};

use crate::interface::Interface;
use crate::set::{set_passthrough_impl, Set};
use crate::sync::{SyncVote, Synchronizer};
use crate::wire::{Agreement, Message};


/***** AUXILLARY *****/
/// Represents a single timestep in the simulation.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Timestamp(pub u128);
impl Display for Timestamp {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult { write!(f, "{}", self.0) }
}





/***** LIBRARY *****/
/// Implements a unification of [`Agreements`] and [`Times`].
///
/// # Generics
/// - `S1`: The [`Synchronizer`] used to reach consensus on new agreements.
/// - `S2`: The [`Synchronizer`] used to reach consensus on new times.
#[derive(Clone, Debug)]
pub struct GlobalState<S1, S2> {
    /// The agreements-part.
    pub agrmnts: Agreements<S1>,
    /// The times-part.
    pub times:   Times<S2>,
}
impl<S1, S2> GlobalState<S1, S2> {
    /// Constructor for the GlobalState.
    ///
    /// # Arguments
    /// - `initial`: The initial [`Timestamp`] to populate the internal [`Times`] with.
    /// - `sync1`: The [`Synchronizer`] used to reach consensus on new [`Agreements`].
    /// - `sync2`: The [`Synchronizer`] used to reach consensus on new [`Times`].
    /// - `interface`: Some shared [`Interface`] that we use to log nice messages about when agents publish actions.
    ///
    /// # Returns
    /// A new GlobalState ready for use in the simulation.
    #[inline]
    pub fn new(initial: Timestamp, sync1: S1, sync2: S2, interface: Rc<RefCell<Interface>>) -> Self {
        Self { agrmnts: Agreements::new(sync1, interface.clone()), times: Times::new(initial, sync2, interface) }
    }

    /// Returns a new GlobalState that is scoped to the given agent.
    ///
    /// # Arguments
    /// - `agent`: The identifier of the agent to scope to.
    ///
    /// # Returns
    /// A new [`GlobalView`] that will only returns messages for the given scope.
    ///
    /// Note that, if the given `agent` is unknown, the resulting `GlobalView` will not return any statements.
    #[inline]
    pub fn scope<'s>(&'s mut self, agent: &'static str) -> GlobalView<'s, S1, S2> {
        GlobalView { agrmnts: self.agrmnts.scope(agent), times: self.times.scope(agent) }
    }
}

/// Implements a per-agent view on the [`GlobalState`].
#[derive(Debug)]
pub struct GlobalView<'s, S1, S2> {
    /// All agreements in the universe.
    agrmnts: AgreementsView<'s, S1>,
    /// All times in the universe.
    times:   TimesView<'s, S2>,
}
set_passthrough_impl!(
    impl<'s, S1: Synchronizer<Agreement>, S2> Set<Agreement> (as AgreementsView<'s, S1>) for GlobalView.agrmnts where S1::Error: 'static;
    impl<'s, S1: Synchronizer<Agreement>, S2> Map<Agreement> for GlobalView.agrmnts where S1::Error: 'static;
);
set_passthrough_impl!(
    impl<'s, S1, S2: Synchronizer<Timestamp>> Set<Timestamp> (as TimesView<'s, S2>) for GlobalView.times where S2::Error: 'static;
);
impl<'s, S1: Synchronizer<Agreement>, S2> justact_core::Agreements for GlobalView<'s, S1, S2>
where
    S1::Error: 'static,
{
    type Agreement = <AgreementsView<'s, S1> as justact_core::Agreements>::Agreement;
    type Time = <AgreementsView<'s, S1> as justact_core::Agreements>::Time;
    type Message = <AgreementsView<'s, S1> as justact_core::Agreements>::Message;
    type Error = <AgreementsView<'s, S1> as justact_core::Agreements>::Error;

    #[inline]
    fn agree(&mut self, time: Self::Time, msg: Self::Message) -> Result<(), Self::Error> { self.agrmnts.agree(time, msg) }
}
impl<'s, S1, S2: Synchronizer<Timestamp>> justact_core::Times for GlobalView<'s, S1, S2>
where
    S2::Error: 'static,
{
    type Time = <TimesView<'s, S2> as justact_core::Times>::Time;
    type Error = <TimesView<'s, S2> as justact_core::Times>::Error;

    #[inline]
    fn advance(&mut self, time: Self::Time) -> Result<(), Self::Error> { self.times.advance(time) }

    #[inline]
    fn current(&self) -> Self::Time { self.times.current() }
}



/// Represents a sychronized view on the agreements that exist.
///
/// # Generics
/// - `S`: The [`Synchronizer`] used to synchronise any updates.
#[derive(Clone, Debug)]
pub struct Agreements<S> {
    /// The of (accepted) agreements.
    pub agrmnts:      Set<Agreement>,
    /// The synchronizer used to push updated.
    pub synchronizer: S,
    /// The interface we're using to log things nicely.
    pub interface:    Rc<RefCell<Interface>>,
}
impl<S> Agreements<S> {
    /// Constructor for the Agreements.
    ///
    /// # Arguments
    /// - `synchronizer`: The [`Synchronizer`] to use for reaching a consensus on which agreements are there.
    /// - `interface`: Some shared [`Interface`] that we use to log nice messages about when agents publish actions.
    ///
    /// # Returns
    /// A new Agreements ready for use in the simulation.
    #[inline]
    pub fn new(synchronizer: S, interface: Rc<RefCell<Interface>>) -> Self { Self { agrmnts: Set::empty(), synchronizer, interface } }

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
    pub fn scope<'s>(&'s mut self, agent: &'static str) -> AgreementsView<'s, S> { AgreementsView { agrmnts: self, agent: Some(agent) } }

    /// Returns a special scope that reflects _all_ agents.
    ///
    /// Kind of like a view for the system as a whole.
    ///
    /// # Returns
    /// A new [`AgreementsView`] that cannot be used to vote.
    #[inline]
    pub fn unspecific(&mut self) -> AgreementsView<S> { AgreementsView { agrmnts: self, agent: None } }
}

/// Represents a particular agent's interpretation of the Agreements.
#[derive(Debug)]
pub struct AgreementsView<'a, S> {
    /// The main Agreements they have a view on.
    agrmnts: &'a mut Agreements<S>,
    /// The agent from which we are viewing. If [`None`], then this view is "unspecific", meaning that it can only be used to view but not to vote.
    agent:   Option<&'static str>,
}
impl<'a, S: Synchronizer<Agreement>> AgreementsView<'a, S> {
    /// Called by an agent to participate in the synchronization of the times.
    ///
    /// Agents should call this function regularly in order to participate in consent schemes and be up-to-date on the current time.
    ///
    /// # Arguments
    /// - `decision`: Some kind of closure that makes decisions in the chosen synchronization scheme `S`.
    pub fn synchronize<E: Error>(&mut self, decision: impl FnMut(&'static str, &Agreement) -> SyncVote<E>) -> Result<(), E> {
        // Crash if we're unspecific but used to vote
        let agent: &'static str = match self.agent {
            Some(agent) => agent,
            None => panic!("Cannot synchronize a non-agent-specific AgreementsView"),
        };

        // Run the vote, pull changes
        self.agrmnts.synchronizer.vote(agent, decision)?;
        while let Some(agrmnt) = self.agrmnts.synchronizer.process() {
            self.agrmnts.interface.borrow().log_agree(&agrmnt);
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
        if let Err(err) = self.agree(new_elem.timestamp, new_elem.msg) {
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
impl<'s, S: Synchronizer<Agreement>> justact_core::Map<Agreement> for AgreementsView<'s, S>
where
    S::Error: 'static,
{
    #[inline]
    fn get(&self, id: &<Agreement as Identifiable>::Id) -> Option<&Agreement> { self.agrmnts.agrmnts.get(id) }
}
impl<'s, S: Synchronizer<Agreement>> justact::Agreements for AgreementsView<'s, S>
where
    S::Error: 'static,
{
    type Agreement = Agreement;
    type Time = Timestamp;
    type Message = Message;
    type Error = S::Error;

    #[inline]
    fn agree(&mut self, time: Self::Time, msg: Self::Message) -> Result<(), Self::Error> {
        // Crash if we're unspecific but used to vote
        let agent: &'static str = match self.agent {
            Some(agent) => agent,
            None => panic!("Cannot synchronize a non-agent-specific TimesView"),
        };

        // Simply start the voting, agents have to do the rest (through `Self::synchronize()`).
        let agrmnt: Agreement = Agreement { timestamp: time, msg };
        self.agrmnts.interface.borrow().log_agree_start(agent, &agrmnt);
        self.agrmnts.synchronizer.start(agent, agrmnt)
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
    pub current:      Timestamp,
    /// The synchronizer used to push updated.
    pub synchronizer: S,
    /// The interface we're using to log things nicely.
    pub interface:    Rc<RefCell<Interface>>,
}
impl<S> Times<S> {
    /// Constructor for the Times.
    ///
    /// # Arguments
    /// - `initial`: The initial timestap to start at.
    /// - `synchronizer`: The [`Synchronizer`] to use for reaching a consensus on the next time.
    /// - `interface`: Some shared [`Interface`] that we use to log nice messages about when agents publish actions.
    ///
    /// # Returns
    /// A new Times ready for use in the simulation.
    #[inline]
    pub fn new(initial: Timestamp, synchronizer: S, interface: Rc<RefCell<Interface>>) -> Self { Self { current: initial, synchronizer, interface } }

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
    pub fn scope<'s>(&'s mut self, agent: &'static str) -> TimesView<'s, S> { TimesView { times: self, agent: Some(agent) } }

    /// Returns a special scope that reflects _all_ agents.
    ///
    /// Kind of like a view for the system as a whole.
    ///
    /// # Returns
    /// A new [`TimesView`] that cannot be used to vote.
    #[inline]
    pub fn unspecific(&mut self) -> TimesView<S> { TimesView { times: self, agent: None } }
}

/// Represents a particular agent's interpretation of the Times.
#[derive(Debug)]
pub struct TimesView<'t, S> {
    /// The main Times they have a view on.
    times: &'t mut Times<S>,
    /// The agent from which we are viewing. If [`None`], then this view is "unspecific", meaning that it can only be used to view but not to vote.
    agent: Option<&'static str>,
}
impl<'t, S: Synchronizer<Timestamp>> TimesView<'t, S> {
    /// Called by an agent to participate in the synchronization of the times.
    ///
    /// Agents should call this function regularly in order to participate in consent schemes and be up-to-date on the current time.
    ///
    /// # Arguments
    /// - `decision`: Some kind of closure that makes decisions in the chosen synchronization scheme `S`.
    pub fn synchronize<E: Error>(&mut self, decision: impl FnMut(&'static str, &Timestamp) -> SyncVote<E>) -> Result<(), E> {
        // Crash if we're unspecific but used to vote
        let agent: &'static str = match self.agent {
            Some(agent) => agent,
            None => panic!("Cannot synchronize a non-agent-specific TimesView"),
        };

        // Run the vote, pull changes
        self.times.synchronizer.vote(agent, decision)?;
        while let Some(new_current) = self.times.synchronizer.process() {
            self.times.interface.borrow().log_advance(new_current);
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
        // Crash if we're unspecific but used to vote
        let agent: &'static str = match self.agent {
            Some(agent) => agent,
            None => panic!("Cannot synchronize a non-agent-specific TimesView"),
        };

        // Simply start the voting, agents have to do the rest (through `Self::synchronize()`).
        self.times.interface.borrow().log_advance_start(agent, time);
        self.times.synchronizer.start(agent, time)
    }

    #[inline]
    fn current(&self) -> Self::Time { self.times.current }
}
