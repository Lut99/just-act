//  POOL.rs
//    by Lut99
//
//  Created:
//    15 Apr 2024, 16:16:19
//  Last edited:
//    13 May 2024, 16:07:41
//  Auto updated?
//    Yes
//
//  Description:
//!   Provides an implementation for the JustAct version of a
//!   [`MessagePool`](justact_core::world::MessagePool).
//

#[cfg(not(any(feature = "datalog")))]
compile_error!("Please enable the 'datalog'-feature.");

use std::borrow::Cow;
use std::collections::{HashMap, HashSet};

use justact_core::local as justact;
use justact_core::wire::{Action as _, Message as _};

#[cfg(feature = "datalog")]
use crate::lang::datalog;


/***** ITERATORS *****/
/// Iterates over all stated [`Message`](justact_core::message::Message)s in a [`StatementsMut`].
#[derive(Clone, Debug)]
pub struct StatedIter<'s> {
    /// The agent for which we're scoped.
    agent:  &'static str,
    /// The iterator producing scopes filtered to us.
    scopes: std::collections::hash_map::Iter<'s, Scope, HashMap<&'static str, datalog::Message>>,
    /// The iterator going over the messages in a scope.
    msgs:   Option<std::collections::hash_map::Values<'s, &'static str, datalog::Message>>,
}
impl<'s> Iterator for StatedIter<'s> {
    type Item = &'s datalog::Message;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self.msgs.iter_mut().flat_map(|i| i).next() {
            Some(next) => Some(next),
            None => match self.scopes.next() {
                Some((scope, msgs)) => {
                    if *scope == Scope::All || *scope == Scope::Agent(self.agent) {
                        self.msgs = Some(msgs.values());
                        self.next()
                    } else {
                        self.next()
                    }
                },
                None => None,
            },
        }
    }
}

/// Iterates over all enacted [`Action`](justact_core::message::Action)s in a [`StatementsMut`].
#[derive(Clone, Debug)]
pub struct EnactedIter<'s> {
    /// The agent for which we're scoped.
    agent:  &'static str,
    /// The iterator producing scopes filtered to us.
    scopes: std::collections::hash_map::Iter<'s, Scope, HashSet<datalog::Action<'static>>>,
    /// The iterator going over the actions in a scope.
    acts:   Option<std::collections::hash_set::Iter<'s, datalog::Action<'static>>>,
}
impl<'s> Iterator for EnactedIter<'s> {
    type Item = &'s datalog::Action<'static>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self.acts.iter_mut().flat_map(|i| i).next() {
            Some(next) => Some(next),
            None => match self.scopes.next() {
                Some((scope, acts)) => {
                    if *scope == Scope::All || *scope == Scope::Agent(self.agent) {
                        self.acts = Some(acts.iter());
                        self.next()
                    } else {
                        self.next()
                    }
                },
                None => None,
            },
        }
    }
}





/***** Auxillary *****/
/// Determines the possible scopes that agents can send messages to for this [`Statements`].
///
/// It's only to everybody.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Scope {
    /// Send it to everybody.
    All,
    /// Send it to a particular agent with this ID.
    Agent(&'static str),
}

/// Explains why an audit of all [`Action`]s in a [`Statements`] failed.
#[derive(Clone, Debug)]
pub struct Explanation {
    /// The action that failed to audit
    #[cfg(feature = "datalog")]
    pub action:      datalog::Action<'static>,
    /// The reason why this action failed.
    #[cfg(feature = "datalog")]
    pub explanation: datalog::Explanation,
}





/***** LIBRARY *****/
/// Implements a [`Statements`](justact_core::statements::Statements) with a potentially partial view on messages.
#[derive(Clone, Debug)]
pub struct Statements {
    /// All messages in the system.
    ///
    /// They are mapped by scope, then by message ID.
    #[cfg(feature = "datalog")]
    messages: HashMap<Scope, HashMap<&'static str, datalog::Message>>,
    /// All actions in the system.
    ///
    /// They are mapped by scope.
    #[cfg(feature = "datalog")]
    actions:  HashMap<Scope, HashSet<datalog::Action<'static>>>,
}

impl Default for Statements {
    #[inline]
    fn default() -> Self { Self::new() }
}
impl Statements {
    /// Constructor for the Statements.
    ///
    /// # Returns
    /// A new Statements ready for use in the simulation.
    #[inline]
    pub fn new() -> Self { Self { messages: HashMap::new(), actions: HashMap::new() } }

    /// Returns a new Statements that is scoped to the given agent.
    ///
    /// # Arguments
    /// - `agent`: The identifier of the agent to scope to.
    ///
    /// # Returns
    /// A new Statements that will only returns messages for the given scope.
    #[inline]
    pub fn scope<'s: 'm, 'm>(&'s mut self, agent: &'static str) -> StatementsMut<'m> {
        StatementsMut { agent, messages: &mut self.messages, actions: &mut self.actions }
    }
}

