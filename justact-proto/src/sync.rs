//  SYNC.rs
//    by Lut99
//
//  Created:
//    16 May 2024, 14:41:45
//  Last edited:
//    16 May 2024, 16:10:13
//  Auto updated?
//    Yes
//
//  Description:
//!   Defines some generic trait for implementing synchronization schemes
//!   between agents.
//

use std::collections::HashSet;
use std::convert::Infallible;
use std::error::Error;
use std::fmt::{Display, Formatter, Result as FResult};

use log::{debug, warn};
use stackvec::StackVec;


/***** ERRORS *****/
/// Describes why an agent couldn't influence the decision in a [`Dictatorship`].
#[derive(Debug)]
pub struct DictatorshipError {
    /// The agent attempting to vote.
    pub agent:    &'static str,
    /// The agent holding all the power which, crucially, wasn't the requesting `agent`.
    pub dictator: &'static str,
}
impl Display for DictatorshipError {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        write!(f, "Agent '{}' could not vote because they are not the dicatator (agent '{}' is).", self.agent, self.dictator)
    }
}
impl Error for DictatorshipError {}





/***** AUXILLARY *****/
/// Describes what agents can decide in [`Synchronizer::sync()`].
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SyncVote<E> {
    /// The agent is OK with accepting the given element.
    Ok,
    /// The agent is not OK with accepting the given element.
    Deny,
    /// The agent needs more time to decide.
    Pending,
    /// The agent encountered a fatal error.
    Err(E),
}





/***** LIBRARY *****/
/// Defines how synchronization schemes look like in general.
///
/// # Generics
/// - `T`: This Synchronizer will synchronize a set of given type `T`.
pub trait Synchronizer<T> {
    /// The type of errors returned by this Synchronizer.
    type Error: Error;


    /// Some agent proposes to add a new element to the synchronized set.
    ///
    /// When this function returns, the synchronization process has started. The initiating agent can then monitor progress by calling `poll()`.
    ///
    /// Note that this function can assume that the requesting agent automatically accepts.
    ///
    /// # Arguments
    /// - `agent`: Some identification that allows us to know who it is. For simplicity's sake, we assume agents can't lie about this.
    /// - `elem`: Some new proposed `T` to add.
    ///
    /// # Errors
    /// This function can error if the process somehow failed to start.
    fn start(&mut self, agent: &'static str, elem: T) -> Result<(), Self::Error>;

    /// Does this agent's part of synchronization.
    ///
    /// When an agent `poll()`s the synchronizer and sees [`SyncPoll::Pending`], it should call this function with a closure of its choice. That will influence how the synchronization is going.
    ///
    /// Note that the synchronization is not applied until it completes (i.e., `retrieve()` returns [`Some`]). If the agent called it without a synchronization requiring action, then its `decision` is simply not called.
    ///
    /// # Arguments
    /// - `agent`: Some identification that allows us to know who it is. For simplicity's sake, we assume agents can't lie about this.
    /// - `decision`: Some closure that takes the agent's decision in the synchronization. The agent can inspect the submitting agent and the to-be-added element, and then return a [`SyncVote`] which either:
    ///   - Indicates the agent agrees with the synchronization ([`SyncVote::Ok`]);
    ///   - Indicates the agent does _not_ agree with the synchronization ([`SyncVote::Deny`]);
    ///   - Indicates the agent needs more time to decide ([`SyncVote::Pending`]); or
    ///   - Indicates the agent met a fatal error ([`SyncVote::Err`]).
    fn vote<E: Error>(&mut self, agent: &'static str, decision: impl FnMut(&'static str, &T) -> SyncVote<E>) -> Result<(), E>;

    /// Function called by the synchronized set to pull any pending accepted elements.
    ///
    /// # Returns
    /// [`Some(T)`] if there's an element, or else [`None`].
    fn process(&mut self) -> Option<T>;
}



/// Implements a synchronization scheme where one agent is always the one indicating everything, and others aren't.
pub struct Dictatorship<T> {
    /// The agent holding supreme power for all decisions in this synchronization scheme.
    dictator: &'static str,
    /// The list of things pushed by the dictator waiting for other users to pickup.
    decided:  StackVec<64, T>,
}
impl<T> Dictatorship<T> {
    /// Constructor for the Dictatorship synchronization scheme.
    ///
    /// This scheme gives supreme power to one agent.
    ///
    /// # Arguments
    /// - `dictator`: The agent that gets to have supreme power over whatever is being synchronized.
    ///
    /// # Returns
    /// A new Dictatorship that places the `dictator` in power.
    #[inline]
    pub fn new(dictator: &'static str) -> Self { Self { dictator, decided: StackVec::new() } }
}
impl<T: Clone> Synchronizer<T> for Dictatorship<T> {
    type Error = DictatorshipError;

