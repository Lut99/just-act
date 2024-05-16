//  SET.rs
//    by Lut99
//
//  Created:
//    18 Apr 2024, 11:37:12
//  Last edited:
//    16 May 2024, 16:27:44
//  Auto updated?
//    Yes
//
//  Description:
//!   Defines implementations for a root [`Set`] that acts as a common part
//!   of both MessageSet and ActionSet implementations.
//

use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;

use justact_core::auxillary::Identifiable;
use justact_core::set::Set as _;
use stackvec::StackVec;


/***** ITERATORS *****/
/// Iterator-by-ref for the [`Set`].
#[derive(Clone, Debug)]
pub enum SetIter<'s, T> {
    // We return a single message
    Singleton(Option<&'s T>),
    // We return a host of messages
    Multi(std::collections::hash_set::Iter<'s, T>),
}
impl<'s, T> Iterator for SetIter<'s, T> {
    type Item = &'s T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Singleton(msg) => msg.take(),
            Self::Multi(msgs) => msgs.next(),
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Self::Singleton(msg) => {
                if msg.is_some() {
                    (1, Some(1))
                } else {
                    (0, Some(0))
                }
            },
            Self::Multi(msgs) => msgs.size_hint(),
        }
    }
}

/// Iterator-by-ref for the [`Map`].
#[derive(Clone, Debug)]
pub enum MapIter<'s, T: Identifiable> {
    // We return a single message
    Singleton(Option<&'s T>),
    // We return a host of messages
    Multi(std::collections::hash_map::Values<'s, T::Id, T>),
}
impl<'s, T: Identifiable> Iterator for MapIter<'s, T> {
    type Item = &'s T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Singleton(msg) => msg.take(),
            Self::Multi(msgs) => msgs.next(),
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Self::Singleton(msg) => {
                if msg.is_some() {
                    (1, Some(1))
                } else {
                    (0, Some(0))
                }
            },
            Self::Multi(msgs) => msgs.size_hint(),
        }
    }
}



/// Iterator-by-ownership for the [`Set`].
#[derive(Debug)]
pub enum SetIntoIter<T> {
    // We return a single message
    Singleton(Option<T>),
    // We return a host of messages
    Multi(std::collections::hash_set::IntoIter<T>),
}
impl<T> Iterator for SetIntoIter<T> {
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Singleton(msg) => msg.take(),
            Self::Multi(msgs) => msgs.next(),
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Self::Singleton(msg) => {
                if msg.is_some() {
                    (1, Some(1))
                } else {
                    (0, Some(0))
                }
            },
            Self::Multi(msgs) => msgs.size_hint(),
        }
    }
}

/// Iterator-by-ownership for the [`Map`].
#[derive(Debug)]
pub enum MapIntoIter<T: Identifiable> {
    // We return a single message
    Singleton(Option<T>),
    // We return a host of messages
    Multi(std::collections::hash_map::IntoValues<T::Id, T>),
}
impl<T: Identifiable> Iterator for MapIntoIter<T> {
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Singleton(msg) => msg.take(),
            Self::Multi(msgs) => msgs.next(),
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Self::Singleton(msg) => {
                if msg.is_some() {
                    (1, Some(1))
                } else {
                    (0, Some(0))
                }
            },
            Self::Multi(msgs) => msgs.size_hint(),
        }
    }
}





/***** LIBRARY *****/
/// A generic implementation of a set, which implements various other structures.
#[derive(Clone, Debug)]
pub enum Set<T> {
    /// No elements are in the set.
    Empty,
    /// In case there's exactly one element, to prevent allocation.
    Singleton(T),
    /// In case there's zero _or_ multiple elements.
    Multi(HashSet<T>),
}
impl<T> Default for Set<T> {
    #[inline]
    fn default() -> Self { Self::empty() }
}
impl<T> Set<T> {
    /// Creates an empty Set.
    ///
    /// # Returns
    /// A Set with not elements in it yet.
    #[inline]
    pub fn empty() -> Self { Self::Empty }
    /// Creates a Set with space for at least the given number of elements.
    ///
    /// # Arguments
    /// - `capacity`: The minimum number of elements this Set can store without re-allocation. Might be more, depending on what the allocator deems efficient.
    ///
    /// # Returns
    /// A Set with not elements in it yet, but with capacity for at least `capacity` elements.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self { Self::Multi(HashSet::with_capacity(capacity)) }

