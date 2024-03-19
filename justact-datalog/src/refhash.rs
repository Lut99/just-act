//  REFHASH.rs
//    by Lut99
//
//  Created:
//    19 Mar 2024, 17:05:37
//  Last edited:
//    19 Mar 2024, 17:43:44
//  Auto updated?
//    Yes
//
//  Description:
//!   Implements a wrapper around some hashmaps that can handle
//!   non-`'static` keys.
//

use std::borrow::Cow;
use std::collections::HashMap;
use std::hash::{BuildHasher as _, Hash, Hasher, RandomState};

use crate::ast::{Atom, AtomArg, Literal};


/***** AUXILLARY *****/
/// A type that is more generic than [`Hash`](std::hash::Hash).
pub trait RefHash {
    /// Hashes Self in the given [`Hasher`].
    ///
    /// # Arguments
    /// - `state`: The [`Hasher`] to update with the hash of ourselves.
    fn hash<H: Hasher>(&self, state: &mut H);
}

// Default impls
impl<'k, T: Clone + RefHash> RefHash for Cow<'k, T> {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) { <T as RefHash>::hash(self, state) }
}
impl<'f, 's> RefHash for Literal<&'f str, &'s str> {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Hash manually :')
        match self {
            Self::Atom(a) => {
                state.write_u8(0);
                <Atom<&'f str, &'s str> as RefHash>::hash(a, state);
            },
            Self::NegAtom(a) => {
                state.write_u8(1);
                <Atom<&'f str, &'s str> as RefHash>::hash(&a.atom, state);
            },
        }
    }
}
impl<'f, 's> RefHash for Atom<&'f str, &'s str> {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Hash manually :')
        self.ident.value.value().hash(state);
        for arg in self.args.iter().map(|a| a.args.values()).flatten() {
            match arg {
                AtomArg::Atom(a) => {
                    state.write_u8(0);
                    a.value.value().hash(state);
                },
                AtomArg::Var(v) => {
                    state.write_u8(1);
                    v.value.value().hash(state);
                },
            }
        }
    }
}





/***** LIBRARY *****/
/// Represents an Interpretation/Knowledge Base of the currently derived facts.
#[derive(Clone, Debug)]
pub struct RefHashMap<'k, K: Clone, V> {
    /// Defines a map of truth values.
    values: HashMap<u64, V>,
    /// Defines a map of meaning.
    defs:   HashMap<u64, Cow<'k, K>>,
    /// The hasher state used for hashing.
    state:  RandomState,
}

impl<'k, K: Clone, V> RefHashMap<'k, K, V> {
    /// Constructor for the RefHashMap that initializes it as empty.
    ///
    /// # Returns
    /// A new RefHashMap that has nothing in it.
    #[inline]
    pub fn new() -> Self { Self { values: HashMap::new(), defs: HashMap::new(), state: RandomState::new() } }

    /// Constructor for the RefHashMap that initializes it with enough space for at least the given number of key/value mappings.
    ///
    /// # Arguments
    /// - `capacity`: The number of values to allocate space for.
    ///
    /// # Returns
    /// A new RefHashMap that has space for at least `capacity` values before it needs to re-allocate.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self { values: HashMap::with_capacity(capacity), defs: HashMap::with_capacity(capacity), state: RandomState::new() }
    }
}
impl<'k, K: Clone + RefHash, V> RefHashMap<'k, K, V> {
    /// Static method that hashes a [`RefHash`] for use in the RefHashMap.
    ///
    /// # Arguments
    /// - `key`: The key to hash.
    ///
    /// # Returns
    /// A `u64` representing the hash.
    #[inline]
    pub fn hash(&self, key: &K) -> u64 {
        let mut hasher = self.state.build_hasher();
        key.hash(&mut hasher);
        hasher.finish()
    }

    /// Inserts a new key into the map by ownership with given value.
    ///
    /// # Arguments
    /// - `key`: The key to insert.
    /// - `value`: The value to insert.
    ///
    /// # Returns
    /// If this `key` already existed, returns the old value of it.
    #[inline]
    pub fn insert(&mut self, literal: K, truth: V) -> Option<V> {
        // Hash the literal
        let hash: u64 = self.hash(&literal);

        // Inject it in the definition table (no change if it already exists, I guess), and then the truth
        self.defs.insert(hash, Cow::Owned(literal));
        self.values.insert(hash, truth)
    }

