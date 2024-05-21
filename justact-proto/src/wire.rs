//  WIRE.rs
//    by Lut99
//
//  Created:
//    13 May 2024, 19:15:18
//  Last edited:
//    21 May 2024, 15:47:57
//  Auto updated?
//    Yes
//
//  Description:
//!   Provides concrete implementations for JustAct's abstract traits that
//!   live on the wire.
//

use std::borrow::Cow;
use std::cmp::Ordering;
use std::fmt::{Display, Formatter, Result as FResult};
use std::hash::{Hash, Hasher};

use console::style;
use justact_core::auxillary::{Authored, Identifiable};
use justact_core::policy::ExtractablePolicy;
use justact_core::set::{Map as _, Set as _};
use justact_core::wire as justact;
use justact_core::wire::{Action as _, Message as _, MessageSet as _};

use crate::global::{AgreementsView, Timestamp};
use crate::local::StatementsView;
use crate::set::{Set, SetIter};
use crate::sync::Synchronizer;


/***** FORMATTERS *****/
/// Formats a [`Message`] with proper indentation and such.
pub struct MessageFormatter<'m, 'p, 'i> {
    /// The message to format.
    msg:    &'m Message,
    /// Some prefix to use when writing the message.
    prefix: &'p str,
    /// The indentation to use while formatting.
    indent: &'i str,
}
impl<'m, 'p, 'i> Display for MessageFormatter<'m, 'p, 'i> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        // First, get the message's payload as UTF-8
        let spayload: Cow<str> = String::from_utf8_lossy(self.msg.payload());

        // Write the message
        writeln!(f, "{} '{}' by '{}' {{", self.prefix, style(self.msg.id()).bold(), style(self.msg.author()).bold())?;
        writeln!(f, "{}    {}", self.indent, spayload.replace('\n', &format!("\n{}    ", self.indent)).trim_end())?;
        writeln!(f, "{}}}", self.indent)
    }
}

/// Formats a [`MessageSet`] with proper indentation and such.
pub struct MessageSetFormatter<'m1, 'm2, 'p, 'i> {
    /// The message to format.
    msgs:   &'m1 MessageSet<'m2>,
    /// Some prefix to use when writing the message.
    prefix: &'p str,
    /// The indentation to use while formatting.
    indent: &'i str,
}
impl<'m1, 'm2, 'p, 'i> Display for MessageSetFormatter<'m1, 'm2, 'p, 'i> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        // Write the messages, one-by-one
        writeln!(f, "{} {{", self.prefix)?;
        for msg in self.msgs {
            writeln!(f, "{}    {}", self.indent, msg.display("Message", &format!("{}    ", self.indent)))?;
        }
        writeln!(f, "{}}}", self.indent)
    }
}

/// Formats an [`Agreement`] with proper indentation and such.
pub struct AgreementFormatter<'a, 'p, 'i> {
    /// The agreement to format.
    msg:    &'a Agreement,
    /// Some prefix to use when writing the message.
    prefix: &'p str,
    /// The indentation to use while formatting.
    indent: &'i str,
}
impl<'a, 'p, 'i> Display for AgreementFormatter<'a, 'p, 'i> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        // First, get the agreement's payload as UTF-8
        let spayload: Cow<str> = String::from_utf8_lossy(self.msg.msg.payload());

        // Write the agreement
        writeln!(
            f,
            "{} '{}' by '{}' (applies at {}) {{",
            self.prefix,
            style(self.msg.id()).bold(),
            style(self.msg.msg.author()).bold(),
            style(self.msg.timestamp).bold(),
        )?;
        writeln!(f, "{}    ", spayload.replace('\n', &format!("\n{}    ", self.indent)).trim_end())?;
        writeln!(f, "{}}}", self.indent)
    }
}





