//  POOL.rs
//    by Lut99
//
//  Created:
//    15 Apr 2024, 16:16:19
//  Last edited:
//    21 May 2024, 15:12:05
//  Auto updated?
//    Yes
//
//  Description:
//!   Provides an implementation for the JustAct version of a
//!   [`MessagePool`](justact_core::world::MessagePool).
//

use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::convert::Infallible;
use std::rc::Rc;

use bit_vec::BitVec;
use justact_core::auxillary::Identifiable;
use justact_core::local::{Actions as _, Statements as _};
use justact_core::ExtractablePolicy;

use crate::global::AgreementsView;
use crate::interface::Interface;
use crate::set::set_passthrough_impl;
use crate::sync::Synchronizer;
use crate::wire::{Action, Agreement, Message};


/***** ITERATORS *****/
/// Iterates over the agent-specific actions in an [`ActionsView`].
#[derive(Debug)]
pub struct ActionsIter<'s, 'a> {
    /// The [`ActionsView`] we're iterating over.
    acts: &'s ActionsView<'a>,
    /// The current index.
    i:    usize,
}
impl<'s, 'a> Iterator for ActionsIter<'s, 'a> {
    type Item = &'s Action;

    fn next(&mut self) -> Option<Self::Item> {
        while self.i < self.acts.acts.acts.len() {
            self.i += 1;

            // Check if should return this one
            if self.acts.agent.map(|a| self.acts.acts.masks.get(a).map(|m| m.get(self.i - 1).unwrap_or(false)).unwrap_or(false)).unwrap_or(true) {
                return Some(&self.acts.acts.acts[self.i - 1]);
            }
        }

        // There are no more statements left
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let count: usize = self
            .acts
            .agent
            .map(|a| self.acts.acts.masks.get(a).map(|m| m.iter().filter(|b| *b).count()).unwrap_or(0))
            .unwrap_or(self.acts.acts.acts.len());
        (count, Some(count))
    }
}

/// Iterates over the agent-specific statements in a [`StatementsView`].
#[derive(Debug)]
pub struct StatementsIter<'s1, 's2> {
    /// The [`StatementsView`] we're iterating over.
    stmts: &'s1 StatementsView<'s2>,
    /// The current index.
    i:     usize,
}
impl<'s1, 's2> Iterator for StatementsIter<'s1, 's2> {
    type Item = &'s1 Message;

    fn next(&mut self) -> Option<Self::Item> {
        while self.i < self.stmts.stmts.msgs.len() {
            self.i += 1;

            // Check if should return this one
            if self.stmts.agent.map(|a| self.stmts.stmts.masks.get(a).map(|m| m.get(self.i - 1).unwrap_or(false)).unwrap_or(false)).unwrap_or(true) {
                return Some(&self.stmts.stmts.msgs[self.i - 1]);
            }
        }

        // There are no more statements left
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let count: usize = self
            .stmts
            .agent
            .map(|a| self.stmts.stmts.masks.get(a).map(|m| m.iter().filter(|b| *b).count()).unwrap_or(0))
            .unwrap_or(self.stmts.stmts.msgs.len());
        (count, Some(count))
    }
}





