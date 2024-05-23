//  AGREEMENTS.rs
//    by Lut99
//
//  Created:
//    23 May 2024, 11:27:32
//  Last edited:
//    23 May 2024, 11:55:24
//  Auto updated?
//    Yes
//
//  Description:
//!   Implements the globally synchronized set of timestamps, including
//!   which one is the current one.
//

use crate::auxillary::{Authored, Identifiable};
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
