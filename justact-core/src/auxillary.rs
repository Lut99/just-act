//  AUXILLARY.rs
//    by Lut99
//
//  Created:
//    13 May 2024, 14:16:11
//  Last edited:
//    17 May 2024, 09:50:29
//  Auto updated?
//    Yes
//
//  Description:
//!   Defines some smaller, not-occuring-in-the-paper traits that are very
//!   convenient.
//

use std::borrow::Cow;
use std::hash::Hash;


/***** LIBRARY *****/
/// Something is **uniquely** identifiable by something else.
///
/// Note, however, that the namespace of uniqueness is only for things of the same type (e.g., across messages or across agents).
pub trait Identifiable {
    /// The thing used as identifier. For convenience, we require it to [`Eq`] and [`Hash`].
    type Id: Eq + Hash;

    /// Returns the identifier for this thing.
    ///
    /// Note, however, that the namespace of uniqueness is only for things of the same type (e.g., across messages or across agents).
    ///
    /// # Returns
    /// Something of type `Self::Id` that uniquely identifiers this object.
    fn id(&self) -> &Self::Id;
}

// Implement over some pointer-like types
impl<'a, T: Identifiable> Identifiable for &'a T {
    type Id = T::Id;

    #[inline]
    fn id(&self) -> &Self::Id { T::id(self) }
}
impl<'a, T: Clone + Identifiable> Identifiable for Cow<'a, T> {
    type Id = T::Id;

    #[inline]
    fn id(&self) -> &Self::Id { T::id(self) }
}



/// Something is authored by some agent.
pub trait Authored {
    /// The thing used as identifier of the agent. For convenience, we require it to [`Eq`] and [`Hash`].
    type AuthorId: Eq + Hash;

    /// Returns the unique identifier of the author of this object.
    ///
    /// # Returns
    /// A `Self::Author::Id` that represents the author of this object.
    fn author(&self) -> &Self::AuthorId;
}

// Implement over some pointer-like types
impl<'a, T: Authored> Authored for &'a T {
    type AuthorId = T::AuthorId;

    #[inline]
    fn author(&self) -> &Self::AuthorId { T::author(self) }
}
impl<'a, T: Clone + Authored> Authored for Cow<'a, T> {
    type AuthorId = T::AuthorId;

    #[inline]
    fn author(&self) -> &Self::AuthorId { T::author(self) }
}
