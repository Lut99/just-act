//  SET.rs
//    by Lut99
//
//  Created:
//    16 Apr 2024, 10:14:23
//  Last edited:
//    23 May 2024, 11:31:00
//  Auto updated?
//    Yes
//
//  Description:
//!   Defines an abstract [`Set`] that can hold a (potentially!)
//!   unordered set of messages or actions.
//

use std::collections::HashMap;
use std::hash::{BuildHasher, Hash, RandomState};

use crate::auxillary::Identifiable;
use crate::statements::{Extractable, Message};


/***** LIBRARY *****/
/// Implements a(n unordered) set of messages or actions.
///
/// The implementation for the set is pre-provided, as we expect this to be the same across
/// implementations.
///
/// # Generics
/// - `V`: The type of [`Message`]/[`Action`]s stored in this set.
/// - `R`: Some kind of [`BuildHasher`] that is used to compute randomized hashes. This means that
///   hashes are **not** comparable between set instances, only within.
#[derive(Clone, Debug)]
pub struct Set<V, R = RandomState> {
    /// The elements in this set.
    data:  HashMap<u64, V>,
    /// The random state used to compute hashes.
    state: R,
}
// Constructors
impl<V, R: Default> Default for Set<V, R> {
    #[inline]
    fn default() -> Self { Self::new() }
}
impl<V, R: Default> Set<V, R> {
    /// Constructor for the Set.
    ///
    /// # Returns
    /// An empty set.
    #[inline]
    pub fn new() -> Self { Self { data: HashMap::new(), state: R::default() } }

    /// Constructor for the Set that gives it an initial capacity.
    ///
    /// # Arguments
    /// - `capacity`: The _minimum_ number of elements the returned set should be able to accept
    ///   before needing to re-allocate. Due to optimizations, it _may_ have a higher capacity, but
    ///   never lower.
    ///
    /// # Returns
    /// An empty set that can accept at least `capacity` elements before needing to re-allocate.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self { Self { data: HashMap::with_capacity(capacity), state: R::default() } }
}
impl<V, R> Set<V, R> {
    /// Constructor for the Set that uses a custom random state.
    ///
    /// # Arguments
    /// - `state`: The custom random state to use to compute hashes with.
    ///
    /// # Returns
    /// An empty set that will compute hashes using the given state.
    #[inline]
    pub fn with_random_state(state: R) -> Self { Self { data: HashMap::new(), state } }
}
// Read-only map functions
impl<'v, V: Identifiable<'v>, R: BuildHasher> Set<V, R> {
    /// Retrieves an element with the given identifier from the set.
    ///
    /// # Arguments
    /// - `id`: The identifier of the element to retrieve.
    ///
    /// # Returns
    /// The referred element if it was known, or else [`None`].
    #[inline]
    pub fn get(&self, id: &V::Id) -> Option<&V> {
        // Hash the key and use that to access the map
        let hash: u64 = self.state.hash_one(id);
        self.data.get(&hash)
    }

    /// Checks if an element with the given identifier exists in the set.
    ///
    /// # Arguments
    /// - `id`: The identifier of the element to check for existance.
    ///
    /// # Returns
    /// True if the element existed, or false otherwise.
    #[inline]
    pub fn contains(&self, id: &V::Id) -> bool { self.get(id).is_some() }



    /// Returns the number of elements this set can accept before resizing.
    ///
    /// Note that this is the _total_ amount of elements. So subtract [`Self::len()`](Set::len()) from this number to find how many are left to go.
    ///
    /// # Returns
    /// A [`usize`] describing the total capacity of the inner memory block.
    #[inline]
    pub fn capacity(&self) -> usize { self.data.capacity() }

    /// Returns the number of elements in the set.
    ///
    /// # Returns
    /// A [`usize`] describing how many elements are in this set.
    #[inline]
    pub fn len(&self) -> usize { self.data.len() }