    /// Inserts a new key into the map by ownership _or_ reference.
    ///
    /// # Arguments
    /// - `key`: The key to insert, wrapped in a [`Cow`].
    /// - `value`: The value to insert.
    ///
    /// # Returns
    /// If this `key` already existed, returns the old value of it.
    #[inline]
    pub fn insert_cow(&mut self, literal: Cow<'k, K>, truth: V) -> Option<V> {
        // Hash the literal
        let hash: u64 = self.hash(&literal);

        // Inject it in the definition table (no change if it already exists, I guess), and then the truth
        self.defs.insert(hash, literal);
        self.values.insert(hash, truth)
    }

    /// Inserts a new key into the map by reference with given value.
    ///
    /// # Arguments
    /// - `key`: The key to insert.
    /// - `value`: The value to insert.
    ///
    /// # Returns
    /// If this `key` already existed, returns the old value of it.
    #[inline]
    pub fn insert_by_ref(&mut self, literal: &'k K, truth: V) -> Option<V> {
        // Hash the literal
        let hash: u64 = self.hash(literal);

        // Inject it in the definition table (no change if it already exists, I guess), and then the truth
        self.defs.insert(hash, Cow::Borrowed(literal));
        self.values.insert(hash, truth)
    }

    /// Checks if a mapping with this key already exists.
    ///
    /// # Arguments
    /// - `key`: The key to check for.
    ///
    /// # Returns
    /// True if the mapping exists, or false otherwise.
    #[inline]
    pub fn contains_key(&self, key: &K) -> bool {
        // Hash the literal
        let hash: u64 = self.hash(key);
        self.values.contains_key(&hash)
    }

    /// Returns the value behind the given key.
    ///
    /// # Returns
    /// The value behind the given key, or [`None`] if this mapping does not yet exist.
    #[inline]
    pub fn get<'s>(&'s self, key: &'_ K) -> Option<&'s V> {
        // Hash the literal
        let hash: u64 = self.hash(key);

        // Query the truth tablé
        self.values.get(&hash)
    }
}

impl<'k, K: Clone, V: Eq> Eq for RefHashMap<'k, K, V> {}
impl<'k, K: Clone, V: PartialEq> PartialEq for RefHashMap<'k, K, V> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        // Comparing the value maps is enough!
        self.values == other.values
    }

    #[inline]
    fn ne(&self, other: &Self) -> bool {
        // Comparing the value maps is enough!
        self.values != other.values
    }
}

impl<K: Clone + RefHash, V> FromIterator<(K, V)> for RefHashMap<'static, K, V> {
    #[inline]
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        let iter = iter.into_iter();
        let size_hint: (usize, Option<usize>) = iter.size_hint();
        let mut refhash: RefHashMap<'static, K, V> = RefHashMap::with_capacity(size_hint.1.unwrap_or(size_hint.0));
        for (k, v) in iter {
            refhash.insert(k, v);
        }
        refhash
    }
}
impl<'k, K: Clone + RefHash, V> FromIterator<(&'k K, V)> for RefHashMap<'k, K, V> {
    #[inline]
    fn from_iter<T: IntoIterator<Item = (&'k K, V)>>(iter: T) -> Self {
        let iter = iter.into_iter();
        let size_hint: (usize, Option<usize>) = iter.size_hint();
        let mut refhash: RefHashMap<'k, K, V> = RefHashMap::with_capacity(size_hint.1.unwrap_or(size_hint.0));
        for (k, v) in iter {
            refhash.insert_by_ref(k, v);
        }
        refhash
    }
}
impl<'k, K: Clone + RefHash, V> FromIterator<(Cow<'k, K>, V)> for RefHashMap<'k, K, V> {
    #[inline]
    fn from_iter<T: IntoIterator<Item = (Cow<'k, K>, V)>>(iter: T) -> Self {
        let iter = iter.into_iter();
        let size_hint: (usize, Option<usize>) = iter.size_hint();
        let mut refhash: RefHashMap<'k, K, V> = RefHashMap::with_capacity(size_hint.1.unwrap_or(size_hint.0));
        for (k, v) in iter {
            refhash.insert_cow(k, v);
        }
        refhash
    }
}

impl<const LEN: usize, K: Clone + RefHash, V> From<[(K, V); LEN]> for RefHashMap<'static, K, V> {
    #[inline]
    fn from(value: [(K, V); LEN]) -> Self { value.into_iter().collect() }
}
impl<'k, const LEN: usize, K: Clone + RefHash, V> From<[(Cow<'k, K>, V); LEN]> for RefHashMap<'k, K, V> {
    #[inline]
    fn from(value: [(Cow<'k, K>, V); LEN]) -> Self { value.into_iter().collect() }
}
impl<'k, const LEN: usize, K: Clone + RefHash, V> From<[(&'k K, V); LEN]> for RefHashMap<'k, K, V> {
    #[inline]
    fn from(value: [(&'k K, V); LEN]) -> Self { value.into_iter().collect() }
}
