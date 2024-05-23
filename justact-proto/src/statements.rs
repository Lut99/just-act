//  STATEMENTS.rs
//    by Lut99
//
//  Created:
//    23 May 2024, 13:54:33
//  Last edited:
//    23 May 2024, 17:38:40
//  Auto updated?
//    Yes
//
//  Description:
//!   Implements the local view set of stated- and enacted messages.
//

use justact_core::agreements::Agreement;
use justact_core::set::LocalSet;
use justact_core::statements::{Action, Statements as JAStatements};

use crate::message::{Message, OwnedMessage};


/***** AUXILLARY *****/
/// Determines the possible targets that agents can send messages to for this [`Statements`].
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Target {
    /// Send it to everybody.
    All,
    /// Send it to a particular agent with this ID.
    Agent(String),
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
    pub fn matches(&self, agent: &str) -> bool {
        match self {
            Self::All => true,
            Self::Agent(a) => *a == agent,
        }
    }
}





/***** LIBRARY *****/
/// Provides agents with a local view on the stated- and enacted messages.
#[derive(Debug)]
pub struct Statements<'v> {
    /// This agent
    agent: String,

    /// The statements that this agent knows of.
    stmts: LocalSet<Message<'v>>,
    /// A queue of statements that this agent pushed.
    pub(crate) stmts_queue: Vec<(Target, OwnedMessage)>,

    /// The enactments that this agent knows of.
    encts: LocalSet<Action<Message<'v>>>,
    /// A queue of enactments that this agent pushed.
    pub(crate) encts_queue: Vec<(Target, Action<OwnedMessage>)>,
}
impl<'v> JAStatements for Statements<'v> {
    type Message = OwnedMessage;
    type Statement<'s> = Message<'s> where Self: 's;
    type Target = Target;
    type Status = ();


    #[inline]
    #[track_caller]
    fn state(&mut self, target: Self::Target, msg: Self::Message) -> Self::Status {
        // Simply push to the queue
        self.stmts_queue.push((target, msg));
    }

    #[inline]
    fn stated<'s>(&'s self) -> LocalSet<Self::Statement<'s>> {
        // Start with what we know...
        let mut set: LocalSet<Message<'s>> = self.stmts.clone();
        // ...and push any queued items for us
        for (target, msg) in &self.stmts_queue {
            if target.matches(&self.agent) {
                set.add(msg.into());
            }
        }
        // OK
        set
    }



    #[inline]
    fn enact<'s>(&'s mut self, target: Self::Target, act: Action<Self::Message>) -> Self::Status {
        // Simply push to the queue
        self.encts_queue.push((target, act));
    }

    #[inline]
    fn enacted<'s>(&'s self) -> LocalSet<Action<Self::Statement<'s>>> {
        // Start with what we know...
        let mut set: LocalSet<Action<Message<'s>>> = self.encts.clone();
        // ...and push any queued items for us
        for (target, act) in &self.encts_queue {
            if target.matches(&self.agent) {
                set.add(Action {
                    basis:     Agreement { msg: (&act.basis.msg).into(), timestamp: act.basis.timestamp },
                    just:      act.just.iter().map(Message::from).collect(),
                    enacts:    (&act.enacts).into(),
                    timestamp: act.timestamp,
                });
            }
        }
        // OK
        set
    }
}