    /// Returns the number of elements in this set.
    pub fn len(&self) -> usize {
        match self {
            Self::Empty => 0,
            Self::Singleton(_) => 1,
            Self::Multi(msgs) => msgs.len(),
        }
    }
}
impl<T: Eq + Hash> Set<T> {
    /// Merges the given Set into this one.
    ///
    /// # Arguments
    /// - `other`: Some other Set to join.
    #[inline]
    pub fn join(&mut self, other: impl IntoIterator<Item = T>) {
        // Get an owned version of self
        let mut temp: Self = Self::Empty;
        std::mem::swap(&mut temp, self);

        // Create a new temp `self` that's potentially changed variant
        let mut temp: Self = match temp {
            Self::Empty => {
                // Create an allocation for other
                Self::Multi(other.into_iter().collect())
            },
            Self::Singleton(msg) => {
                let iter = other.into_iter();
                let size_hint: (usize, Option<usize>) = iter.size_hint();

                // Create a set with both self element and the other
                let mut elems: HashSet<T> = HashSet::with_capacity(1 + size_hint.1.unwrap_or(size_hint.0));
                elems.insert(msg);
                elems.extend(iter);

                // Return it. Even if the set is only 1 (i.e., the other was empty or duplicate), we still insert as multi to not waste the allocation.
                Self::Multi(elems)
            },
            Self::Multi(mut msgs) => {
                if msgs.capacity() == 0 {
                    // The list has no allocation yet. As such, we just directly use the one from the `other`.
                    msgs = other.into_iter().collect();
                } else {
                    // Extend it instead. Even if the extension would result in a set of exactly 1, we still use this over Singleton to not waste the allocation.
                    msgs.extend(other);
                }
                Self::Multi(msgs)
            },
        };

        // Now we swap the temp back and put it in self
        std::mem::swap(self, &mut temp);
    }
}
impl<'a, T: Clone + Eq + Hash> Set<Cow<'a, T>> {
    /// Clones this `Set`, returning an equivalent where all elements are borrowed from `self` instead of whatever they were.
    ///
    /// # Returns
    /// A new `Set` that has its lifetime scoped down to `self`.
    #[inline]
    pub fn reborrow<'s>(&'s self) -> Set<Cow<'s, T>> {
        match self {
            Self::Empty => Self::Empty,
            Self::Singleton(elem) => Set::Singleton(Cow::Borrowed(elem.as_ref())),
            Self::Multi(elems) => Set::Multi(elems.iter().map(|e| Cow::Borrowed(e.as_ref())).collect()),
        }
    }
}

impl<T: Eq + Hash> Eq for Set<T> {}
impl<T: Hash + Ord> Hash for Set<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // Collect everything in an ordered fashion, then commit to some deterministic order
        let mut buf: StackVec<64, &T> = StackVec::new();
        for elem in self {
            buf.push(elem);
        }
        buf.sort();

        // Hash that
        for elem in buf {
            elem.hash(state);
        }
    }
}
impl<T: Eq + Hash> PartialEq for Set<T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Empty, Self::Empty) => true,
            (Self::Empty, Self::Singleton(_)) => false,
            (Self::Empty, Self::Multi(rhs)) => rhs.is_empty(),
            (Self::Singleton(_), Self::Empty) => false,
            (Self::Singleton(lhs), Self::Singleton(rhs)) => lhs == rhs,
            (Self::Singleton(lhs), Self::Multi(rhs)) => rhs.len() == 1 && lhs == rhs.iter().next().unwrap(),
            (Self::Multi(lhs), Self::Empty) => lhs.is_empty(),
            (Self::Multi(lhs), Self::Singleton(rhs)) => lhs.len() == 1 && lhs.iter().next().unwrap() == rhs,
            (Self::Multi(lhs), Self::Multi(rhs)) => lhs == rhs,
        }
    }
}

