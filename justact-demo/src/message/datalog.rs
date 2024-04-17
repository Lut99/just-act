//  DATALOG.rs
//    by Lut99
//
//  Created:
//    15 Apr 2024, 16:22:55
//  Last edited:
//    17 Apr 2024, 17:14:35
//  Auto updated?
//    Yes
//
//  Description:
//!   Defines how messages carrying $Datalog^\neg$ look like in the demo environment.
//

use std::borrow::Cow;
use std::collections::HashSet;
use std::fmt::{Display, Formatter, Result as FResult};
use std::hash::Hash;

use justact_core::collection::{Collection, CollectionMut};
use justact_core::message::{self as justact, Message as _, MessageSet as _};
use justact_policy::datalog;
use justact_policy::datalog::ast::{datalog, AtomArg, Rule};
use stackvec::StackVec;


/***** LIBRARY *****/
/// Defines a [`Message`](justact::Message) that carries $Datalog^\neg$ policy information.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Message {
    /// Some ID of this message.
    id:     &'static str,
    /// The author of this message.
    author: &'static str,
    /// Some policy that is emitted here.
    policy: datalog::Policy,
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
    pub fn new(id: &'static str, author: &'static str, policy: datalog::Policy) -> Self { Self { id, author, policy } }
}

impl Display for Message {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult { write!(f, "{}", self.policy) }
}

impl justact::Message for Message {
    type Author = &'static str;
    type Id = &'static str;

    #[inline]
    fn author(&self) -> Self::Author { self.author }

    #[inline]
    fn id(&self) -> Self::Id { self.id }
}



/// Implements a _meaningful_ collection of $Datalog^\neg$ policy messages.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MessageSet<'m> {
    /// In case there's exactly nothing in the set.
    Empty,
    /// In case there's exactly one message, to prevent allocation.
    Singleton(&'m Message),
    /// In case there's multiple messages.
    Multi(HashSet<&'m Message>),
}
impl<'m> Default for MessageSet<'m> {
    #[inline]
    fn default() -> Self { Self::Empty }
}
impl<'m> MessageSet<'m> {
    /// Merges the given MessageSet into this one.
    ///
    /// # Arguments
    /// - `other`: Some other MessageSet to join.
    #[inline]
    pub fn join(&mut self, other: impl IntoIterator<Item = &'m Message>) {
        *self = match *self {
            Self::Empty => Self::from_iter(other),
            Self::Singleton(msg) => {
                let iter = other.into_iter();
                let size_hint: (usize, Option<usize>) = iter.size_hint();

                // Create a set with both self message and the other
                let mut msgs: HashSet<&'m Message> = HashSet::with_capacity(1 + size_hint.1.unwrap_or(size_hint.0));
                msgs.insert(msg);
                msgs.extend(iter);

                // Return it. Even if the set is only 1 (i.e., the other was empty or duplicate), we still insert as multi to not waste the allocation.
                Self::Multi(msgs)
            },
            Self::Multi(mut msgs) => {
                msgs.extend(other);
                Self::Multi(msgs)
            },
        }
    }

    /// Returns a clone of this MessageSet that borrows all rules from this MessageSet.
    ///
    /// # Returns
    /// A [`MessageSet<'s>`].
    #[inline]
    pub fn as_borrow(&self) -> MessageSet {
        match self {
            Self::Singleton(msg) => MessageSet::Singleton(msg.as_borrow()),
            Self::Multi(msgs) => MessageSet::Multi(msgs.iter().map(Message::as_borrow).collect()),
        }
    }

    /// Returns a clone of this MessageSet that owns all of its rules.
    ///
    /// # Returns
    /// A [`MessageSet<'static>`].
    #[inline]
    pub fn to_owned(&self) -> MessageSet<'static> {
        match self {
            Self::Singleton(msg) => MessageSet::Singleton(msg.to_owned()),
            Self::Multi(msgs) => MessageSet::Multi(msgs.iter().map(Message::to_owned).collect()),
        }
    }
}

impl<'p> Display for MessageSet<'p> {
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> FResult {
        writeln!(f, "MessageSet {{")?;
        for msg in self {
            writeln!(f, "    Message {{")?;
            for rule in &msg.policy.0 {
                writeln!(f, "        {rule}")?;
            }
            writeln!(f, "    }}")?;
        }
        writeln!(f, "}}")
    }
}
impl<'p> Hash for MessageSet<'p> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // Collect everything in an ordered fashion, then commit to some deterministic order
        let mut buf: StackVec<64, &Message> = StackVec::new();
        for msg in self {
            buf.push(msg);
        }
        buf.sort_by_key(|msg| msg.id);

        // Hash that
        for msg in buf {
            msg.hash(state);
        }
    }
}