/***** AUXILLARY *****/
/// Determines the possible targets that agents can send messages to for this [`Statements`].
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Target {
    /// Send it to everybody.
    All,
    /// Send it to a particular agent with this ID.
    Agent(&'static str),
}
impl Target {
    /// Checks if the given agent is targeted by this Target.
    ///
    /// # Arguments
    /// - `agent`: The agent to check for a match.
    ///
    /// # Returns
    /// True if this Target targets the given `agent`, else false.
    #[inline]
    pub fn matches(&self, agent: &'static str) -> bool {
        match self {
            Self::All => true,
            Self::Agent(a) => *a == agent,
        }
    }
}

/// Explains why an audit of all [`Action`]s in a [`Statements`] failed.
#[derive(Debug)]
pub struct AuditExplanation<E1, E2> {
    /// The action that failed to audit
    pub act:  Action,
    /// The reason why this action failed.
    pub expl: crate::wire::AuditExplanation<E1, E2>,
}





/***** LIBRARY *****/
/// Implements a unification of [`Actions`] and [`Statements`].
#[derive(Clone, Debug)]
pub struct LocalState {
    /// The actions-part.
    pub acts:  Actions,
    /// The statements-part.
    pub stmts: Statements,
}
impl LocalState {
    /// Constructor for the LocalState.
    ///
    /// # Arguments
    /// - `interface`: Some shared [`Interface`] that we use to log nice messages about when agents publish actions.
    ///
    /// # Returns
    /// A new LocalState ready for use in the simulation.
    #[inline]
    pub fn new(interface: Rc<RefCell<Interface>>) -> Self { Self { acts: Actions::new(interface.clone()), stmts: Statements::new(interface) } }

    /// Returns a new LocalState that is scoped to the given agent.
    ///
    /// # Arguments
    /// - `agent`: The identifier of the agent to scope to.
    ///
    /// # Returns
    /// A new [`LocalView`] that will only returns messages for the given scope.
    ///
    /// Note that, if the given `agent` is unknown, the resulting `LocalView` will not return any statements.
    #[inline]
    pub fn scope<'s>(&'s mut self, agent: &'static str) -> LocalView<'s> {
        LocalView { acts: self.acts.scope(agent), stmts: self.stmts.scope(agent) }
    }
}

/// Implements a per-agent view on the [`LocalState`].
#[derive(Debug)]
pub struct LocalView<'s> {
    /// All actions in the universe.
    acts:  ActionsView<'s>,
    /// All statements in the universe.
    stmts: StatementsView<'s>,
}
set_passthrough_impl!(
    impl<'s> Set<Action> (as ActionsView<'s>) for LocalView.acts;
    impl<'s> Map<Action> for LocalView.acts;
);
set_passthrough_impl!(
    impl<'s> Set<Message> (as StatementsView<'s>) for LocalView.stmts;
    impl<'s> Map<Message> for LocalView.stmts;
);
impl<'s> justact_core::Actions for LocalView<'s> {
    type Enactment = <ActionsView<'s> as justact_core::Actions>::Enactment;
    type Action = <ActionsView<'s> as justact_core::Actions>::Action;
    type Target = <ActionsView<'s> as justact_core::Actions>::Target;
    type Error = <ActionsView<'s> as justact_core::Actions>::Error;

    #[inline]
    fn enact(&mut self, target: Self::Target, msg: Self::Action) -> Result<(), Self::Error> { self.acts.enact(target, msg) }
}
impl<'s> justact_core::Statements for LocalView<'s> {
    type Statement = <StatementsView<'s> as justact_core::Statements>::Statement;
    type Message = <StatementsView<'s> as justact_core::Statements>::Message;
    type Target = <StatementsView<'s> as justact_core::Statements>::Target;
    type Error = <StatementsView<'s> as justact_core::Statements>::Error;

    #[inline]
    fn state(&mut self, target: Self::Target, msg: Self::Message) -> Result<(), Self::Error> { self.stmts.state(target, msg) }
}



/// Implements an [`Actions`](justact_core::Actions) with a potentially partial view on actions.
#[derive(Clone, Debug)]
pub struct Actions {
    /// All enacted statements in the system.
    pub acts:      Vec<Action>,
    /// Per-agent bitmaps that mask the msgs to find their own.
    pub masks:     HashMap<&'static str, BitVec>,
    /// The interface we're using to log things nicely.
    pub interface: Rc<RefCell<Interface>>,
}
impl Actions {
    /// Constructor for the Actions.
    ///
    /// # Arguments
    /// - `interface`: Some shared [`Interface`] that we use to log nice messages about when agents publish actions.
    ///
    /// # Returns
    /// A new Actions ready for use in the simulation.
    #[inline]
    pub fn new(interface: Rc<RefCell<Interface>>) -> Self { Self { acts: Vec::new(), masks: HashMap::new(), interface } }

    /// Returns a new Actions that is scoped to the given agent.
    ///
    /// # Arguments
    /// - `agent`: The identifier of the agent to scope to.
    ///
    /// # Returns
    /// A new [`ActionsView`] that will only returns messages for the given scope.
    ///
    /// Note that, if the given `agent` is unknown, the resulting `ActionsView` will not return any statements.
    #[inline]
    pub fn scope<'s>(&'s mut self, agent: &'static str) -> ActionsView<'s> { ActionsView { acts: self, agent: Some(agent) } }

    /// Returns a special scope that reflects _all_ agents.
    ///
    /// Kind of like a view for the system as a whole.
    ///
    /// # Returns
    /// A new [`ActionsView`] that returns all actions all agents enacted together.
    #[inline]
    pub fn full(&mut self) -> ActionsView { ActionsView { acts: self, agent: None } }