impl<T: Eq + Hash> justact_core::Set<T> for Set<T> {
    type Item<'s> = &'s T where Self: 's;
    type Iter<'s> = SetIter<'s, T> where Self: 's;

    #[inline]
    fn add(&mut self, new_elem: T) -> bool {
        // Get an owned version of self
        let mut temp: Self = Self::Empty;
        std::mem::swap(&mut temp, self);

        // Incorporate the element, potentially mutating self
        let existed;
        let mut temp: Self = match temp {
            Self::Empty => {
                existed = false;
                Self::Singleton(new_elem)
            },
            Self::Singleton(elem) => {
                existed = elem == new_elem;
                Self::Multi(HashSet::from([elem, new_elem]))
            },
            Self::Multi(mut elems) => {
                // If the `elems` are empty, we might avoid an allocation by creating a singleton instead
                if elems.capacity() == 0 {
                    // Store as singleton instead
                    existed = false;
                    Self::Singleton(new_elem)
                } else {
                    // There is already an alloc, use it
                    existed = elems.insert(new_elem);
                    Self::Multi(elems)
                }
            },
        };

        // Swap back, the exit with existed
        std::mem::swap(self, &mut temp);
        existed
    }

    #[inline]
    fn iter<'s>(&'s self) -> Self::Iter<'s> {
        match self {
            Self::Empty => SetIter::Singleton(None),
            Self::Singleton(elem) => SetIter::Singleton(Some(elem)),
            Self::Multi(elems) => SetIter::Multi(elems.iter()),
        }
    }

    #[inline]
    fn len(&self) -> usize {
        match self {
            Self::Empty => 0,
            Self::Singleton(_) => 1,
            Self::Multi(msgs) => msgs.len(),
        }
    }
}
impl<T: Eq + Hash + Identifiable> justact_core::Map<T> for Set<T> {
    #[inline]
    fn get(&self, id: T::Id) -> Option<&T> {
        // We have to do iterative search
        match self {
            Self::Empty => None,
            Self::Singleton(msg) => {
                if msg.id() == id {
                    Some(msg)
                } else {
                    None
                }
            },
            Self::Multi(msgs) => {
                for msg in msgs {
                    if msg.id() == id {
                        return Some(msg);
                    }
                }
                None
            },
        }
    }
}

impl<T> IntoIterator for Set<T> {
    type IntoIter = SetIntoIter<T>;
    type Item = T;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        match self {
            Self::Empty => SetIntoIter::Singleton(None),
            Self::Singleton(msg) => SetIntoIter::Singleton(Some(msg)),
            Self::Multi(msgs) => SetIntoIter::Multi(msgs.into_iter()),
        }
    }
}
impl<'s, T: Eq + Hash> IntoIterator for &'s Set<T> {
    type IntoIter = SetIter<'s, T>;
    type Item = &'s T;

    #[inline]
    fn into_iter(self) -> Self::IntoIter { self.iter() }
}

impl<T> From<T> for Set<T> {
    #[inline]
    fn from(value: T) -> Self { Set::Singleton(value) }
}



/// A variation of a [`Set`] which uses the element's ID to make searching the set more efficient.
#[derive(Clone, Debug)]
pub enum Map<T: Identifiable> {
    /// No elements are in the set.
    Empty,
    /// In case there's exactly one element, to prevent allocation.
    Singleton(T),
    /// In case there's zero _or_ multiple elements.
    Multi(HashMap<T::Id, T>),
}
impl<T: Identifiable> Default for Map<T> {
    #[inline]
    fn default() -> Self { Self::empty() }
}
impl<T: Identifiable> Map<T> {
    /// Creates an empty Map.
    ///
    /// # Returns
    /// A Map with not elements in it yet.
    #[inline]
    pub fn empty() -> Self { Self::Empty }
    /// Creates a Map with space for at least the given number of elements.
    ///
    /// # Arguments
    /// - `capacity`: The minimum number of elements this Map can store without re-allocation. Might be more, depending on what the allocator deems efficient.
    ///
    /// # Returns
    /// A Map with not elements in it yet, but with capacity for at least `capacity` elements.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self { Self::Multi(HashMap::with_capacity(capacity)) }

