//  COLLECTION.rs
//    by Lut99
//
//  Created:
//    16 Apr 2024, 10:14:23
//  Last edited:
//    17 Apr 2024, 11:30:17
//  Auto updated?
//    Yes
//
//  Description:
//!   Defines an abstract [`Collection`] that can hold a (potentially!)
//!   unordered set of messages or actions.
//

use std::cell::{Ref, RefMut};
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::rc::Rc;
use std::sync::Arc;

use crate::message::Message;


/***** LIBRARY *****/
/// Defines an abstract collection of messages.
///
/// The collection is conceptually unordered. Depending on implementations, though, it may be practically ordered, but this should be ignored for correct implementations.
///
/// This trait only implements the immutable interface of the collection. See [`CollectionMut`] for the mutable part.
pub trait Collection<T> {
    type Iter<'s>: 's + Iterator<Item = &'s T>
    where
        Self: 's,
        T: 's;

    /// Returns some iterator over references to the internal element.
    ///
    /// # Returns
    /// Something of type `Self::Iter` that returns `&T`.
    fn iter<'s>(&'s self) -> Self::Iter<'s>;
}

// Defaul impl for [`HashSet`]s.
impl<T> Collection<T> for HashSet<T> {
    type Iter<'s> = std::collections::hash_set::Iter<'s, T> where T: 's;

    #[inline]
    fn iter<'s>(&'s self) -> Self::Iter<'s> { HashSet::iter(self) }
}
// Defaul impl for [`HashMap`]s.
impl<'a, K, T> Collection<T> for HashMap<K, T>
where
// K: Eq + Hash,
// T: 'a + Message,
// T::Id<'a>: Into<K>,
{
    type Iter<'s> = std::collections::hash_map::Values<'s, K, T> where K: 's, T: 's;

    #[inline]
    fn iter<'s>(&'s self) -> Self::Iter<'s> { HashMap::values(self) }
}

// Default impls for pointer-like types
impl<'a, T, C: Collection<T>> Collection<T> for &'a C {
    type Iter<'s> = C::Iter<'s> where Self: 's, T: 's;

    #[inline]
    fn iter<'s>(&'s self) -> Self::Iter<'s> { C::iter(self) }
}
impl<'a, T, C: Collection<T>> Collection<T> for &'a mut C {
    type Iter<'s> = C::Iter<'s> where Self: 's, T: 's;

    #[inline]
    fn iter<'s>(&'s self) -> Self::Iter<'s> { C::iter(self) }
}
impl<T, C: Collection<T>> Collection<T> for Arc<C> {
    type Iter<'s> = C::Iter<'s> where Self: 's, T: 's;

    #[inline]
    fn iter<'s>(&'s self) -> Self::Iter<'s> { C::iter(self) }
}
impl<T, C: Collection<T>> Collection<T> for Box<C> {
    type Iter<'s> = C::Iter<'s> where Self: 's, T: 's;

    #[inline]
    fn iter<'s>(&'s self) -> Self::Iter<'s> { C::iter(self) }
}
impl<T, C: Collection<T>> Collection<T> for Rc<C> {
    type Iter<'s> = C::Iter<'s> where Self: 's, T: 's;

    #[inline]
    fn iter<'s>(&'s self) -> Self::Iter<'s> { C::iter(self) }
}

// Default impls for cell pointers
impl<'a, T, C: Collection<T>> Collection<T> for Ref<'a, C> {
    type Iter<'s> = C::Iter<'s> where Self: 's, T: 's;

    #[inline]
    fn iter<'s>(&'s self) -> Self::Iter<'s> { C::iter(self) }
}
impl<'a, T, C: Collection<T>> Collection<T> for RefMut<'a, C> {
    type Iter<'s> = C::Iter<'s> where Self: 's, T: 's;

    #[inline]
    fn iter<'s>(&'s self) -> Self::Iter<'s> { C::iter(self) }
}



/// Defines the mutable interface to an abstract [`Collection`] of messages.
pub trait CollectionMut<T>: Collection<T> {
    /// Inserts a new element into this collection.
    ///
    /// # Arguments
    /// - `elem`: Some element of type `T` to add.
    ///
    /// # Returns
    /// True if this element already existed, or false otherwise.
    fn add(&mut self, elem: T) -> bool;
}

// Defaul impl for [`HashSet`]s.
impl<T: Eq + Hash> CollectionMut<T> for HashSet<T> {
    #[inline]
    fn add(&mut self, elem: T) -> bool { self.insert(elem) }
}
// Defaul impl for [`HashMap`]s.
impl<K, T> CollectionMut<T> for HashMap<K, T>
where
    K: Default + Eq + Hash,
    T: Message,
    T::Id: Into<K>,
{
    #[inline]
    fn add(&mut self, elem: T) -> bool { self.insert(elem.id().into(), elem).is_some() }
}

// Default impls for pointer-like types
impl<'a, T, C: CollectionMut<T>> CollectionMut<T> for &'a mut C {
    #[inline]
    fn add(&mut self, elem: T) -> bool { (**self).add(elem) }
}
impl<T, C: CollectionMut<T>> CollectionMut<T> for Box<C> {
    #[inline]
    fn add(&mut self, elem: T) -> bool { (**self).add(elem) }
}

// Default impls for cell pointers
impl<'a, T, C: CollectionMut<T>> CollectionMut<T> for RefMut<'a, C> {
    #[inline]
    fn add(&mut self, elem: T) -> bool { (**self).add(elem) }
}
