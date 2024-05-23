//  STATEMENTS.rs
//    by Lut99
//
//  Created:
//    21 May 2024, 16:48:17
//  Last edited:
//    23 May 2024, 13:37:06
//  Auto updated?
//    Yes
//
//  Description:
//!   Implements the globally synchronized set of stated messages.
//

use std::error::Error;

use crate::agreements::{Agreement, Agreements};
use crate::auxillary::{Authored, Identifiable};
use crate::set::Set;
use crate::times::Timestamp;


/***** AUXILLARY *****/
/// Explains why an audit of an [`Action`] in a [`Statements`] failed.
///
/// # Generics
/// - `ID`: The identifier used by messages.
/// - `SYN`: The [`Extractable::SyntaxError`] of the policy language that was potentially erronously extracted.
/// - `SEM`: The [`Policy::SemanticError`] of the policy language that was potentially invalid.
#[derive(Debug)]
pub enum AuditExplanation<ID, SYN, SEM> {
    /// One of the messages in the action was not stated (property 3).
    Stated { stmt: ID },
    /// Failed to extract the policy from the justification (property 5).
    Extract { err: SYN },
    /// The policy was not valid (property 5).
    Valid { expl: SEM },
    /// The basis was not an agreement (property 6).
    Based { stmt: ID },
    /// The basis was an agreement but not one for the action's taken time (property 6).
    Timely { stmt: ID, applies_at: Timestamp, taken_at: Timestamp },
}





/***** LIBRARY *****/
/// Defines the framework's notion of policy.
///
/// This is usually accompanied by [`Extractable`] in order to communicate that policy can be
/// extracted from message sets.
///
/// # Generics
/// - `'v`: The lifetime of the [`SystemView`](crate::SystemView) where the policy's data lives.
pub trait Policy<'v> {
    /// The type of error emitted when the policy is not valid (**semantically** incorrect).
    type SemanticError: Error;

    /// Checks whether this policy is valid according to its own semantics.
    ///
    /// # Errors
    /// If the policy is not valid, this function errors. The resulting
    /// [`Self::SemanticError`](Policy::SemanticError) encodes some explanation of why the policy
    /// wasn't valid.
    fn assert_validity(&self) -> Result<(), Self::SemanticError>;
}

/// Extends [`Policy`] with the power to be extracted from [`MessageSet`]s.
///
/// # Generics
/// - `'v`: The lifetime of the [`SystemView`](crate::SystemView) where the set's (and therefore
///   resulting policy's) data lives.
pub trait Extractable<'v> {
    /// The type of error emitted when the policy is **syntactically** incorrect.
    type SyntaxError: Error;

    /// Extracts this policy from a given [`Set`] over messages.
    ///
    /// # Arguments
    /// - `set`: The [`Set`] to extract from.
    ///
    /// # Returns
    /// A new instance of Self which represents the parsed policy.
    ///
    /// # Errors
    /// This function should throw a [`Self::SyntaxError`](Extractable::SyntaxError) if and only if
    /// the combined messages' payloads did not make a **syntactically** correct policy.
    ///
    /// Semantic correctness is conventionally modelled by returning a legal policy, but that fails
    /// the [`Policy::check_validity()`]-check.
    fn extract_from<V, R>(set: &Set<V, R>) -> Result<Self, Self::SyntaxError>
    where
        Self: Sized,
        V: Message<'v>;
}



/// Implements a representation of messages in the framework.
///
/// There's a lot of leeway for implementation w.r.t. identifying authors and
/// message identifiers. However, all messages are expected to somehow carry
/// their policies as raw bytes.
///
/// # Generics
/// - `'v`: The lifetime of the [`SystemView`](crate::SystemView) where the message's data lives.
pub trait Message<'v>: Authored<'v> + Identifiable<'v> {
    /// Returns the payload of the message.
    ///
    /// The payload of the message must always be a continious series of raw bytes. What these
    /// bytes mean is up to the relevant agents, who decide which policy language to use.
    ///
    /// # Returns
    /// A byte slice ([`&[u8]`](u8)) that represents the message's full payload.
    fn payload(&self) -> &'v [u8];
}

/// Implements a representation of actions in the framework.
///
/// The implementation for the action is pre-provided, as we expect this to be the same across
/// implementations.
///
/// # Generics
/// - `M`: The concrete type of [`Message`]s stored in the action.
/// - `T`: The concrete type of the [`Time`]stamp stored in the action and its nested basis.
#[derive(Clone, Debug)]
pub struct Action<M> {
    /// The basis, i.e., agreement upon which the action relies.
    pub basis:     Agreement<M>,
    /// The justification that will make the composition of the `basis` and `enactment` [valid](Policy::assert_validity()).
    pub just:      Set<M>,
    /// The enacted statement that encodes the effect of the action.
    pub enacts:    M,
    /// The timestamp encoding when this action was taken.
    pub timestamp: Timestamp,
}
impl<M> Action<M> {
    /// Returns the basis of the action.
    ///
    /// This is the agreement that was valid at the time it was taken (at least, claimed*).
    ///
    /// # Returns
    /// A reference to the internal [`Agreement`].
    #[inline]
    pub fn basis(&self) -> &Agreement<M> { &self.basis }

