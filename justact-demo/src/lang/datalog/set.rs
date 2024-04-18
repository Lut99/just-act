//  SET.rs
//    by Lut99
//
//  Created:
//    18 Apr 2024, 13:36:28
//  Last edited:
//    18 Apr 2024, 16:56:01
//  Auto updated?
//    Yes
//
//  Description:
//!   Defines a $Datalog^\neg$-compatible [`MessageSet`] and
//!   [`ActionSet`].
//

use std::borrow::Cow;
use std::fmt::{Display, Formatter, Result as FResult};
use std::hash::Hash;

use justact_core::message::Message as _;
use justact_core::set::Set as _;
use justact_policy::datalog;
use justact_policy::datalog::ast::{datalog, AtomArg, Rule, Spec};

use super::message::Message;
use crate::set::Set;


/***** LIBRARY *****/
/// Implements a collection of policy messages.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MessageSet<'m> {
    /// The internal data we wrap
    data: Set<Cow<'m, Message>>,
}
impl<'m> Default for MessageSet<'m> {
    #[inline]
    fn default() -> Self { Self::empty() }
}
impl<'m> MessageSet<'m> {
    /// Creates an empty MessageSet.
    ///
    /// # Returns
    /// A MessageSet with not messages in it yet.
    #[inline]
    pub fn empty() -> Self { Self { data: Set::empty() } }

    /// Creates a new clone where all messages are borrowed from this one.
    ///
    /// # Returns
    /// A MessageSet with the same messages as this, except they're all borrowed from `self`.
    #[inline]
    pub fn reborrow<'s>(&'s self) -> MessageSet<'s> { MessageSet { data: self.data.reborrow() } }

    /// Merges the given MessageSet into this one.
    ///
    /// # Arguments
    /// - `other`: Some other MessageSet to join.
    #[inline]
    pub fn join(&mut self, other: impl IntoIterator<Item = Cow<'m, Message>>) { self.data.join(other) }

    /// Returns the number of messages in this set.
    #[inline]
    pub fn len(&self) -> usize { self.data.len() }
}

impl<'m> Display for MessageSet<'m> {
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> FResult {
        writeln!(f, "MessageSet {{")?;
        for msg in self {
            writeln!(f, "    Message {{")?;
            for rule in &msg.policy.rules {
                writeln!(f, "        {rule}")?;
            }
            writeln!(f, "    }}")?;
        }
        writeln!(f, "}}")
    }
}
impl<'m> Hash for MessageSet<'m> {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) { self.data.hash(state) }
}

impl<'m> justact_core::set::Set for MessageSet<'m> {
    type Elem = Cow<'m, Message>;
    type Item<'s> = &'s Message where Self: 's;
    type Iter<'s> = std::iter::Map<crate::set::Iter<'s, Cow<'m, Message>>, fn(&'s Cow<'m, Message>) -> &'s Message> where Self: 's;

    #[inline]
    fn iter<'s>(&'s self) -> Self::Iter<'s> {
        self.data.iter().map(|m| match m {
            Cow::Borrowed(m) => m,
            Cow::Owned(m) => m,
        })
    }

    #[inline]
    fn add(&mut self, new_elem: Cow<'m, Message>) -> bool { self.data.add(new_elem) }
}
impl<'m> justact_core::set::MessageSet for MessageSet<'m> {
    type Message = Cow<'m, Message>;
    type Policy<'s> = datalog::Policy where Self: 's;

    fn extract<'s>(&'s self) -> Self::Policy<'s> {
        // Collect the rules of all the messages
        let mut rules: Vec<Rule> = Vec::with_capacity(self.len());
        for msg in self {
            for rule in &msg.policy.spec.rules {
                // Check if the controller-rule is violated
                let mut violation: bool = false;
                for cons in rule.consequences.values() {
                    // See if this consequent is controlled
                    if !cons.ident.value.value().starts_with("ctl_") {
                        continue;
                    }

                    // Find the first argument of the consequent
                    let first_arg: &AtomArg = match &cons.args {
                        Some(args) => match args.args.values().next() {
                            Some(arg) => arg,
                            None => continue,
                        },
                        None => continue,
                    };

                    // Ensure it is the author of the message
                    if first_arg.ident().value.value() != msg.author() {
                        violation = true;
                        break;
                    }
                }

                // Now add the rule if it's OK, or else add an always-error rule
                if !violation {
                    rules.push(rule.clone());
                } else {
                    rules.push(datalog! { #![crate = "::justact_policy::datalog"] error. }.rules.swap_remove(0));
                }
            }
        }

        // OK, simply return the inner ones
        datalog::Policy { spec: Spec { rules } }
    }
}

impl<'m> IntoIterator for MessageSet<'m> {
    type IntoIter = crate::set::IntoIter<Cow<'m, Message>>;
    type Item = Cow<'m, Message>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter { self.data.into_iter() }
}
impl<'s, 'm> IntoIterator for &'s MessageSet<'m> {
    type IntoIter = std::iter::Map<crate::set::Iter<'s, Cow<'m, Message>>, fn(&'s Cow<'m, Message>) -> &'s Message>;
    type Item = &'s Message;

    #[inline]
    fn into_iter(self) -> Self::IntoIter { self.iter() }
}

impl<'m> From<Cow<'m, Message>> for MessageSet<'m> {
    #[inline]
    fn from(value: Cow<'m, Message>) -> Self { MessageSet { data: Set::from(value) } }
}
