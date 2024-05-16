//  WIRE.rs
//    by Lut99
//
//  Created:
//    13 May 2024, 19:15:18
//  Last edited:
//    16 May 2024, 17:46:25
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
use justact_core::policy::ExtractablePolicy;
use justact_core::set::{Map as _, Set as _};
use justact_core::wire as justact;
use justact_core::wire::Action as _;

use crate::global::{AgreementsView, Timestamp};
use crate::local::StatementsView;
use crate::set::Set;
use crate::sync::Synchronizer;


/***** AUXILLARY *****/
/// Explains why the [`Action`] failed an audit.
#[derive(Debug)]
pub enum AuditExplanation<E1, E2> {
    /// One of the messages in the action was not stated (property 3).
    Stated { stmt: &'static str },
    /// Failed to extract the policy from the justification (property 4).
    Extract { err: E1 },
    /// The policy was not valid (property 4).
    Valid { expl: E2 },
}





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

impl<'m> justact_core::Set<Cow<'m, Message>> for MessageSet<'m> {
    type Item<'s> = <Set<Cow<'m, Message>> as justact_core::set::Set<Cow<'m, Message>>>::Item<'s> where Self: 's;
    type Iter<'s> = <Set<Cow<'m, Message>> as justact_core::set::Set<Cow<'m, Message>>>::Iter<'s> where Self: 's;

    #[inline]
    fn add(&mut self, new_elem: Cow<'m, Message>) -> bool { self.msgs.add(new_elem) }

    #[inline]
    fn iter<'s>(&'s self) -> Self::Iter<'s> { self.msgs.iter() }

    #[inline]
    fn len(&self) -> usize { self.msgs.len() }
}
impl<'m> justact_core::Map<Cow<'m, Message>> for MessageSet<'m> {
    #[inline]
    fn get(&self, id: <Cow<'m, Message> as Identifiable>::Id) -> Option<&Cow<'m, Message>>
    where
        Message: Identifiable,
    {
        self.msgs.get(id)
    }
}
impl<'m> justact::MessageSet for MessageSet<'m> {
    type Message = Cow<'m, Message>;
}

impl<'m> IntoIterator for MessageSet<'m> {
    type Item = <Set<<Self as justact::MessageSet>::Message> as IntoIterator>::Item;
    type IntoIter = <Set<<Self as justact::MessageSet>::Message> as IntoIterator>::IntoIter;

    #[inline]
    fn into_iter(self) -> Self::IntoIter { self.msgs.into_iter() }
}
impl<'s, 'm> IntoIterator for &'s MessageSet<'m> {
    type Item = <&'s Set<<Self as justact::MessageSet>::Message> as IntoIterator>::Item;
    type IntoIter = <&'s Set<<Self as justact::MessageSet>::Message> as IntoIterator>::IntoIter;

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
impl Identifiable for Action {
    type Id = &'static str;

    #[inline]
    fn id(&self) -> Self::Id { self.enactment.id }
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

impl Action {
    /// Audits this action.
    ///
    /// This will check whether this action meets all the properties specified in the framework paper:
    /// - Property 3: Stated (all messages must be stated);
    /// - Property 5: Valid (the extracted policy of the justification must be valid); and
    /// - Property 6: Based (the basis of the action was an agreement valid at the time it was taken).
    ///
    /// Note that Property 4, relevant (the basis and enactment are in the justification) is trivially guaranteed by the implementation of [`Action::justification()`].
    ///
    /// # Arguments
    /// - `agrmnts`: An [`AgreementsView`] that provides access to the globally synchronized agreements.
    /// - `stmts`: A [`StatementsView`] that provides access to the messages this agent knows are stated.
    ///
    /// # Errors
    /// This function errors if the audit failed. Which property is violated, and how, is explained by the returned [`AuditExplanation`].
    #[inline]
    pub fn audit<'s, S, P>(
        &'s self,
        agrmnts: &AgreementsView<S>,
        stmts: &StatementsView,
    ) -> Result<(), AuditExplanation<P::ExtractError, P::Explanation>>
    where
        S: Synchronizer<Agreement>,
        S::Error: 'static,
        P: ExtractablePolicy<<MessageSet<'s> as IntoIterator>::IntoIter>,
    {
        let just: MessageSet = self.justification();

        /* Property 3 */
        // Checks if the policy is stated correctly.
        for stmt in &just {
            if !stmts.contains(stmt.id()) {
                return Err(AuditExplanation::Stated { stmt: stmt.id() });
            }
        }



        /* Property 4 */
        // Checks if the basis and enactment are included in the justification
        // Trivial due to how we created the action



        /* Property 5 */
        // Attempt to extract the policy
        let policy: P = match P::extract_from(just.into_iter()) {
            Ok(policy) => policy,
            Err(err) => return Err(AuditExplanation::Extract { err }),
        };

        // Check if the policy is valid
        if let Err(expl) = policy.check_validity() {
            return Err(AuditExplanation::Valid { expl });
        }



        /* Property 6 */
        // Assert that the basis is an agreement
        for agrmnt in agrmnts.iter() {}



        // Done
        Ok(())
    }
}