    /// Audits all actions.
    ///
    /// For more information on what exactly is audited, see [`Action::audit()`].
    ///
    /// # Arguments
    /// - `agrmnts`: An [`AgreementsView`] that provides access to the globally synchronized agreements.
    /// - `stmts`: A [`StatementsView`] that provides access to the messages this agent knows are stated.
    /// - `reported`: A list of actions that we already reported as failing the audit. This will prevent them from being raised again. If it's [`Some`], then it will be automatically updated.
    ///
    /// # Errors
    /// This function errors if the audit failed. Which property is violated, and how, is explained by the returned [`AuditExplanation`].
    #[inline]
    pub fn audit<'s, S, P>(
        &'s self,
        agrmnts: &AgreementsView<S>,
        stmts: &StatementsView,
        mut reported: Option<&mut HashSet<&'static str>>,
    ) -> Result<(), AuditExplanation<P::ExtractError, P::Explanation>>
    where
        S: Synchronizer<Agreement>,
        S::Error: 'static,
        P: ExtractablePolicy<&'s Message>,
    {
        // Go through all the actions to find the first culprit
        for act in &self.acts {
            if let Err(expl) = act.audit::<S, P>(agrmnts, stmts) {
                // See if we need to prevent reporting
                if let Some(reported) = &mut reported {
                    if !reported.contains(act.id()) {
                        reported.insert(act.id());
                        return Err(AuditExplanation { act: act.clone(), expl });
                    }
                } else {
                    return Err(AuditExplanation { act: act.clone(), expl });
                }
            }
        }
        Ok(())
    }
}

/// Implements a per-agent view on [`Actions`].
#[derive(Debug)]
pub struct ActionsView<'s> {
    /// All actions in the universe.
    acts:  &'s mut Actions,
    /// The agent for which this view behaves. If it's [`None`], that means a special view that has the full overview (pun intended).
    agent: Option<&'static str>,
}
impl<'s> justact_core::Set<Action> for ActionsView<'s> {
    type Item<'s2> = &'s2 Action where Self: 's2;
    type Iter<'s2> = ActionsIter<'s2, 's> where Self: 's2;

    #[inline]
    fn add(&mut self, new_elem: Action) -> bool {
        // Same as enacting only to yourself
        self.enact(self.agent.map(|a| Target::Agent(a)).unwrap_or(Target::All), new_elem).unwrap();
        false
    }

    #[inline]
    fn iter<'s2>(&'s2 self) -> Self::Iter<'s2> { ActionsIter { acts: self, i: 0 } }

    #[inline]
    fn len(&self) -> usize {
        let msgs_len: usize = self.acts.acts.len();
        self.acts
            .acts
            .iter()
            .zip(
                self.agent
                    .map(|a| self.acts.masks.get(a).map(Cow::Borrowed).unwrap_or_else(|| Cow::Owned(BitVec::from_elem(msgs_len, false))))
                    .unwrap_or_else(|| Cow::Owned(BitVec::from_elem(msgs_len, true)))
                    .iter(),
            )
            .filter(|(_, msk)| *msk)
            .count()
    }
}
impl<'s> justact_core::Map<Action> for ActionsView<'s> {
    #[inline]
    fn get(&self, id: &<Action as Identifiable>::Id) -> Option<&Action> {
        // See if it exists, which is if there's one with an ID that is not masked out
        for (i, stmt) in self.acts.acts.iter().enumerate() {
            if stmt.id() == id && self.agent.map(|a| self.acts.masks.get(a).map(|m| m[i]).unwrap_or(false)).unwrap_or(true) {
                // Found it
                return Some(stmt);
            }
        }
        None
    }
}
impl<'s> justact_core::Actions for ActionsView<'s> {
    type Enactment = Action;
    type Action = Action;
    type Target = Target;
    type Error = Infallible;

    #[inline]
    fn enact(&mut self, target: Self::Target, act: Self::Action) -> Result<(), Self::Error> {
        let Actions { acts, masks, interface } = &mut self.acts;

        // First, insert the act
        interface.borrow().log_enact(self.agent.unwrap_or("<system>"), &act);
        acts.push(act);

        // Then push the appropriate masks
        for (agent, masks) in masks {
            masks.push(target.matches(agent));
        }

        // aaaaand then we can return
        Ok(())
    }
}



