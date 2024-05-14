//  WIRE.rs
//    by Lut99
//
//  Created:
//    13 May 2024, 19:15:18
//  Last edited:
//    14 May 2024, 10:20:11
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
use justact_core::set::Set as _;
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
pub struct MessageSet<'m> {
    /// The actual messages in the set.
    pub msgs: Set<Cow<'m, Message>>,
}
impl<'m> MessageSet<'m> {
    /// Creates an identical MessageSet as this one, except that all elements are references to this.
    ///
    /// # Returns
    /// A `MessageSet<'s>`, where `'s` is the lifetime of `'self`.
    #[inline]
    pub fn borrow<'s>(&'s self) -> MessageSet<'s> {
        let mut set: MessageSet = MessageSet { msgs: Set::with_capacity(self.msgs.len()) };
        for msg in &self.msgs {
            set.add(Cow::Borrowed(msg));
        }
        set
    }
}

impl<'m> justact_core::set::Set<Cow<'m, Message>> for MessageSet<'m> {
    type Item<'s> = <Set<Cow<'m, Message>> as justact_core::set::Set<Cow<'m, Message>>>::Item<'s> where Self: 's;
    type Iter<'s> = <Set<Cow<'m, Message>> as justact_core::set::Set<Cow<'m, Message>>>::Iter<'s> where Self: 's;

    #[inline]
    fn add(&mut self, new_elem: Cow<'m, Message>) -> bool { self.msgs.add(new_elem) }
    #[inline]
    fn get(&self, id: <Cow<'m, Message> as Identifiable>::Id) -> Option<&Cow<'m, Message>>
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
impl<'m> justact::MessageSet for MessageSet<'m> {
    type Message = Cow<'m, Message>;
}

impl<'s, 'm> IntoIterator for &'s MessageSet<'m> {
    type Item = <Set<<Self as justact::MessageSet>::Message> as justact_core::set::Set<<Self as justact::MessageSet>::Message>>::Item<'s>;
    type IntoIter = <Set<<Self as justact::MessageSet>::Message> as justact_core::set::Set<<Self as justact::MessageSet>::Message>>::Iter<'s>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter { self.msgs.iter() }
}

impl<'m> From<&'m Message> for MessageSet<'m> {
    #[inline]
    fn from(value: &'m Message) -> Self { Self { msgs: Set::<Cow<'m, Message>>::from(Cow::Borrowed(value)) } }
}
impl<'m> From<Cow<'m, Message>> for MessageSet<'m> {
    #[inline]
    fn from(value: Cow<'m, Message>) -> Self { Self { msgs: Set::<Cow<'m, Message>>::from(value) } }
}
impl<'m> From<Message> for MessageSet<'m> {
    #[inline]
    fn from(value: Message) -> Self { Self { msgs: Set::<Cow<'static, Message>>::from(Cow::Owned(value)) } }
}



/// Represents a synchronized [`justact::Agreement`] in the prototype simulation.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Agreement {
    /// The time at which this agreement was valid.
    pub timestamp: Timestamp,
    /// The message set that was agreed upon.
    pub msgs:      MessageSet<'static>,
}
impl justact::Agreement for Agreement {
    type MessageSet<'s> = &'s MessageSet<'static>;
    type Time = Timestamp;

    #[inline]
    fn statements<'s>(&'s self) -> Self::MessageSet<'s> { &self.msgs }

    #[inline]
    fn applies_at(&self) -> Self::Time { self.timestamp }
}



/// Represents an enacted [`justact::Action`] in the prototype simulation.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Action {
    /// The time the action was taken.
    pub timestamp: Timestamp,
    /// The agreement that forms the basis of this action.
    pub basis: Agreement,
    /// The justificate from the basis to the enactment. This does _not_ include the basis and enactment itself yet.
    pub justification: MessageSet<'static>,
    /// The enacted statement that is justified by this action.
    pub enactment: Message,
}
impl justact::Action for Action {
    type Time = Timestamp;
    type Agreement<'s> = &'s Agreement;
    type MessageSet<'s> = MessageSet<'s>;
    type Message<'s> = &'s Message;

    #[inline]
    fn taken_at(&self) -> Self::Time { self.timestamp }

    #[inline]
    fn basis<'s>(&'s self) -> Self::Agreement<'s> { &self.basis }

    fn justification<'s>(&'s self) -> Self::MessageSet<'s> {
        // Clone the internal message set by borrow
        let mut set: MessageSet<'s> = self.justification.borrow();

        // Inject the basis & enactment into it
        for msg in &self.basis.msgs {
            set.add(Cow::Borrowed(msg));
        }
        set.add(Cow::Borrowed(&self.enactment));

        // OK, return the full justification
        set
    }

    #[inline]
    fn enacts<'s>(&'s self) -> Self::Message<'s> { &self.enactment }
}