impl<'p> Collection<Message<'p>> for MessageSet<'p> {
    type Iter<'s> = MessageIter<'s, 'p> where Message<'p>: 's;

    #[inline]
    fn iter<'s>(&'s self) -> Self::Iter<'s> {
        match self {
            Self::Singleton(msg) => MessageIter::Singleton(msg),
            Self::Multi(msgs) => MessageIter::Multi(msgs.iter()),
        }
    }
}
impl<'p> CollectionMut<Message<'p>> for MessageSet<'p> {
    #[inline]
    fn add(&mut self, elem: Message<'p>) -> bool {
        let mut existed;
        *self = match *self {
            Self::Singleton(msg) => {
                existed = msg == elem;
                Self::Multi(HashSet::from([msg, elem]))
            },
            Self::Multi(mut msgs) => {
                existed = msgs.insert(elem);
                Self::Multi(msgs)
            },
        };
        existed
    }
}
impl<'p> justact::Message for MessageSet<'p> {
    type Author = String;
    type Id = String;

    fn id(&self) -> Self::Id { self.iter().map(Message::id).collect::<Vec<&'static str>>().join(", ") }

    fn author(&self) -> Self::Author { self.iter().map(Message::author).collect::<Vec<&'static str>>().join(", ") }
}
impl<'p> justact::MessageSet for MessageSet<'p> {
    type Message = Message<'p>;
    type Policy<'s> = datalog::Policy<'s> where Self: 's;

    fn from_singleton(msg: Self::Message) -> Self {}

    fn from_singleton_ref(msg: &Self::Message) -> Self {}

    fn extract<'s>(&'s self) -> Self::Policy<'s> {
        // Collect the rules of all the messages
        let mut rules: Vec<Cow<Rule>> = Vec::with_capacity(self.len());
        for msg in self {
            for rule in &msg.policy.0 {
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
                    if first_arg.ident().value.value() != self.author() {
                        violation = true;
                        break;
                    }
                }

                // Now add the rule if it's OK, or else add an always-error rule
                if !violation {
                    rules.push(Cow::Borrowed(rule.as_ref()));
                } else {
                    rules.push(Cow::Owned(datalog! { #![crate = "::justact_policy::datalog"] error. }.rules.swap_remove(0)))
                }
            }
        }

        // OK, simply return the inner ones
        datalog::Policy(rules)
    }
}

impl<'p> IntoIterator for MessageSet<'p> {
    type IntoIter = IntoMessageIter<'p>;
    type Item = Message<'p>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        match self {
            Self::Singleton(msg) => IntoMessageIter::Singleton(msg),
            Self::Multi(msgs) => IntoMessageIter::Multi(msgs.into_iter()),
        }
    }
}
impl<'s, 'p> IntoIterator for &'s MessageSet<'p> {
    type IntoIter = MessageIter<'s, 'p>;
    type Item = &'s Message<'p>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter { self.iter() }
}



/// Implements a justified set of statements.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Action<'p> {
    /// The basis of this action.
    pub basis: Message<'p>,
    /// The justification of this action.
    pub justification: MessageSet<'p>,
    /// The enactment of this action.
    pub enactment: Message<'p>,
}
impl<'p> Action<'p> {
    /// Returns a clone of this Action that borrows all rules from this Action.
    ///
    /// # Returns
    /// A [`Action<'s>`].
    #[inline]
    pub fn as_borrow(&self) -> Action {
        Action { basis: self.basis.as_borrow(), justification: self.justification.as_borrow(), enactment: self.enactment.as_borrow() }
    }

    /// Returns a clone of this Action that owns all of its rules.
    ///
    /// # Returns
    /// A [`Action<'static>`].
    #[inline]
    pub fn to_owned(&self) -> Action<'static> {
        Action { basis: self.basis.to_owned(), justification: self.justification.to_owned(), enactment: self.enactment.to_owned() }
    }
}

impl<'p> Display for Action<'p> {
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> FResult {
        // Write the Action header
        writeln!(f, "Action {{")?;

        // Write the basis
        writeln!(f, "    Basis {{")?;
        for rule in MessageSet::from_singleton_ref(&self.basis).extract().0 {
            writeln!(f, "        {rule}")?;
        }
        writeln!(f, "    }}")?;

        // Write the justificaiton
        writeln!(f, "    Justification {{")?;
        for rule in &self.justification.extract().0 {
            writeln!(f, "        {rule}")?;
        }
        writeln!(f, "    }}")?;

        // Write the enactment
        writeln!(f, "    Enactment {{")?;
        for rule in MessageSet::from_singleton_ref(&self.enactment).extract().0 {
            writeln!(f, "        {rule}")?;
        }
        writeln!(f, "    }}")?;

        // Write footer
        writeln!(f, "}}")
    }
}

impl<'p> justact::Action for Action<'p> {
    type Message = Message<'p>;
    type MessageSet = MessageSet<'p>;

    #[inline]
    fn basis(&self) -> &Self::Message { &self.basis }

    #[inline]
    fn justification(&self) -> &Self::MessageSet { &self.justification }

    #[inline]
    fn enactment(&self) -> &Self::Message { &self.enactment }
}
