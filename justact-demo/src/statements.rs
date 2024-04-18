//  POOL.rs
//    by Lut99
//
//  Created:
//    15 Apr 2024, 16:16:19
//  Last edited:
//    18 Apr 2024, 17:06:43
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
use std::cell::{Ref, RefCell};
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use justact_core::message::{Action as _, Message as _};
use justact_core::statements as justact;

#[cfg(feature = "datalog")]
use crate::lang::datalog;


/***** Auxillary *****/
/// Determines the possible scopes that agents can send messages to for this [`MessagePool`].
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
    /// Some scope that determines which part of the messages is returned.
    scope: Scope,

    /// All messages in the system.
    ///
    /// They are mapped by scope, then by message ID.
    #[cfg(feature = "datalog")]
    messages: Rc<RefCell<HashMap<Scope, HashMap<&'static str, datalog::Message>>>>,
    /// All actions in the system.
    ///
    /// They are mapped by scope.
    #[cfg(feature = "datalog")]
    actions:  Rc<RefCell<HashMap<Scope, HashSet<datalog::Action<'static>>>>>,
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
    pub fn new() -> Self {
        Self { scope: Scope::All, messages: Rc::new(RefCell::new(HashMap::new())), actions: Rc::new(RefCell::new(HashMap::new())) }
    }

    /// Returns a new Statements that is scoped to the given agent.
    ///
    /// # Arguments
    /// - `scope`: The [`Scope`] of the agent.
    ///
    /// # Returns
    /// A new Statements that will only returns messages for the given scope.
    #[inline]
    pub fn scope(&self, scope: Scope) -> Self { Self { scope, messages: self.messages.clone(), actions: self.actions.clone() } }
}

impl justact::Statements for Statements {
    type Explanation<'s> = Explanation;
    type Id = &'static str;
    type Message<'s> = datalog::Message;

    #[inline]
    fn audit<'s>(&'s self) -> Result<(), Self::Explanation<'s>> {
        match self.scope {
            Scope::All => match self.actions.borrow().values().find_map(|acts| acts.iter().find_map(|a| a.audit(self).err().map(|e| (a, e)))) {
                Some((action, explanation)) => Err(Explanation { action: action.clone(), explanation }),
                None => Ok(()),
            },

            Scope::Agent(agent) => {
                let acts: Ref<_> = self.actions.borrow();
                if let Some(all) = acts.get(&Scope::All) {
                    if let Some((action, explanation)) = all.iter().find_map(|a| a.audit(self).err().map(|e| (a, e))) {
                        return Err(Explanation { action: action.clone(), explanation });
                    }
                } else if let Some(agent) = acts.get(&Scope::Agent(agent)) {
                    if let Some((action, explanation)) = agent.iter().find_map(|a| a.audit(self).err().map(|e| (a, e))) {
                        return Err(Explanation { action: action.clone(), explanation });
                    }
                }
                Ok(())
            },
        }
    }

    #[inline]
    fn get_stated<'s>(&'s self, id: Self::Id) -> Option<Self::Message<'s>> {
        match self.scope {
            Scope::All => self.messages.borrow().values().find_map(|msgs| msgs.get(id)).cloned(),
            Scope::Agent(agent) => {
                let msgs: Ref<_> = self.messages.borrow();
                if let Some(all) = msgs.get(&Scope::All) {
                    return all.get(id).cloned();
                } else if let Some(agent) = msgs.get(&Scope::Agent(agent)) {
                    return agent.get(id).cloned();
                }
                None
            },
        }
    }

    #[inline]
    fn is_stated(&self, id: Self::Id) -> bool { self.get_stated(id).is_some() }

    #[inline]
    fn n_stated(&self) -> usize {
        match self.scope {
            Scope::All => self.messages.borrow().values().map(|msgs| msgs.len()).sum(),
            Scope::Agent(agent) => {
                let msgs: Ref<_> = self.messages.borrow();
                msgs.get(&Scope::All).map(|msgs| msgs.len()).unwrap_or(0) + msgs.get(&Scope::Agent(agent)).map(|msgs| msgs.len()).unwrap_or(0)
            },
        }
    }

    #[inline]
    fn n_enacted(&self) -> usize {
        match self.scope {
            Scope::All => self.actions.borrow().values().map(|acts| acts.len()).sum(),
            Scope::Agent(agent) => {
                let acts: Ref<_> = self.actions.borrow();
                acts.get(&Scope::All).map(|acts| acts.len()).unwrap_or(0) + acts.get(&Scope::Agent(agent)).map(|acts| acts.len()).unwrap_or(0)
            },
        }
    }
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
        self.messages.borrow_mut().entry(scope).or_default().insert(message.id(), message.into_owned());
        Ok(())
    }

    #[inline]
    fn enact(&mut self, act: Self::Action, scope: Scope) -> Result<(), Self::Error> {
        // Inject it into the action pool.
        self.actions.borrow_mut().entry(scope).or_default().insert(act);
        Ok(())
    }
}
