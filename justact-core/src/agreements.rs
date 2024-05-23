//  AGREEMENTS.rs
//    by Lut99
//
//  Created:
//    23 May 2024, 11:27:32
//  Last edited:
//    23 May 2024, 13:35:04
//  Auto updated?
//    Yes
//
//  Description:
//!   Implements the globally synchronized set of timestamps, including
//!   which one is the current one.
//

use std::error::Error;

use crate::auxillary::{Authored, Identifiable};
use crate::set::Set;
use crate::statements::Message;
use crate::times::Timestamp;


/***** LIBRARY *****/
/// Implements an [`Agreement`], which is like a message plus some timestamp that relates to when it was valid.
///
/// # Generics
/// - `M`: The concrete type of the [`Message`] stored in the agreement.
/// - `T`: The concrete type of the [`Time`]stamp stored in the agreement.
#[derive(Clone, Copy, Debug)]
pub struct Agreement<M> {
    /// The (stated!) message that was agreed upon.
    pub msg: M,
    /// The timestamp indicating when this message is OK to be used as basis for actions.
    pub timestamp: Timestamp,
}
impl<M> Agreement<M> {
    /// Returns when the agreement applied, i.e., for which time it may be used as basis for [`Action`](crate::statements::Action)s.
    ///
    /// # Returns
    /// The internal [`Timestamp`].
    pub fn applies_at(&self) -> Timestamp { self.timestamp }
}

impl<'v, M: 'v + Identifiable<'v>> Identifiable<'v> for Agreement<M> {
    type Id = M::Id;

    #[inline]
    fn id(&self) -> &'v Self::Id { self.msg.id() }
}
impl<'v, M: 'v + Authored<'v>> Authored<'v> for Agreement<M> {
    type AuthorId = M::AuthorId;

    #[inline]
    fn author(&self) -> &'v Self::AuthorId { self.msg.author() }
}



/// Defines the total set of [`Agreement`]s that agents (collaberatively) agree on as a common basis.
///
/// This is a _globally synchronized_ set, meaning that the framework requires agents to be in
/// agreement at all times about this set's contents.
pub trait Agreements {
    /// The type of [`Message`]s that are agreed upon in the form of [`Agreement`]s.
    type Message<'s>: Message<'s>
    where
        Self: 's;
    /// The type of errors returned by this set.
    type Error: Error;


    /// Agrees on a new message.
    ///
    /// # Arguments
    /// - `agr`: The [`Agreement<Self::Message>`]-like to agree on.
    ///
    /// # Errors
    /// This function errors if it failed to synchronize the agreement to all other agents, either
    /// because they could not be updated (synchronization) or did not agree with it (consensus).
    fn agree<'s>(&'s mut self, agr: impl Into<Agreement<Self::Message<'s>>>) -> Result<(), Self::Error>;

    /// Returns an agreement set with all agreements in this Agreements.
    ///
    /// # Returns
    /// A [`Set`] that contains all the agreements in this Agreements.
    fn agreed<'s>(&'s self) -> Set<Agreement<Self::Message<'s>>>;
}