    #[inline]
    fn start(&mut self, agent: &'static str, elem: T) -> Result<(), Self::Error> {
        // If the agent is not the Dictator, they cannot start
        if agent != self.dictator {
            return Err(DictatorshipError { agent, dictator: self.dictator });
        }

        // Add the decision, which everybody agrees with because it's the dicatator saying it (yay).
        self.decided.push(elem);
        Ok(())
    }

    #[inline]
    fn vote<E: Error>(&mut self, _agent: &'static str, _decision: impl FnMut(&'static str, &T) -> SyncVote<E>) -> Result<(), E> {
        /* Dicatatorships don't need voting. */
        warn!("Called Dictatorship::vote(). Obviously, that doesn't do anything.");
        Ok(())
    }

    #[inline]
    fn process(&mut self) -> Option<T> {
        // Simply pop any
        self.decided.pop()
    }
}

/// Implements a synchronization scheme where agents decide by majority vote.
pub struct Democracy<T> {
    /// The minimum number of votes necessary to call it a democracy.
    majority: usize,
    /// The total number of participants in the democracy.
    total:    usize,
    /// The things of which votes are in progress.
    pending:  StackVec<64, (&'static str, HashSet<&'static str>, HashSet<&'static str>, T)>,
    /// The list of things which have won the votes.
    decided:  StackVec<64, T>,
}
impl<T> Democracy<T> {
    /// Constructor for the Democracy synchronization scheme.
    ///
    /// This scheme requires that at least the given number of agents vote in favour of something being accepted.
    ///
    /// # Arguments
    /// - `minimum_majority`: The number of agents once, when reached, counts as a majority.
    /// - `total`: The total number of agents.
    ///
    /// # Returns
    /// A new Democracy that requires `minimum_majority` votes in order to reach consensus on a change.
    #[inline]
    pub fn new(minimum_majority: usize, total: usize) -> Self {
        Self { majority: minimum_majority, total, pending: StackVec::new(), decided: StackVec::new() }
    }
}
impl<T> Synchronizer<T> for Democracy<T> {
    type Error = Infallible;

    #[inline]
    fn start(&mut self, agent: &'static str, elem: T) -> Result<(), Self::Error> {
        // Push this as pending, with one vote for this agent
        debug!("Agent '{agent}' starts a synchronization round in a Democracy scheme");
        self.pending.push((agent, HashSet::from([agent]), HashSet::new(), elem));
        Ok(())
    }

    #[inline]
    fn vote<E: Error>(&mut self, agent: &'static str, mut decision: impl FnMut(&'static str, &T) -> SyncVote<E>) -> Result<(), E> {
        // See if there's any decisions to take
        for (initiator, yays, nays, elem) in self.pending.iter_mut().filter(|(_, yays, nays, _)| !yays.contains(&agent) && !nays.contains(&agent)) {
            // Let the agent decide on a vote
            let vote: SyncVote<E> = decision(initiator, elem);
            debug!("Agent '{agent}' voted '{vote:?}' for the round initiated by '{initiator}'");

            // Process it
            match vote {
                SyncVote::Ok => yays.insert(agent),
                SyncVote::Deny => nays.insert(agent),
                SyncVote::Pending => continue,
                SyncVote::Err(err) => return Err(err),
            };
        }

        // Move any votes with enough yays to decided, and any votes with enough nays away entirely
        self.pending.retain_drain(|(initiator, yays, nays, elem)| {
            if yays.len() >= self.majority {
                debug!("The vote initiated by agent '{initiator}' PASSED with {} votes", yays.len());
                self.decided.push(elem);
                None
            } else if nays.len() > self.total - self.majority {
                debug!("The vote initiated by agent '{initiator}' FAILED with {} votes", yays.len());
                None
            } else {
                Some((initiator, yays, nays, elem))
            }
        });

        // Done
        Ok(())
    }

    #[inline]
    fn process(&mut self) -> Option<T> {
        // Simply pop what has been decided upon
        self.decided.pop()
    }
}
