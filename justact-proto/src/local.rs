//  POOL.rs
//    by Lut99
//
//  Created:
//    15 Apr 2024, 16:16:19
//  Last edited:
//    15 May 2024, 10:51:30
//  Auto updated?
//    Yes
//
//  Description:
//!   Provides an implementation for the JustAct version of a
//!   [`MessagePool`](justact_core::world::MessagePool).
//

use std::borrow::Cow;
use std::collections::HashMap;
use std::convert::Infallible;

use bit_vec::BitVec;
use justact_core::auxillary::Identifiable;
use justact_core::local as justact;

use crate::set::Set;
use crate::wire::{Action, AuditExplanation, Message};


/***** Auxillary *****/
/// Determines the possible targets that agents can send messages to for this [`Statements`].
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Target {
    /// Send it to everybody.
    All,
    /// Send it to a particular agent with this ID.
    Agent(&'static str),
}

/// Explains why an audit of all [`Action`]s in a [`Statements`] failed.
#[derive(Debug)]
pub struct Explanation<E1, E2> {
    /// The action that failed to audit
    pub act:  Action,
    /// The reason why this action failed.
    pub expl: AuditExplanation<E1, E2>,
}





/***** LIBRARY *****/
/// Implements a [`Statements`](justact_core::statements::Statements) with a potentially partial view on messages.
#[derive(Clone, Debug)]
pub struct Statements {
    /// All stated messages in the system.
    msgs:  Vec<Message>,
    /// Per-agent bitmaps that mask the msgs to find their own.
    masks: HashMap<&'static str, BitVec>,
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
    pub fn new() -> Self { Self { msgs: Vec::new(), masks: HashMap::new() } }

    /// Returns a new Statements that is scoped to the given agent.
    ///
    /// # Arguments
    /// - `agent`: The identifier of the agent to scope to.
    ///
    /// # Returns
    /// A new [`StatementsMut`] that will only returns messages for the given scope.
    ///
    /// Note that, if the given `agent` is unknown, the resulting `StatementsMut` will not return any statements.
    #[inline]
    pub fn scope<'s>(&'s mut self, agent: &'static str) -> StatementsMut<'s> { StatementsMut { stmts: self, agent } }
}



/// Implements a per-agent view on [`Statements`].
#[derive(Debug)]
pub struct StatementsMut<'s> {
    /// All statements in the universe.
    stmts: &'s mut Statements,
    /// The agent for which this view behaves.
    agent: &'static str,
}
impl<'s> justact_core::Set<Cow<'s, Message>> for StatementsMut<'s> {
    type Item<'s2> = &'s2 Message where Self: 's2;
    type Iter<'s2> = StatementsIter<'s2> where Self: 's2;

    #[inline]
    fn add(&mut self, new_elem: Cow<'s, Message>) -> bool {
        // Same as stating only to yourself
        todo!()
    }

    #[inline]
    fn get(&self, id: <Cow<'s, Message> as Identifiable>::Id) -> Option<&Cow<'s, Message>>
    where
        Cow<'s, Message>: justact_core::auxillary::Identifiable,
    {
        let Statements { msgs, masks } = &self.stmts;

        // See if it exists
        for (i, stmt) in msgs.iter().enumerate() {
            if stmt.id() == id && masks.get(self.agent).map(|m| m[i]).unwrap_or(false) {
                // Found it
                return Ok(&Cow::Borrowed(stmt));
            }
        }
    }
}
impl<'s> justact_core::Statements for StatementsMut<'s> {
    type Statement = Cow<'s, Message>;
    type Message = Message;
    type Target = Target;
    type Error = Infallible;

    #[inline]
    fn state(&mut self, target: Self::Target, msg: Self::Message) -> Result<(), Self::Error> {}
}