impl justact::Statements for Statements {
    type EnactedIter<'s> = std::iter::FlatMap<
        std::collections::hash_map::Values<'s, Scope, HashSet<datalog::Action<'s>>>,
        std::collections::hash_set::Iter<'s, datalog::Action<'s>>,
        fn(&'s HashSet<datalog::Action>) -> std::collections::hash_set::Iter<'s, datalog::Action<'s>>,
    >;
    type Explanation<'s> = Explanation;
    type Id = &'static str;
    type StatedIter<'s> = std::iter::FlatMap<
        std::collections::hash_map::Values<'s, Scope, HashMap<&'static str, datalog::Message>>,
        std::collections::hash_map::Values<'s, &'static str, datalog::Message>,
        fn(&'s HashMap<&'static str, datalog::Message>) -> std::collections::hash_map::Values<'s, &'static str, datalog::Message>,
    >;
    type Statement<'s> = datalog::Message;

    #[inline]
    fn audit<'s>(&'s self) -> Result<(), Self::Explanation<'s>> {
        match self.actions.values().find_map(|acts| acts.iter().find_map(|a| a.audit(self).err().map(|e| (a, e)))) {
            Some((action, explanation)) => Err(Explanation { action: action.clone(), explanation }),
            None => Ok(()),
        }
    }

    #[inline]
    fn get_stated<'s>(&'s self, id: Self::Id) -> Option<Self::Statement<'s>> { self.messages.values().find_map(|msgs| msgs.get(id)).cloned() }

    #[inline]
    fn n_stated(&self) -> usize { self.messages.values().map(|msgs| msgs.len()).sum() }

    #[inline]
    fn stated<'s>(&'s self) -> Self::StatedIter<'s> { self.messages.values().flat_map(|msgs| msgs.values()) }

    #[inline]
    fn n_enacted(&self) -> usize { self.actions.values().map(|acts| acts.len()).sum() }

    #[inline]
    fn enacted<'s>(&'s self) -> Self::EnactedIter<'s> { self.actions.values().flat_map(|acts| acts.iter()) }
}
impl justact::Stating for Statements {
    #[cfg(feature = "datalog")]
    type Action = datalog::Action<'static>;
    type Error = std::convert::Infallible;
    #[cfg(feature = "datalog")]
    type Message = Cow<'static, datalog::Message>;
    type Scope = Scope;

    #[inline]
    fn state(&mut self, message: Self::Message, scope: Scope) -> Result<(), Self::Error> {
        // Inject it into the message pool.
        self.messages.entry(scope).or_default().insert(message.id(), message.into_owned());
        Ok(())
    }

    #[inline]
    fn enact(&mut self, act: Self::Action, scope: Scope) -> Result<(), Self::Error> {
        // Inject it into the action pool.
        self.actions.entry(scope).or_default().insert(act);
        Ok(())
    }
}



/// A mutably borrowed version of the [`Statements`] that is scoped to one particular version of an [`Agent`].
#[derive(Debug)]
pub struct StatementsMut<'s> {
    /// The agent to which this StatementsMut is scoped.
    agent:    &'static str,
    /// The messages to mutate/read.
    #[cfg(feature = "datalog")]
    messages: &'s mut HashMap<Scope, HashMap<&'static str, datalog::Message>>,
    /// The actions to mutate/read.
    #[cfg(feature = "datalog")]
    actions:  &'s mut HashMap<Scope, HashSet<datalog::Action<'static>>>,
}

impl<'s> justact::Statements for StatementsMut<'s> {
    type EnactedIter<'s2> = EnactedIter<'s2> where Self: 's2;
    type Explanation<'s2> = Explanation where Self: 's2;
    type Id = &'static str;
    type StatedIter<'s2> = StatedIter<'s2> where Self: 's2;
    type Statement<'s2> = datalog::Message where Self: 's2;

    #[inline]
    fn audit<'s2>(&'s2 self) -> Result<(), Self::Explanation<'s2>> {
        if let Some(all) = self.actions.get(&Scope::All) {
            if let Some((action, explanation)) = all.iter().find_map(|a| a.audit(self).err().map(|e| (a, e))) {
                return Err(Explanation { action: action.clone(), explanation });
            }
        } else if let Some(agent) = self.actions.get(&Scope::Agent(self.agent)) {
            if let Some((action, explanation)) = agent.iter().find_map(|a| a.audit(self).err().map(|e| (a, e))) {
                return Err(Explanation { action: action.clone(), explanation });
            }
        }
        Ok(())
    }

    #[inline]
    fn get_stated<'s2>(&'s2 self, id: Self::Id) -> Option<Self::Statement<'s2>> {
        if let Some(all) = self.messages.get(&Scope::All) {
            return all.get(id).cloned();
        } else if let Some(agent) = self.messages.get(&Scope::Agent(self.agent)) {
            return agent.get(id).cloned();
        }
        None
    }

    #[inline]
    fn n_stated(&self) -> usize {
        self.messages.get(&Scope::All).map(|msgs| msgs.len()).unwrap_or(0)
            + self.messages.get(&Scope::Agent(self.agent)).map(|msgs| msgs.len()).unwrap_or(0)
    }

    #[inline]
    fn stated<'s2>(&'s2 self) -> Self::StatedIter<'s2> { StatedIter { agent: self.agent, scopes: self.messages.iter(), msgs: None } }

    #[inline]
    fn n_enacted(&self) -> usize {
        self.actions.get(&Scope::All).map(|acts| acts.len()).unwrap_or(0)
            + self.actions.get(&Scope::Agent(self.agent)).map(|acts| acts.len()).unwrap_or(0)
    }

    #[inline]
    fn enacted<'s2>(&'s2 self) -> Self::EnactedIter<'s2> { EnactedIter { agent: self.agent, scopes: self.actions.iter(), acts: None } }
}
impl<'s> justact::Stating for StatementsMut<'s> {
    #[cfg(feature = "datalog")]
    type Action = datalog::Action<'static>;
    type Error = std::convert::Infallible;
    #[cfg(feature = "datalog")]
    type Message = Cow<'static, datalog::Message>;
    type Scope = Scope;

    #[inline]
    fn state(&mut self, message: Self::Message, scope: Scope) -> Result<(), Self::Error> {
        // Inject it into the message pool.
        self.messages.entry(scope).or_default().insert(message.id(), message.into_owned());
        Ok(())
    }

    #[inline]
    fn enact(&mut self, act: Self::Action, scope: Scope) -> Result<(), Self::Error> {
        // Inject it into the action pool.
        self.actions.entry(scope).or_default().insert(act);
        Ok(())
    }
}
