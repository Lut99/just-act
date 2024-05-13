//  MESSAGE.rs
//    by Lut99
//
//  Created:
//    18 Apr 2024, 13:50:56
//  Last edited:
//    13 May 2024, 15:39:56
//  Auto updated?
//    Yes
//
//  Description:
//!   Defines $Datalog^\neg$-carrying [`Message`]s and [`Action`]s.
//

use std::borrow::Cow;
use std::fmt::{Display, Formatter, Result as FResult};
use std::hash::Hash;

use justact_core::policy::Policy as _;
use justact_core::set::{MessageSet as _, Set as _};
use justact_core::wire::{self as justact, Message as _};
use justact_policy::datalog::interpreter::interpretation::Interpretation;
use justact_policy::datalog::{self, Policy};

use super::set::MessageSet;


/***** AUXILLARY *****/
/// Explains why an [`Action`] did not succeed an audit.
#[derive(Clone, Debug)]
pub enum Explanation {
    /// One of the messages embedded in the action was not stated.
    NotStated { message: &'static str },
    /// 'Error' was actually derived from the audit.
    Error { int: Interpretation<'static, 'static> },
}





/***** LIBRARY *****/
/// Defines a [`Message`](justact::Message) that carries $Datalog^\neg$ policy information.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Message {
    /// Some ID of this message.
    pub id:     &'static str,
    /// The author of this message.
    pub author: &'static str,
    /// Some policy that is emitted here.
    pub policy: datalog::Policy<'static, 'static>,
}
impl Message {
    /// Constructor for a Message.
    ///
    /// # Arguments
    /// - `id`: Some identifier for this message.
    /// - `author`: Some Agent that has authored this message. For the demo, we assume that this signature is always correct and cannot be tampered with.
    /// - `policy`: The policy rules wrapped in this message.
    ///
    /// # Returns
    /// A new Message.
    #[inline]
    pub fn new(id: &'static str, author: &'static str, policy: datalog::Policy<'static, 'static>) -> Self { Self { id, author, policy } }
}

impl Display for Message {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult { write!(f, "{}", self.policy) }
}
impl Ord for Message {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // SAFETY: OK to unwrap due to Ord's requirement on PartialOrd always returning `Some`
        self.partial_cmp(other).unwrap()
    }
}
impl PartialOrd for Message {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> { Some(self.id.cmp(&other.id)) }
}

impl justact::Message for Message {
    type Author = &'static str;
    type Identifier = &'static str;

    #[inline]
    fn author(&self) -> Self::Author { self.author }

    #[inline]
    fn id(&self) -> Self::Identifier { self.id }
}



/// Implements a justified set of statements.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Action<'m> {
    /// The basis of this action.
    pub basis: Cow<'m, Message>,
    /// The justification of this action.
    pub justification: MessageSet<'m>,
    /// The enactment of this action.
    pub enactment: Cow<'m, Message>,
}

impl<'m> Display for Action<'m> {
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> FResult {
        // Write the Action header
        writeln!(f, "Action {{")?;

        // Write the basis
        writeln!(f, "    Basis {{")?;
        for rule in &MessageSet::from(Cow::Borrowed(self.basis.as_ref())).extract().rules {
            writeln!(f, "        {rule}")?;
        }
        writeln!(f, "    }}")?;

        // Write the justificaiton
        writeln!(f, "    Justification {{")?;
        for rule in &self.justification.extract().rules {
            writeln!(f, "        {rule}")?;
        }
        writeln!(f, "    }}")?;

        // Write the enactment
        writeln!(f, "    Enactment {{")?;
        for rule in &MessageSet::from(Cow::Borrowed(self.enactment.as_ref())).extract().rules {
            writeln!(f, "        {rule}")?;
        }
        writeln!(f, "    }}")?;

        // Write footer
        writeln!(f, "}}")
    }
}

impl<'m> justact::Action for Action<'m> {
    type Explanation = Explanation;
    type Message<'s> = &'s Message where Self: 's;
    type MessageSet<'s> = MessageSet<'s> where Self: 's;

    #[inline]
    fn basis<'s>(&'s self) -> Self::Message<'s> { &self.basis }

    #[inline]
    fn justification<'s>(&'s self) -> Self::MessageSet<'s> {
        // Include the basis & enactment into the justification
        let mut set: MessageSet<'s> = self.justification.reborrow();
        set.add(Cow::Borrowed(&self.basis));
        set.add(Cow::Borrowed(&self.enactment));

        // Done, return it
        set
    }

    #[inline]
    fn enactment<'s>(&'s self) -> Self::Message<'s> { &self.enactment }

    #[inline]
    fn audit<'s, S>(&'s self, stmts: &S) -> Result<(), Self::Explanation>
    where
        S: justact_core::local::Statements<Id = <Self::Message<'s> as justact::Message>::Identifier>,
    {
        let just: MessageSet = self.justification();

        // Check if the justification is all stated
        for msg in &just {
            if !stmts.is_stated(msg.id()) {
                return Err(Explanation::NotStated { message: msg.id() });
            }
        }

        // Then check if the justification as a whole is valid
        let pol: Policy = just.extract();
        if let Err(int) = pol.check_validity() {
            return Err(Explanation::Error { int });
        }

        // Otherwise, OK
        Ok(())
    }
}
