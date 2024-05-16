//  SET.rs
//    by Lut99
//
//  Created:
//    16 Apr 2024, 10:14:23
//  Last edited:
//    16 May 2024, 16:24:27
//  Auto updated?
//    Yes
//
//  Description:
//!   Defines an abstract [`Set`] that can hold a (potentially!)
//!   unordered set of messages or actions.
//

use std::borrow::Cow;

use crate::auxillary::Identifiable;


/***** LIBRARY *****/
/// Defines an abstract collection of elements.
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

// Default impls for pointer-like types
impl<'a, Elem, T: Clone + Set<Elem>> Set<Elem> for &'a T {
    type Item<'s> = T::Item<'s> where Self: 's;
    type Iter<'s> = T::Iter<'s> where Self: 's;

    /// This function is not implemented, as it is unreachable on non-mutable pointers.
    #[inline]
    fn add(&mut self, _new_elem: Elem) -> bool { unimplemented!() }

    #[inline]
    fn iter<'s>(&'s self) -> Self::Iter<'s> { T::iter(self) }

    #[inline]
    fn len(&self) -> usize { T::len(self) }
}
impl<'a, Elem, T: Clone + Set<Elem>> Set<Elem> for &'a mut T {
    type Item<'s> = T::Item<'s> where Self: 's;
    type Iter<'s> = T::Iter<'s> where Self: 's;

    /// This function is not implemented, as it is unreachable on non-mutable pointers.
    #[inline]
    fn add(&mut self, new_elem: Elem) -> bool { T::add(self, new_elem) }

    #[inline]
    fn iter<'s>(&'s self) -> Self::Iter<'s> { T::iter(self) }

    #[inline]
    fn len(&self) -> usize { T::len(self) }
}
impl<'a, Elem, T: Clone + Set<Elem>> Set<Elem> for Cow<'a, T> {
    type Item<'s> = T::Item<'s> where Self: 's;
    type Iter<'s> = T::Iter<'s> where Self: 's;

    #[inline]
    fn add(&mut self, new_elem: Elem) -> bool { T::add(self.to_mut(), new_elem) }

    #[inline]
    fn iter<'s>(&'s self) -> Self::Iter<'s> { T::iter(self) }

    #[inline]
    fn len(&self) -> usize { T::len(self) }
}



/// Defines a counterpart to a [`Set`] that is indexable by the element's identifiers.
pub trait Map<Elem>: Set<Elem>
where
    Elem: Identifiable,
{
    /// Returns an item by its unique identifier.
    ///
    /// # Arguments
    /// - `id`: Something of type `Elem::Id` that identifies the targeted object.
    ///
    /// # Returns
    /// A reference to the internal `Elem` that was identified by `id`, or [`None`] if no such item could be found.
    fn get(&self, id: Elem::Id) -> Option<&Elem>;

    /// Checks if an item with a given unique identifier is in this set.
    ///
    /// # Arguments
    /// - `id`: Something of type `Elem::Id` that identifies the targeted object.
    ///
    /// # Returns
    /// True if such an item existed, or false otherwise.
    #[inline]
    fn contains(&self, id: Elem::Id) -> bool { self.get(id).is_some() }
}

// Default impls for pointer-like types
impl<'a, Elem: Identifiable, T: Clone + Map<Elem>> Map<Elem> for &'a T {
    #[inline]
    fn get(&self, id: <Elem>::Id) -> Option<&Elem> { T::get(self, id) }
}
impl<'a, Elem: Identifiable, T: Clone + Map<Elem>> Map<Elem> for &'a mut T {
    #[inline]
    fn get(&self, id: <Elem>::Id) -> Option<&Elem> { T::get(self, id) }
}
impl<'a, Elem: Identifiable, T: Clone + Map<Elem>> Map<Elem> for Cow<'a, T> {
    #[inline]
    fn get(&self, id: <Elem>::Id) -> Option<&Elem> { T::get(self, id) }
}
