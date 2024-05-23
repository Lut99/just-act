//  AGREEMENTS.rs
//    by Lut99
//
//  Created:
//    23 May 2024, 17:42:56
//  Last edited:
//    23 May 2024, 17:46:52
//  Auto updated?
//    Yes
//
//  Description:
//!   Implements (various) global views on agreements.
//

use std::error::Error;
use std::fmt::{Display, Formatter, Result as FResult};

use justact_core::agreements::{Agreement, Agreements as JAAgreements};
use justact_core::auxillary::Identifiable as _;
use justact_core::set::LocalSet;

use crate::message::{Message, OwnedMessage};


/***** ERRORS *****/
/// Determines the possible errors for the [`AgreementsDictator`] set.
#[derive(Debug)]
pub enum AgreementsDictatorError {
    /// The agent attempting to advance the time was not the dictator.
    NotTheDictator { id: String, agent: String, dictator: String },
}
impl Display for AgreementsDictatorError {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        use AgreementsDictatorError::*;
        match self {
            NotTheDictator { id, agent, dictator } => {
                write!(f, "Agent '{agent}' failed to create an agreement out of statement '{id}' because they are not the dictator ('{dictator}' is)")
            },
        }
    }
}
impl Error for AgreementsDictatorError {}





/***** LIBRARY *****/
/// Provides agents with a global view on the agreed upon agreements.
///
/// This variation synchronizes new agreements if and only if it's a particular agent claiming it.
#[derive(Debug)]
pub struct AgreementsDictator<'v> {
    /// This agent
    agent:    String,
    /// The only agent allowed to make changes.
    dictator: String,

    /// The statements that this agent knows of.
    agrs: LocalSet<Agreement<Message<'v>>>,
    /// A queue of statements that this agent pushed.
    pub(crate) queue: Vec<Agreement<OwnedMessage>>,
}
impl<'v> JAAgreements for AgreementsDictator<'v> {
    type Message = OwnedMessage;
    type Statement<'s> = Message<'s> where Self: 's;
    type Error = AgreementsDictatorError;

    #[inline]
    fn agree(&mut self, agr: Agreement<Self::Message>) -> Result<(), Self::Error> {
        // Do not advance if we're not the dictator
        if self.agent == self.dictator {
            self.queue.push(agr);
            Ok(())
        } else {
            Err(AgreementsDictatorError::NotTheDictator { id: agr.id().into(), agent: self.agent.clone(), dictator: self.dictator.clone() })
        }
    }

    #[inline]
    fn agreed<'s>(&'s self) -> LocalSet<Agreement<Self::Statement<'s>>> {
        // Start with what we know...
        let mut set: LocalSet<Agreement<Message<'s>>> = self.agrs.clone();
        // ...and push any queued items for us
        set.reserve(self.queue.len());
        for agr in &self.queue {
            set.add(Agreement { msg: (&agr.msg).into(), timestamp: agr.timestamp });
        }
        // OK
        set
    }
}
