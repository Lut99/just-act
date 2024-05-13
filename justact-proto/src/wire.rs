//  WIRE.rs
//    by Lut99
//
//  Created:
//    13 May 2024, 19:15:18
//  Last edited:
//    13 May 2024, 19:34:53
//  Auto updated?
//    Yes
//
//  Description:
//!   Provides concrete implementations for JustAct's abstract traits that
//!   live on the wire.
//

use std::borrow::Cow;
use std::cmp::Ordering;
use std::hash::{Hash, Hasher};

use justact_core::auxillary::{Authored, Identifiable};
use justact_core::wire as justact;

use crate::global::Timestamp;
use crate::set::Set;


/***** LIBRARY *****/
/// Represents a [`justact::Message`] in the prototype simulation.
#[derive(Clone, Debug)]
pub struct Message {
    /// The identifier of the message, by string ID.
    pub id:     &'static str,
    /// The author of the message, by string ID.
    pub author: &'static str,
    /// Some bytes carried by the message.
    pub data:   Vec<u8>,
}

impl Eq for Message {}
impl Hash for Message {
    fn hash<H: Hasher>(&self, state: &mut H) { self.id.hash(state) }
}
impl Ord for Message {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering { self.id.cmp(other.id) }
}
impl PartialEq for Message {
    #[inline]
    fn eq(&self, other: &Self) -> bool { self.id == other.id }
}
impl PartialOrd for Message {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { self.id.partial_cmp(other.id) }
}

impl Identifiable for Message {
    type Id = &'static str;

    #[inline]
    fn id(&self) -> Self::Id { self.id }
}
impl Authored for Message {
    type AuthorId = &'static str;

    #[inline]
    fn author(&self) -> Self::AuthorId { self.author }
}
impl justact::Message for Message {
    #[inline]
    fn payload(&self) -> &[u8] { self.data.as_slice() }
}



/// Represents a set of [`Message`]s in the prototype simulation.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct MessageSet {
    /// The actual messages in the set.
    msgs: Set<Message>,
}

impl justact_core::set::Set<Message> for MessageSet {
    type Item<'s> = <Set<Message> as justact_core::set::Set<Message>>::Item<'s>;
    type Iter<'s> = <Set<Message> as justact_core::set::Set<Message>>::Iter<'s>;

    #[inline]
    fn add(&mut self, new_elem: Message) -> bool { self.msgs.add(new_elem) }
    #[inline]
    fn get(&self, id: <Message as Identifiable>::Id) -> Option<&Message>
    where
        Message: Identifiable,
    {
        self.msgs.get(id)
    }

    #[inline]
    fn iter<'s>(&'s self) -> Self::Iter<'s> { self.msgs.iter() }

    #[inline]
    fn len(&self) -> usize { self.msgs.len() }
}
impl justact::MessageSet for MessageSet {
    type Message = Message;
}

impl From<Message> for MessageSet {
    #[inline]
    fn from(value: Message) -> Self { Self { msgs: Set::<Message>::from(value) } }
}



/// Represents a synchronized [`justact::Agreement`] in the prototype simulation.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Agreement {
    /// The time at which this agreement was valid.
    pub timestamp: Timestamp,
    /// The message set that was agreed upon.
    pub msgs:      MessageSet,
}
impl justact::Agreement for Agreement {
    // NOTE: Returns 'Cow' instead of a reference, because a reference cannot be built from only one Message.
    type MessageSet<'s> = Cow<'s, MessageSet>;
    type Time = Timestamp;

    #[inline]
    fn applies_at(&self) -> Self::Time { self.timestamp }
}
