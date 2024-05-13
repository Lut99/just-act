//  AUXILLARY.rs
//    by Lut99
//
//  Created:
//    13 May 2024, 14:16:11
//  Last edited:
//    13 May 2024, 14:21:34
//  Auto updated?
//    Yes
//
//  Description:
//!   Defines some smaller, not-occuring-in-the-paper traits that are very
//!   convenient.
//

use std::borrow::Cow;


/***** LIBRARY *****/
/// Something is **uniquely** identifiable by something else.
///
/// Note, however, that the namespace of uniqueness is only for things of the same type (e.g., across messages or across agents).
pub trait Identifiable {
    /// The thing used as identifier.
    type Id;

    /// Returns the identifier for this thing.
    ///
    /// Note, however, that the namespace of uniqueness is only for things of the same type (e.g., across messages or across agents).
    ///
    /// # Returns
    /// Something of type `Self::Id` that uniquely identifiers this object.
    fn id(&self) -> Self::Id;
}

// Implement over some pointer-like types
impl<'a, T: Identifiable> Identifiable for &'a T {
    type Id = T::Id;

    #[inline]
    fn id(&self) -> Self::Id { T::id(self) }
}
impl<'a, T: Clone + Identifiable> Identifiable for Cow<'a, T> {
    type Id = T::Id;

    #[inline]
    fn id(&self) -> Self::Id { T::id(self) }
}



/// Something is authored by some agent.
pub trait Authored {
    /// The agent type by which this thing is authored.
    type Author: Identifiable;

    /// Returns the unique identifier of the author of this object.
    ///
    /// # Returns
    /// A `Self::Author::Id` that represents the author of this object.
    fn author(&self) -> <Self::Author as Identifiable>::Id;
}

// Implement over some pointer-like types
impl<'a, T: Authored> Authored for &'a T {
    type Author = T::Author;

    #[inline]
    fn author(&self) -> <Self::Author as Identifiable>::Id { T::author(self) }
}
impl<'a, T: Clone + Authored> Authored for Cow<'a, T> {
    type Author = T::Author;

    #[inline]
    fn author(&self) -> <Self::Author as Identifiable>::Id { T::author(self) }
}
