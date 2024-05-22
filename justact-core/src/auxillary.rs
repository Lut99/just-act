//  AUXILLARY.rs
//    by Lut99
//
//  Created:
//    13 May 2024, 14:16:11
//  Last edited:
//    22 May 2024, 10:49:19
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
/// Note, however, that the namespace of uniqueness is only for things of the same type (e.g.,
/// across messages or across agents).
///
/// # Generics
/// - `'v`: The lifetime of the [`SystemView`](crate::SystemView) where the message's data lives.
pub trait Identifiable<'v> {
    /// The thing used as identifier. For convenience, we require it to [`Eq`] and [`Hash`].
    type Id: ?Sized + Eq + Hash;

    /// Returns the identifier for this thing.
    ///
    /// Note, however, that the namespace of uniqueness is only for things of the same type (e.g., across messages or across agents).
    ///
    /// # Returns
    /// Something of type `Self::Id` that uniquely identifiers this object.
    fn id(&self) -> &'v Self::Id;
}

// Implement over some pointer-like types
impl<'a, 'v, T: Identifiable<'v>> Identifiable<'v> for &'a T {
    type Id = T::Id;

    #[inline]
    fn id(&self) -> &'v Self::Id { T::id(self) }
}
impl<'a, 'v, T: Clone + Identifiable<'v>> Identifiable<'v> for Cow<'a, T> {
    type Id = T::Id;

    #[inline]
    fn id(&self) -> &'v Self::Id { T::id(self) }
}



/// Something is authored by some agent.
///
/// # Generics
/// - `'v`: The lifetime of the [`SystemView`](crate::SystemView) where the message's data lives.
pub trait Authored<'v> {
    /// The thing used as identifier of the agent. For convenience, we require it to [`Eq`] and [`Hash`].
    type AuthorId: ?Sized + Eq + Hash;

    /// Returns the unique identifier of the author of this object.
    ///
    /// # Returns
    /// A `Self::Author::Id` that represents the author of this object.
    fn author(&self) -> &'v Self::AuthorId;
}

// Implement over some pointer-like types
impl<'a, 'v, T: Authored<'v>> Authored<'v> for &'a T {
    type AuthorId = T::AuthorId;

    #[inline]
    fn author(&self) -> &'v Self::AuthorId { T::author(self) }
}
impl<'a, 'v, T: Clone + Authored<'v>> Authored<'v> for Cow<'a, T> {
    type AuthorId = T::AuthorId;

    #[inline]
    fn author(&self) -> &'v Self::AuthorId { T::author(self) }
}