/***** AUXILLARY *****/
/// Explains why the [`Action`] failed an audit.
#[derive(Debug)]
pub enum AuditExplanation<E1, E2> {
    /// One of the messages in the action was not stated (property 3).
    Stated { stmt: &'static str },
    /// Failed to extract the policy from the justification (property 5).
    Extract { err: E1 },
    /// The policy was not valid (property 5).
    Valid { expl: E2 },
    /// The basis was not an agreement (property 6).
    Based { stmt: &'static str },
    /// The basis was an agreement but not one for the action's taken time (property 6).
    Timely { stmt: &'static str, applies_at: Timestamp, taken_at: Timestamp },
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
impl Message {
    // /// Returns a [`MessageRef`] from the [`Message`].
    // ///
    // /// # Returns
    // /// A [`MessageRef`] that borrows the data from this [`Message`] and, importantly, implements [`justact::Message`] with lifetimes skipping the reference lifetimes.
    // #[inline]
    // pub fn as_ref(&self) -> MessageRef { MessageRef { id: self.id, author: self.author, data: self.data.as_slice() } }

    /// Returns a formatter that displays the message.
    ///
    /// # Arguments
    /// - `prefix`: Some to call the message, e.g., `Message`.
    /// - `indent`: The indentation (as a concrete string to write) to apply before every newline.
    ///
    /// # Returns
    /// A [`MessageFormatter`] that does not write additional indentation.
    #[inline]
    pub fn display<'s, 'p, 'i>(&'s self, prefix: &'p str, indent: &'i str) -> MessageFormatter<'s, 'p, 'i> {
        MessageFormatter { msg: self, prefix, indent }
    }
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
    type Id = str;

    #[inline]
    fn id(&self) -> &Self::Id { &self.id }
}
impl Authored for Message {
    type AuthorId = str;

    #[inline]
    fn author(&self) -> &Self::AuthorId { &self.author }
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
            set.msgs.add(Cow::Borrowed(msg.as_ref()));
        }
        set
    }

    /// Returns a formatter that displays the set.
    ///
    /// # Arguments
    /// - `prefix`: Something to call the set, e.g., `MessageSet`.
    /// - `indent`: The indentation (as a concrete string to write) to apply before every newline.
    ///
    /// # Returns
    /// A [`MessageFormatter`] that does not write additional indentation.
    #[inline]
    pub fn display<'s, 'p, 'i>(&'s self, prefix: &'p str, indent: &'i str) -> MessageSetFormatter<'s, 'm, 'p, 'i> {
        MessageSetFormatter { msgs: self, prefix, indent }
    }
}

impl<'m> justact_core::Set<Message> for MessageSet<'m> {
    type Item<'s> = &'m Message where Self: 's;
    type Iter<'s> = std::iter::Map<SetIter<'s, Cow<'m, Message>>, fn(&'s Cow<'m, Message>) -> &'m Message> where Self: 's;

    #[inline]
    fn add(&mut self, new_elem: Message) -> bool { self.msgs.add(Cow::Owned(new_elem)) }

    #[inline]
    fn iter<'s>(&'s self) -> Self::Iter<'s> {
        self.msgs.iter().map(|e| if let Cow::Borrowed(e) = e { e } else { panic!("This trick only works when all elements are borrowed...") })
    }

    #[inline]
    fn len(&self) -> usize { self.msgs.len() }
}
impl<'m> justact_core::Map<Message> for MessageSet<'m> {
    #[inline]
    fn get(&self, id: &<Message as Identifiable>::Id) -> Option<&Message> { self.msgs.get(id).map(|m| m.as_ref()) }
}
impl<'m> justact::MessageSet for MessageSet<'m> {
    type Message = Message;
}

impl<'m> IntoIterator for MessageSet<'m> {
    type Item = Cow<'m, Message>;
    type IntoIter = <Set<Cow<'m, Message>> as IntoIterator>::IntoIter;

