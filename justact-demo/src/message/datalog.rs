//  DATALOG.rs
//    by Lut99
//
//  Created:
//    15 Apr 2024, 16:22:55
//  Last edited:
//    16 Apr 2024, 16:49:52
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
pub struct Message<'p> {
    /// Some ID of this message.
    id:     &'static str,
    /// The author of this message.
    author: &'static str,
    /// Some policy that is emitted here.
    policy: datalog::Policy<'p>,
}
impl<'p> Message<'p> {
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
    pub fn new(id: &'static str, author: &'static str, policy: datalog::Policy<'p>) -> Self { Self { id, author, policy } }

    /// Returns a clone of this Message that borrows all rules from this Message.
    ///
    /// # Returns
    /// A [`Message<'s>`].
    #[inline]
    pub fn as_borrow(&self) -> Message { Message { id: self.id, author: self.author, policy: self.policy.as_borrow() } }

    /// Returns a clone of this Message that owns all of its rules.
    ///
    /// # Returns
    /// A [`Message<'static>`].
    #[inline]
    pub fn to_owned(&self) -> Message<'static> { Message { id: self.id, author: self.author, policy: self.policy.to_owned() } }
}

impl<'p> justact::Message for Message<'p> {
    type Author = &'static str;
    type Id = &'static str;

    #[inline]
    fn author(&self) -> Self::Author { self.author }

    #[inline]
    fn id(&self) -> Self::Id { self.id }
}



/// Implements a _meaningful_ collection of $Datalog^\neg$ policy messages.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MessageSet<'p> {
    /// The actual messages wrapped.
    messages: HashSet<Message<'p>>,
}
impl<'p> Default for MessageSet<'p> {
    #[inline]
    fn default() -> Self { Self::empty() }
}
impl<'p> MessageSet<'p> {
    /// Constructs an empty MessageSet.
    ///
    /// # Returns
    /// A MessageSet without anything in it.
    #[inline]
    pub fn empty() -> Self { Self { messages: HashSet::new() } }

    /// Merges the given MessageSet into this one.
    ///
    /// # Arguments
    /// - `other`: Some other MessageSet to join.
    #[inline]
    pub fn join(&mut self, other: impl IntoIterator<Item = Message<'p>>) { self.messages.extend(other); }

    /// Returns a clone of this MessageSet that borrows all rules from this MessageSet.
    ///
    /// # Returns
    /// A [`MessageSet<'s>`].
    #[inline]
    pub fn as_borrow(&self) -> MessageSet { MessageSet { messages: self.messages.iter().map(Message::as_borrow).collect() } }

    /// Returns a clone of this MessageSet that owns all of its rules.
    ///
    /// # Returns
    /// A [`MessageSet<'static>`].
    #[inline]
    pub fn to_owned(&self) -> MessageSet<'static> { MessageSet { messages: self.messages.iter().map(Message::to_owned).collect() } }
}

impl<'p> Display for MessageSet<'p> {
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> FResult {
        writeln!(f, "MessageSet {{")?;
        for msg in &self.messages {
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
        for msg in &self.messages {
            buf.push(msg);
        }
        buf.sort_by_key(|msg| msg.id);

        // Hash that
        for msg in buf {
            msg.hash(state);
        }
    }
}

impl<'p> Collection<Message<'p>> for MessageSet<'p> {}
impl<'p> CollectionMut<Message<'p>> for MessageSet<'p> {
    #[inline]
    fn add(&mut self, elem: Message<'p>) -> bool { self.messages.insert(elem) }
}
impl<'p> justact::MessageSet for MessageSet<'p> {
    type Message = Message<'p>;
    type Policy<'s> = datalog::Policy<'s> where Self: 's;

    fn extract<'s>(&'s self) -> Self::Policy<'s> {
        // Combine all the policies
        let mut rules: Vec<Cow<'s, Rule>> = Vec::with_capacity(self.messages.len());
        for message in &self.messages {
            rules.reserve(message.policy.0.len());
            for rule in &message.policy.0 {
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
                    if first_arg.ident().value.value() != message.author() {
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
        datalog::Policy(rules)
    }
}

impl<'p> IntoIterator for MessageSet<'p> {
    type IntoIter = std::collections::hash_set::IntoIter<Message<'p>>;
    type Item = Message<'p>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter { self.messages.into_iter() }
}

impl<'p> From<Message<'p>> for MessageSet<'p> {
    #[inline]
    fn from(value: Message<'p>) -> Self { Self { messages: HashSet::from([value]) } }
}



/// Implements a justified set of statements.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Action<'p> {
    /// The basis of this action.
    pub basis: MessageSet<'p>,
    /// The justification of this action.
    pub justification: MessageSet<'p>,
    /// The enactment of this action.
    pub enactment: MessageSet<'p>,
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
        for rule in &self.basis.extract().0 {
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
        for rule in &self.enactment.extract().0 {
            writeln!(f, "        {rule}")?;
        }
        writeln!(f, "    }}")?;

        // Write footer
        writeln!(f, "}}")
    }
}

impl<'p> justact::Action for Action<'p> {
    type MessageSet = MessageSet<'p>;

    #[inline]
    fn basis(&self) -> &Self::MessageSet { &self.basis }

    #[inline]
    fn justification(&self) -> &Self::MessageSet { &self.justification }

    #[inline]
    fn enactment(&self) -> &Self::MessageSet { &self.enactment }
}
