//  POOL.rs
//    by Lut99
//
//  Created:
//    15 Apr 2024, 16:16:19
//  Last edited:
//    16 Apr 2024, 14:49:11
//  Auto updated?
//    Yes
//
//  Description:
//!   Provides an implementation for the JustAct version of a
//!   [`MessagePool`](justact_core::world::MessagePool).
//

#[cfg(not(any(feature = "datalog")))]
compile_error!("Please enable the 'datalog'-feature.");

use std::collections::{HashMap, HashSet};

use justact_core::collection::CollectionMut as _;
use justact_core::world as justact;

#[cfg(feature = "datalog")]
use crate::message::datalog;


/***** Auxillary *****/
/// Determines the possible scopes that agents can send messages to for this [`MessagePool`].
///
/// It's only to everybody.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Scope {
    /// Send it to everybody.
    All,
}





/***** LIBRARY *****/
/// Implements a [`MessagePool`](justact_core::world::MessagePool) with a _total_ view on all messages.
///
/// This means that all agents see the exact same messages.
#[derive(Clone, Debug)]
pub struct MessagePool {
    /// All messages in the system.
    #[cfg(feature = "datalog")]
    messages: HashMap<&'static str, datalog::Message<'static>>,
    /// All actions in the system.
    #[cfg(feature = "datalog")]
    actions:  HashSet<datalog::Action<'static>>,
}

impl Default for MessagePool {
    #[inline]
    fn default() -> Self { Self::new() }
}
impl MessagePool {
    /// Constructor for the MessagePool.
    ///
    /// # Returns
    /// A new MessagePool ready for use in the simulation.
    #[inline]
    pub fn new() -> Self { Self { messages: HashMap::new(), actions: HashSet::new() } }
}

impl justact::MessagePool for MessagePool {
    #[cfg(feature = "datalog")]
    type Action = datalog::Action<'static>;
    type ActionCollection<'s> = &'s HashSet<Self::Action>;
    type Error = std::convert::Infallible;
    #[cfg(feature = "datalog")]
    type Message = datalog::Message<'static>;
    type MessageCollection<'s> = &'s HashMap<&'static str, Self::Message>;
    type Scope = Scope;

    #[inline]
    fn emit(&mut self, message: Self::Message, _scope: Scope) -> Result<(), Self::Error> {
        // Inject it into the message pool.
        self.messages.add(message);
        Ok(())
    }

    #[inline]
    fn enact(&mut self, act: Self::Action, _scope: Scope) -> Result<(), Self::Error> {
        // Inject it into the action pool.
        self.actions.add(act);
        Ok(())
    }

    #[inline]
    fn all_messages<'s>(&'s mut self) -> Self::MessageCollection<'s> {
        // No need to update anything, because it's total and updated on emit()
        &self.messages
    }

    #[inline]
    fn all_actions<'s>(&'s mut self) -> Self::ActionCollection<'s> {
        // No need to update anything, because it's total and updated on emit()
        &self.actions
    }
}
