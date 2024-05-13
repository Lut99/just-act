//  SET.rs
//    by Lut99
//
//  Created:
//    16 Apr 2024, 10:14:23
//  Last edited:
//    13 May 2024, 14:26:09
//  Auto updated?
//    Yes
//
//  Description:
//!   Defines an abstract [`Set`] that can hold a (potentially!)
//!   unordered set of messages or actions.
//

use crate::auxillary::Identifiable;


/***** LIBRARY *****/
/// Defines an abstract collection of messages or actions.
///
/// The collection is conceptually unordered. That's not to stop implementations for being practically ordered, but there should be no reliance on that in the general case.
pub trait Set<Elem> {
    /// The type returned by [`Set::iter()`]'s iterator.
    type Item<'s>
    where
        Self: 's;
    /// The iterator returned by [`Set::iter()`].
    type Iter<'s>: Iterator<Item = Self::Item<'s>>
    where
        Self: 's;


    /// Inserts a new element into this collection.
    ///
    /// # Arguments
    /// - `elem`: Some element of type `T` to add.
    ///
    /// # Returns
    /// True if this element already existed, or false otherwise.
    fn add(&mut self, new_elem: Elem) -> bool;

    /// Returns an item by its unique identifier.
    ///
    /// # Arguments
    /// - `id`: Something of type `Elem::Id` that identifies the targeted object.
    ///
    /// # Returns
    /// A reference to the internal `Elem` that was identified by `id`, or [`None`] if no such item could be found.
    fn get(&self, id: Elem::Id) -> Option<&Elem>
    where
        Elem: Identifiable;

    /// Checks if an item with a given unique identifier is in this set.
    ///
    /// # Arguments
    /// - `id`: Something of type `Elem::Id` that identifies the targeted object.
    ///
    /// # Returns
    /// True if such an item existed, or false otherwise.
    #[inline]
    fn contains(&self, id: Elem::Id) -> bool
    where
        Elem: Identifiable,
    {
        self.get(id).is_some()
    }


    /// Returns some iterator over references to the internal element.
    ///
    /// # Returns
    /// Something of type `Self::Iter` that returns `&T`.
    fn iter<'s>(&'s self) -> Self::Iter<'s>;


    /// Returns whether there are any elements in this set.
    ///
    /// # Returns
    /// True if there aren't, or false if there are.
    #[inline]
    fn is_empty(&self) -> bool { self.len() == 0 }

    /// Returns the count of elements in this set.
    ///
    /// # Returns
    /// A [`usize`] denoting how many elements are in this set.
    fn len(&self) -> usize;
}