    /// Checks if there are any elements in the set.
    ///
    /// # Returns
    /// True if there are **none**, or false otherwise.
    #[inline]
    pub fn is_empty(&self) -> bool { self.len() == 0 }
}
// Mutable map functions
impl<'v, V: 'v + Identifiable<'v>, R: BuildHasher> Set<V, R> {
    /// Adds a new element to the set.
    ///
    /// # Arguments
    /// - `elem`: The element to add to this set.
    ///
    /// # Returns
    /// The old element if one with the same identifier already existed, or else [`None`].
    #[inline]
    pub fn add(&mut self, elem: V) -> Option<V> {
        // Hash the identifier, then use that as index
        let hash: u64 = self.state.hash_one(elem.id());
        self.data.insert(hash, elem)
    }

    /// Removes an element from the set.
    ///
    /// # Arguments
    /// - `id`: The identifier of the element to remove.
    ///
    /// # Returns
    /// The removed element `V`, or else [`None`] if there was nothing to remove.
    #[inline]
    pub fn remove(&mut self, id: &V::Id) -> Option<V> {
        // Hash the identifier, then use that as index
        let hash: u64 = self.state.hash_one(id);
        self.data.remove(&hash)
    }



    /// Re-allocates the underlying memory block in order to fascilitate more elements.
    ///
    /// Note that this re-allocation only happens if the set doesn't already have enough space.
    ///
    /// # Arguments
    /// - `additional`: The number of elements for which there should be space **in addition to the ones already there**.
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        if self.len() + additional > self.capacity() {
            self.data.reserve(additional)
        }
    }
}
// JustAct functions
impl<'v, V: Message<'v>, R> Set<V, R> {
    /// Extracts the policy contained within this set if it's a set over messages.
    ///
    /// # Generics arguments
    /// - `P`: The policy language that should be extracted from this set.
    ///
    /// # Returns
    /// A parsed `P` from the payload of all internal messages.
    ///
    /// # Errors
    /// This function may error if the internal payloads did not form a **syntactically correct** policy.
    ///
    /// Note that **semantic incorrectness** is conventionally not treated as this kind of error,
    /// but instead returned as a valid but failing policy.
    #[inline]
    pub fn extract<P>(&self) -> Result<P, P::SyntaxError>
    where
        P: Extractable<'v>,
    {
        P::extract_from(self)
    }
}
// Iterator implementations
impl<'v, M, R> Set<M, R> {
    /// Returns an iterator-by-reference for the message set.
    ///
    /// This returns exactly the same elements as a [`Self::from_iter()`](Set::from_iter())-call, except that it does not consume the set itself.
    ///
    /// # Returns
    /// An iterator that returns `&'v M` message references.
    #[inline]
    pub fn iter(&self) -> <&Self as IntoIterator>::IntoIter { self.into_iter() }
}
impl<M, R> IntoIterator for Set<M, R> {
    type Item = M;
    type IntoIter = std::collections::hash_map::IntoValues<u64, M>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter { self.data.into_values() }
}
impl<'a, M, R> IntoIterator for &'a Set<M, R> {
    type Item = &'a M;
    type IntoIter = std::collections::hash_map::Values<'a, u64, M>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter { self.data.values() }
}
// From-impls
impl<M: Hash, R: Default + BuildHasher> FromIterator<M> for Set<M, R> {
    #[inline]
    fn from_iter<T: IntoIterator<Item = M>>(iter: T) -> Self {
        // See if we can get a size hint
        let iter: T::IntoIter = iter.into_iter();
        let size_hint: (usize, Option<usize>) = iter.size_hint();
        let size_hint: usize = size_hint.1.unwrap_or(size_hint.0);

        // Populate a set with at least this many elements
        let mut set: Self = Self { data: HashMap::with_capacity(size_hint), state: R::default() };
        for elem in iter {
            // Compute the hash of the message
            let hash: u64 = set.state.hash_one(&elem);
            set.data.insert(hash, elem);
        }

        // OK, that's it
        set
    }
}
impl<const LEN: usize, M: Hash, R: Default + BuildHasher> From<[M; LEN]> for Set<M, R> {
    #[inline]
    fn from(value: [M; LEN]) -> Self { Self::from_iter(value.into_iter()) }
}
impl<M: Hash, R: Default + BuildHasher> From<Vec<M>> for Set<M, R> {
    #[inline]
    fn from(value: Vec<M>) -> Self { Self::from_iter(value.into_iter()) }
}