    /// Returns the number of elements in this Map.
    pub fn len(&self) -> usize {
        match self {
            Self::Empty => 0,
            Self::Singleton(_) => 1,
            Self::Multi(msgs) => msgs.len(),
        }
    }
}
impl<T> Map<T>
where
    T: Identifiable,
    T::Id: Eq + Hash,
{
    /// Merges the given Map into this one.
    ///
    /// # Arguments
    /// - `other`: Some other Map to join.
    #[inline]
    pub fn join(&mut self, other: impl IntoIterator<Item = T>) {
        // Get an owned version of self
        let mut temp: Self = Self::Empty;
        std::mem::swap(&mut temp, self);

        // Create a new temp `self` that's potentially changed variant
        let mut temp: Self = match temp {
            Self::Empty => {
                // Create an allocation for other
                Self::Multi(other.into_iter().map(|elem| (elem.id(), elem)).collect())
            },
            Self::Singleton(msg) => {
                let iter = other.into_iter().map(|elem| (elem.id(), elem));
                let size_hint: (usize, Option<usize>) = iter.size_hint();

                // Create a map with both self element and the other
                let mut elems: HashMap<T::Id, T> = HashMap::with_capacity(1 + size_hint.1.unwrap_or(size_hint.0));
                elems.insert(msg.id(), msg);
                elems.extend(iter);

                // Return it. Even if the map is only 1 (i.e., the other was empty or duplicate), we still insert as multi to not waste the allocation.
                Self::Multi(elems)
            },
            Self::Multi(mut msgs) => {
                if msgs.capacity() == 0 {
                    // The list has no allocation yet. As such, we just directly use the one from the `other`.
                    msgs = other.into_iter().map(|elem| (elem.id(), elem)).collect();
                } else {
                    // Extend it instead. Even if the extension would result in a map of exactly 1, we still use this over Singleton to not waste the allocation.
                    msgs.extend(other.into_iter().map(|elem| (elem.id(), elem)));
                }
                Self::Multi(msgs)
            },
        };

        // Now we swap the temp back and put it in self
        std::mem::swap(self, &mut temp);
    }
}
impl<'a, T> Map<Cow<'a, T>>
where
    T: Clone + Identifiable,
    T::Id: Clone + Eq + Hash,
{
    /// Clones this `Set`, returning an equivalent where all elements are borrowed from `self` instead of whatever they were.
    ///
    /// # Returns
    /// A new `Set` that has its lifetime scoped down to `self`.
    #[inline]
    pub fn reborrow<'s>(&'s self) -> Map<Cow<'s, T>> {
        match self {
            Self::Empty => Map::Empty,
            Self::Singleton(elem) => Map::Singleton(Cow::Borrowed(elem.as_ref())),
            Self::Multi(elems) => Map::Multi(elems.iter().map(|(i, e)| (i.clone(), Cow::Borrowed(e.as_ref()))).collect()),
        }
    }
}