    /// Returns the enactment of the action.
    ///
    /// This is a statement encoding the effects of the action.
    ///
    /// # Returns
    /// A reference to the internal `M`essage.
    #[inline]
    pub fn enacts(&self) -> &M { &self.enacts }
}
impl<'v, M: 'v + Clone + Identifiable<'v>> Action<M> {
    /// Returns the justification of the action.
    ///
    /// Note that, contrary to accessing `just` manually, this includes both the `basis` _and_ the `enacts`.
    ///
    /// # Returns
    /// A [`Set`] of messages that form the entire justification, including its basis and effects.
    pub fn justification(&self) -> Set<M> {
        // Get the justification first
        let mut just: Set<M> = self.just.clone();
        // Include the agreement
        just.add(self.basis.msg.clone());
        // Include the enactment
        just.add(self.enacts.clone());
        // Done
        just
    }
}
impl<'v, M: 'v + Clone + Message<'v>> Action<M> {
    /// Audits this action, checking whether it satisfies the well-behaved properties specified in
    /// the paper.
    ///
    /// Specifically, it checks:
    ///
    /// # Generics
    /// - `P`: The [`Policy`] language that is used to verify the messages' payload's validity.
    /// - `S`: The type of `stmts` given to check which messages are stated by agents.
    /// - `A`: The type of `agrmtns` given to check which agreements are actually agreed upon.
    ///
    /// # Arguments
    /// - `stmts`: The set of [`Statements`] to which the auditing entity has access.
    ///
    /// # Errors
    /// This function errors if one of the properties does not hold. The returned
    /// [`AuditExplanation`] encodes specifically which one did not.
    pub fn audit<P, S, A>(&self, stmts: &'v S, agrmnts: &'v A) -> Result<(), AuditExplanation<&'v M::Id, P::SyntaxError, P::SemanticError>>
    where
        P: Extractable<'v> + Policy<'v>,
        S: Statements<Message<'v> = M>,
        A: Agreements<Message<'v> = M>,
    {
        let just: Set<M> = self.justification();

        /* Property 3 */
        // Checks if the policy is stated correctly.
        for stmt in &just {
            if !stmts.stated().contains(stmt.id()) {
                return Err(AuditExplanation::Stated { stmt: stmt.id() });
            }
        }



        /* Property 4 */
        // Checks if the basis and enactment are included in the justification
        // Trivial due to [`Action::justification()`]



        /* Property 5 */
        // Attempt to extract the policy
        let policy: P = match just.extract::<P>() {
            Ok(policy) => policy,
            Err(err) => return Err(AuditExplanation::Extract { err }),
        };

        // Check if the policy is valid
        if let Err(expl) = policy.assert_validity() {
            return Err(AuditExplanation::Valid { expl });
        }



        /* Property 6 */
        // Assert that the basis is an agreement
        if !agrmnts.agreed().contains(self.basis.id()) {
            return Err(AuditExplanation::Based { stmt: self.basis.id() });
        }

        // Assert the agreement's time matches the action's
        if self.basis.applies_at() != self.timestamp {
            return Err(AuditExplanation::Timely { stmt: self.basis.id(), applies_at: self.basis.timestamp, taken_at: self.timestamp });
        }



        /* Success */
        Ok(())
    }
}

impl<'v, M: 'v + Identifiable<'v>> Identifiable<'v> for Action<M> {
    type Id = M::Id;

    #[inline]
    fn id(&self) -> &'v Self::Id { self.enacts.id() }
}
impl<'v, M: 'v + Authored<'v>> Authored<'v> for Action<M> {
    type AuthorId = M::AuthorId;

    #[inline]
    fn author(&self) -> &'v Self::AuthorId { self.enacts.author() }
}



/// Defines the set of messages that are stated by agents.
///
/// Note that this set is _local_, meaning that its contents may differ per-agent.
pub trait Statements {
    /// The type of [`Message`]s that can be stated.
    type Message<'s>: Message<'s>
    where
        Self: 's;
    /// The target that specifies who might learn of the statements.
    type Target;
    /// Something describing how successful stating was.
    type Status;


    /// States a new message.
    ///
    /// # Arguments
    /// - `target`: Some specifyer of where the new message should end up (e.g., all other agents,
    ///   a particular subset of agents, ...).
    /// - `msg`: The [`Self::Message`](Statements::Message)-like to state.
    ///
    /// # Returns
    /// This function returns a description of how much of a success the stating was.
    ///
    /// Remember that the statements-set may be partial and incomplete. Depending on
    /// implementations, this means that it is OK for some synchronisations with agents to
    /// succeed, and some of them to fail. As such, this function doesn't have a binary concept
    /// of success like [`Result`] implies; instead, [`Self::Status`](Statements::Status) describes
    /// where on the continuum of success the result lies.
    fn state<'s>(&'s mut self, target: Self::Target, msg: impl Into<Self::Message<'s>>) -> Self::Status;

    /// Returns a message set with the messages in this Statements.
    ///
    /// # Returns
    /// A [`Set`] that contains all the messages in this statements.
    fn stated<'s>(&'s self) -> Set<Self::Message<'s>>;



    /// Enacts a new statement with a justification for it.
    ///
    /// # Arguments
    /// - `target`: Some specifyer of where the enactment should end up (e.g., all other agents, a
    ///   particular subset of agents, ...).
    /// - `act`: The [`Action`]-like to enact.
    ///
    /// # Returns
    /// This function returns a description of how much of a success the enacting was.
    ///
    /// Remember that the statements-set may be partial and incomplete. Depending on
    /// implementations, this means that it is OK for some synchronisations with agents to
    /// succeed, and some of them to fail. As such, this function doesn't have a binary concept
    /// of success like [`Result`] implies; instead, [`Self::Status`](Statements::Status) describes
    /// where on the continuum of success the result lies.
    fn enact<'s>(&'s mut self, target: Self::Target, act: impl Into<Action<Self::Message<'s>>>) -> Self::Status;

    /// Returns an action set with the enacted actions in this Statements.
    ///
    /// # Returns
    /// A [`Set`] that contains all the actions in this statements.
    fn enacted<'s>(&'s self) -> Set<Action<Self::Message<'s>>>;
}