/// Implements a [`Statements`](justact_core::statements::Statements) with a potentially partial view on messages.
#[derive(Clone, Debug)]
pub struct Statements {
    /// All stated messages in the system.
    pub msgs:      Vec<Message>,
    /// Per-agent bitmaps that mask the msgs to find their own.
    pub masks:     HashMap<&'static str, BitVec>,
    /// The interface we're using to log things nicely.
    pub interface: Rc<RefCell<Interface>>,
}
impl Statements {
    /// Constructor for the Statements.
    ///
    /// # Arguments
    /// - `interface`: Some shared [`Interface`] that we use to log nice messages about when agents publish actions.
    ///
    /// # Returns
    /// A new Statements ready for use in the simulation.
    #[inline]
    pub fn new(interface: Rc<RefCell<Interface>>) -> Self { Self { msgs: Vec::new(), masks: HashMap::new(), interface } }

    /// Returns a new Statements that is scoped to the given agent.
    ///
    /// # Arguments
    /// - `agent`: The identifier of the agent to scope to.
    ///
    /// # Returns
    /// A new [`StatementsView`] that will only returns messages for the given scope.
    ///
    /// Note that, if the given `agent` is unknown, the resulting `StatementsView` will not return any statements.
    #[inline]
    pub fn scope<'s>(&'s mut self, agent: &'static str) -> StatementsView<'s> { StatementsView { stmts: self, agent: Some(agent) } }

    /// Returns a special scope that reflects _all_ agents.
    ///
    /// Kind of like a view for the system as a whole.
    ///
    /// # Returns
    /// A new [`StatementsView`] that returns all messages all agents stated together.
    #[inline]
    pub fn full(&mut self) -> StatementsView { StatementsView { stmts: self, agent: None } }
}

/// Implements a per-agent view on [`Statements`].
#[derive(Debug)]
pub struct StatementsView<'s> {
    /// All statements in the universe.
    stmts: &'s mut Statements,
    /// The agent for which this view behaves. If it's [`None`], that means a special view that has the full overview (pun intended).
    agent: Option<&'static str>,
}
impl<'s> justact_core::Set<Message> for StatementsView<'s> {
    type Item<'s2> = &'s2 Message where Self: 's2;
    type Iter<'s2> = StatementsIter<'s2, 's> where Self: 's2;

    #[inline]
    fn add(&mut self, new_elem: Message) -> bool {
        // Same as stating only to yourself
        self.state(self.agent.map(Target::Agent).unwrap_or(Target::All), new_elem).unwrap();
        false
    }

    #[inline]
    fn iter<'s2>(&'s2 self) -> Self::Iter<'s2> { StatementsIter { stmts: self, i: 0 } }

    #[inline]
    fn len(&self) -> usize {
        let msgs_len: usize = self.stmts.msgs.len();
        self.stmts
            .msgs
            .iter()
            .zip(
                self.agent
                    .map(|a| self.stmts.masks.get(a).map(Cow::Borrowed).unwrap_or_else(|| Cow::Owned(BitVec::from_elem(msgs_len, false))))
                    .unwrap_or_else(|| Cow::Owned(BitVec::from_elem(msgs_len, true)))
                    .iter(),
            )
            .filter(|(_, msk)| *msk)
            .count()
    }
}
impl<'s> justact_core::Map<Message> for StatementsView<'s> {
    #[inline]
    fn get(&self, id: &<Message as Identifiable>::Id) -> Option<&Message> {
        // See if it exists, which is if there's one with an ID that is not masked out
        for (i, stmt) in self.stmts.msgs.iter().enumerate() {
            if stmt.id() == id && self.agent.map(|a| self.stmts.masks.get(a).map(|m| m[i]).unwrap_or(false)).unwrap_or(true) {
                // Found it
                return Some(stmt);
            }
        }
        None
    }
}
impl<'s> justact_core::Statements for StatementsView<'s> {
    type Statement = Message;
    type Message = Message;
    type Target = Target;
    type Error = Infallible;

    #[inline]
    fn state(&mut self, target: Self::Target, msg: Self::Message) -> Result<(), Self::Error> {
        let Statements { msgs, masks, interface } = &mut self.stmts;

        // First, insert the message
        interface.borrow().log_state(self.agent.unwrap_or("<system>"), &msg);
        msgs.push(msg);

        // Then push the appropriate masks
        for (agent, masks) in masks {
            masks.push(target.matches(agent));
        }

        // aaaaand then we can return
        Ok(())
    }
}