impl<T> Eq for Map<T> where T: Eq + Identifiable {}
impl<T> Hash for Map<T>
where
    T: Hash + Identifiable,
    T::Id: Ord,
{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // Collect everything in an ordered fashion, then commit to some deterministic order
        let mut buf: StackVec<64, &T> = StackVec::new();
        for elem in self {
            buf.push(elem);
        }
        buf.sort_by_key(|elem| elem.id());

        // Hash that
        for elem in buf {
            elem.hash(state);
        }
    }
}
impl<T> PartialEq for Map<T>
where
    T: Identifiable + PartialEq,
{
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Empty, Self::Empty) => true,
            (Self::Empty, Self::Singleton(_)) => false,
            (Self::Empty, Self::Multi(rhs)) => rhs.is_empty(),
            (Self::Singleton(_), Self::Empty) => false,
            (Self::Singleton(lhs), Self::Singleton(rhs)) => lhs == rhs,
            (Self::Singleton(lhs), Self::Multi(rhs)) => rhs.len() == 1 && lhs == rhs.values().next().unwrap(),
            (Self::Multi(lhs), Self::Empty) => lhs.is_empty(),
            (Self::Multi(lhs), Self::Singleton(rhs)) => lhs.len() == 1 && lhs.values().next().unwrap() == rhs,
            (Self::Multi(lhs), Self::Multi(rhs)) => lhs == rhs,
        }
    }
}

impl<T> justact_core::Set<T> for Map<T>
where
    T: Identifiable,
    T::Id: Eq + Hash,
{
    type Item<'s> = &'s T where Self: 's;
    type Iter<'s> = MapIter<'s, T> where Self: 's;

    #[inline]
    fn add(&mut self, new_elem: T) -> bool {
        // Get an owned version of self
        let mut temp: Self = Self::Empty;
        std::mem::swap(&mut temp, self);

        // Incorporate the element, potentially mutating self
        let existed;
        let mut temp: Self = match temp {
            Self::Empty => {
                existed = false;
                Self::Singleton(new_elem)
            },
            Self::Singleton(elem) => {
                existed = elem.id() == new_elem.id();
                Self::Multi(HashMap::from([(elem.id(), elem), (new_elem.id(), new_elem)]))
            },
            Self::Multi(mut elems) => {
                // If the `elems` are empty, we might avoid an allocation by creating a singleton instead
                if elems.capacity() == 0 {
                    // Store as singleton instead
                    existed = false;
                    Self::Singleton(new_elem)
                } else {
                    // There is already an alloc, use it
                    existed = elems.insert(new_elem.id(), new_elem).is_some();
                    Self::Multi(elems)
                }
            },
        };

        // Swap back, the exit with existed
        std::mem::swap(self, &mut temp);
        existed
    }

    #[inline]
    fn iter<'s>(&'s self) -> Self::Iter<'s> {
        match self {
            Self::Empty => MapIter::Singleton(None),
            Self::Singleton(elem) => MapIter::Singleton(Some(elem)),
            Self::Multi(elems) => MapIter::Multi(elems.values()),
        }
    }

    #[inline]
    fn len(&self) -> usize {
        match self {
            Self::Empty => 0,
            Self::Singleton(_) => 1,
            Self::Multi(msgs) => msgs.len(),
        }
    }
}
impl<T: Identifiable> justact_core::Map<T> for Map<T> {
    #[inline]
    fn get(&self, id: T::Id) -> Option<&T> {
        // We can do fast search
        match self {
            Self::Empty => None,
            Self::Singleton(msg) => {
                if msg.id() == id {
                    Some(msg)
                } else {
                    None
                }
            },
            Self::Multi(msgs) => msgs.get(&id),
        }
    }
}

impl<T: Identifiable> IntoIterator for Map<T> {
    type IntoIter = MapIntoIter<T>;
    type Item = T;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        match self {
            Self::Empty => MapIntoIter::Singleton(None),
            Self::Singleton(msg) => MapIntoIter::Singleton(Some(msg)),
            Self::Multi(msgs) => MapIntoIter::Multi(msgs.into_values()),
        }
    }
}
impl<'s, T> IntoIterator for &'s Map<T>
where
    T: Identifiable,
    T::Id: Eq + Hash,
{
    type IntoIter = MapIter<'s, T>;
    type Item = &'s T;

    #[inline]
    fn into_iter(self) -> Self::IntoIter { self.iter() }
}

impl<T: Identifiable> From<T> for Map<T> {
    #[inline]
    fn from(value: T) -> Self { Self::Singleton(value) }
}