    #[inline]
    fn into_iter(self) -> Self::IntoIter { self.msgs.into_iter() }
}
impl<'s, 'm> IntoIterator for &'s MessageSet<'m> {
    type Item = <MessageSet<'m> as justact_core::Set<Message>>::Item<'s>;
    type IntoIter = <MessageSet<'m> as justact_core::Set<Message>>::Iter<'s>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter { <MessageSet<'m> as justact_core::Set<Message>>::iter(self) }
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
    /// The message (statement, rather) set that was agreed upon.
    pub msg: Message,
}
impl Agreement {
    /// Returns a formatter that displays the agreement.
    ///
    /// # Arguments
    /// - `prefix`: Some to call the agreement, e.g., `Agreement`.
    /// - `indent`: The indentation (as a concrete string to write) to apply before every newline.
    ///
    /// # Returns
    /// A [`AgreementFormatter`] that does not write additional indentation.
    #[inline]
    pub fn display<'s, 'p, 'i>(&'s self, prefix: &'p str, indent: &'i str) -> AgreementFormatter<'s, 'p, 'i> {
        AgreementFormatter { msg: self, prefix, indent }
    }
}
impl Identifiable for Agreement {
    type Id = <Message as Identifiable>::Id;

    #[inline]
    fn id(&self) -> &Self::Id { self.msg.id() }
}
impl Authored for Agreement {
    type AuthorId = <Message as Authored>::AuthorId;

    #[inline]
    fn author(&self) -> &Self::AuthorId { &self.msg.id }
}
impl justact::Agreement for Agreement {
    type Message = Message;
    type Time = Timestamp;

    #[inline]
    fn statements(&self) -> &Self::Message { &self.msg }

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
    fn id(&self) -> &Self::Id { &self.enactment.id }
}
impl Authored for Action {
    type AuthorId = <Message as Authored>::AuthorId;

    #[inline]
    fn author(&self) -> &Self::AuthorId { &self.enactment.id }
}
impl justact::Action for Action {
    type Time = Timestamp;
    type Agreement = Agreement;
    type MessageSet<'s> = MessageSet<'s>;
    type Message = Message;

    #[inline]
    fn taken_at(&self) -> Self::Time { self.timestamp }

    #[inline]
    fn basis(&self) -> &Self::Agreement { &self.basis }

    fn justification<'s>(&'s self) -> Self::MessageSet<'s> {
        // Clone the internal message set by borrow
        let mut set: MessageSet<'s> = self.justification.borrow();

        // Inject the basis & enactment into it
        set.msgs.add(Cow::Borrowed(&self.basis.msg));
        set.msgs.add(Cow::Borrowed(&self.enactment));

        // OK, return the full justification
        set
    }

    #[inline]
    fn enacts(&self) -> &Self::Message { &self.enactment }
}

impl Action {
    /// Audits this action.
    ///
    /// This will check whether this action meets all the properties specified in the framework paper:
    /// - Property 3: Stated (all messages must be stated);
    /// - Property 5: Valid (the extracted policy of the justification must be valid); and
    /// - Property 6: Based (the basis of the action was an agreement valid at the time it was taken).
    ///
    /// Note that Property 4, Relevant (the basis and enactment are in the justification) is trivially guaranteed by the implementation of [`Action::justification()`].
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
        P: ExtractablePolicy<&'s Message>,
    {
        let just: MessageSet = self.justification();

        /* Property 3 */
        // Checks if the policy is stated correctly.
        for stmt in &just {
            if !stmts.contains(stmt.id()) {
                return Err(AuditExplanation::Stated { stmt: stmt.id });
            }
        }



        /* Property 4 */
        // Checks if the basis and enactment are included in the justification
        // Trivial due to how we created the action



        /* Property 5 */
        // Attempt to extract the policy
        let policy: P = match just.extract::<P>() {
            Ok(policy) => policy,
            Err(err) => return Err(AuditExplanation::Extract { err }),
        };

        // Check if the policy is valid
        if let Err(expl) = policy.check_validity() {
            return Err(AuditExplanation::Valid { expl });
        }



        /* Property 6 */
        // Assert that the basis is an agreement
        if !agrmnts.contains(self.basis.id()) {
            return Err(AuditExplanation::Based { stmt: self.basis.msg.id });
        }

        // Assert the agreement's time matches the action's
        if self.basis.timestamp != self.timestamp {
            return Err(AuditExplanation::Timely { stmt: self.basis.msg.id, applies_at: self.basis.timestamp, taken_at: self.timestamp });
        }



        /* Success */
        Ok(())
    }
}
