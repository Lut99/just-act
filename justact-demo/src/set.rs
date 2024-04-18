//  SET.rs
//    by Lut99
//
//  Created:
//    18 Apr 2024, 11:37:12
//  Last edited:
//    18 Apr 2024, 16:58:17
//  Auto updated?
//    Yes
//
//  Description:
//!   Defines implementations for a root [`Set`] that acts as a common part
//!   of both MessageSet and ActionSet implementations.
//

use std::borrow::Cow;
use std::collections::HashSet;
use std::hash::Hash;
use std::mem::MaybeUninit;

use justact_core::set::Set as _;
use stackvec::StackVec;


/***** ITERATORS *****/
/// Iterator-by-ref for the [`Set`].
#[derive(Clone, Debug)]
pub enum Iter<'s, T> {
    // We return a single message
    Singleton(Option<&'s T>),
    // We return a host of messages
    Multi(std::collections::hash_set::Iter<'s, T>),
}
impl<'s, T> Iterator for Iter<'s, T> {
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
pub enum IntoIter<T> {
    // We return a single message
    Singleton(Option<T>),
    // We return a host of messages
    Multi(std::collections::hash_set::IntoIter<T>),
}
impl<T> Iterator for IntoIter<T> {
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
/// A common ancetor to both [`MessageSet`]s and [`ActionSet`]s.
#[derive(Clone, Debug)]
pub enum Set<T: Hash> {
    /// In case there's exactly one element, to prevent allocation.
    Singleton(T),
    /// In case there's zero _or_ multiple elements.
    Multi(HashSet<T>),
}
impl<T: Hash> Default for Set<T> {
    #[inline]
    fn default() -> Self { Self::empty() }
}
impl<T: Hash> Set<T> {
    /// Creates an empty Set.
    ///
    /// # Returns
    /// A Set with not elements in it yet.
    #[inline]
    pub fn empty() -> Self { Self::Multi(HashSet::new()) }

    /// Returns the number of elements in this set.
    pub fn len(&self) -> usize {
        match self {
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
        // Get an owned version of self, leaving us temporarily muted
        // SAFETY: We can cast the pointers here because `MaybeUninit` will have the same layout and alignment as `Self`
        let this: &mut MaybeUninit<Self> = unsafe { &mut *((self as *mut Self) as *mut MaybeUninit<Self>) };
        let mut temp: MaybeUninit<Self> = MaybeUninit::uninit();
        std::mem::swap(&mut temp, this);

        // Create a new temp `self` that's potentially changed variant
        // SAFETY: We know it's initialized because we constructed it through a pointer cast
        let mut temp: MaybeUninit<Self> = match unsafe { temp.assume_init() } {
            Self::Singleton(msg) => {
                let iter = other.into_iter();
                let size_hint: (usize, Option<usize>) = iter.size_hint();

                // Create a set with both self element and the other
                let mut elems: HashSet<T> = HashSet::with_capacity(1 + size_hint.1.unwrap_or(size_hint.0));
                elems.insert(msg);
                elems.extend(iter);

                // Return it. Even if the set is only 1 (i.e., the other was empty or duplicate), we still insert as multi to not waste the allocation.
                MaybeUninit::new(Self::Multi(elems))
            },
            Self::Multi(mut msgs) => {
                if msgs.capacity() == 0 {
                    // The list has no allocation yet. As such, we just directly use the one from the `other`.
                    msgs = other.into_iter().collect();
                } else {
                    // Extend it instead. Even if the extension would result in a set of exactly 1, we still use this over Singleton to not waste the allocation.
                    msgs.extend(other);
                }
                MaybeUninit::new(Self::Multi(msgs))
            },
        };

        // Now we swap the temp back and put it in self
        // SAFETY: We can cast the pointers here because `MaybeUninit` has the same layout and alignment as `Self`
        // SAFETY: We can unwrap the MaybeUninit because all codepaths above initialize it
        std::mem::swap(this, &mut temp);
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
            (Self::Singleton(lhs), Self::Singleton(rhs)) => lhs == rhs,
            (Self::Singleton(lhs), Self::Multi(rhs)) => rhs.len() == 1 && lhs == rhs.iter().next().unwrap(),
            (Self::Multi(lhs), Self::Singleton(rhs)) => lhs.len() == 1 && lhs.iter().next().unwrap() == rhs,
            (Self::Multi(lhs), Self::Multi(rhs)) => lhs == rhs,
        }
    }
}

impl<T: Eq + Hash> justact_core::set::Set for Set<T> {
    type Elem = T;
    type Item<'s> = &'s T where Self: 's;
    type Iter<'s> = Iter<'s, T> where Self: 's;

    #[inline]
    fn iter<'s>(&'s self) -> Self::Iter<'s> {
        match self {
            Self::Singleton(elem) => Iter::Singleton(Some(elem)),
            Self::Multi(elems) => Iter::Multi(elems.iter()),
        }
    }

    #[inline]
    fn add(&mut self, new_elem: Self::Elem) -> bool {
        // Get an owned version of self, leaving us temporarily muted
        // SAFETY: We can cast the pointers here because `MaybeUninit` will have the same layout and alignment as `Self`
        let this: &mut MaybeUninit<Self> = unsafe { &mut *((self as *mut Self) as *mut MaybeUninit<Self>) };
        let mut temp: MaybeUninit<Self> = MaybeUninit::uninit();
        std::mem::swap(&mut temp, this);

        // Incorporate the element, potentially mutating self
        // SAFETY: We know it's initialized because we constructed it through a pointer cast
        let existed;
        let mut temp: MaybeUninit<Self> = match unsafe { temp.assume_init() } {
            Self::Singleton(elem) => {
                existed = elem == new_elem;
                MaybeUninit::new(Self::Multi(HashSet::from([elem, new_elem])))
            },
            Self::Multi(mut elems) => {
                // If the `elems` are empty, we might avoid an allocation by creating a singleton instead
                if elems.capacity() == 0 {
                    // Store as singleton instead
                    existed = false;
                    MaybeUninit::new(Self::Singleton(new_elem))
                } else {
                    // There is already an alloc, use it
                    existed = elems.insert(new_elem);
                    MaybeUninit::new(Self::Multi(elems))
                }
            },
        };

        // Swap back, the exit with existed
        // SAFETY: We can cast the pointers here because `MaybeUninit` has the same layout and alignment as `Self`
        // SAFETY: We can unwrap the MaybeUninit because all codepaths above initialize it
        std::mem::swap(this, &mut temp);
        existed
    }
}

impl<T: Hash> IntoIterator for Set<T> {
    type IntoIter = IntoIter<T>;
    type Item = T;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        match self {
            Self::Singleton(msg) => IntoIter::Singleton(Some(msg)),
            Self::Multi(msgs) => IntoIter::Multi(msgs.into_iter()),
        }
    }
}
impl<'s, T: Eq + Hash> IntoIterator for &'s Set<T> {
    type IntoIter = Iter<'s, T>;
    type Item = &'s T;

    #[inline]
    fn into_iter(self) -> Self::IntoIter { self.iter() }
}

impl<T: Hash> From<T> for Set<T> {
    #[inline]
    fn from(value: T) -> Self { Set::Singleton(value) }
}
