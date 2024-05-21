//  STATEMENTS.rs
//    by Lut99
//
//  Created:
//    21 May 2024, 16:48:17
//  Last edited:
//    21 May 2024, 17:13:56
//  Auto updated?
//    Yes
//
//  Description:
//!   Implements the globally synchronized set of stated messages.
//

use std::collections::HashMap;
use std::hash::{BuildHasher, Hash, RandomState};

use crate::auxillary::{Authored, Identifiable};


/***** LIBRARY *****/
/// Implements a representation of messages in the framework.
///
/// There's a lot of leeway for implementation w.r.t. identifying authors and
/// message identifiers. However, all messages are expected to somehow carry
/// their policies as raw bytes.
pub trait Message: Authored + Identifiable {
    /// Returns the payload of the message.
    ///
    /// The payload of the message must always be a continious series of raw bytes. What these
    /// bytes mean is up to the relevant agents, who decide which policy language to use.
    ///
    /// # Returns
    /// A byte slice ([`&[u8]`](u8)) that represents the message's full payload.
    fn payload(&self) -> &[u8];
}



/// Implements a(n unordered) set of messages.
///
/// The implementation for this set is pre-provided, as we expect this to be the same across
/// implementations.
///
/// Note that the set is always over references to messages, which are stored in the agent's
/// [`SystemView`](crate::SystemView).
pub struct MessageSet<'m, M, R = RandomState> {
    /// The elements in this set.
    data:  HashMap<u64, &'m M>,
    /// The random state used to compute hashes.
    state: R,
}
// Constructors
impl<'m, M, R: Default> Default for MessageSet<'m, M, R> {
    #[inline]
    fn default() -> Self { Self::new() }
}
impl<'m, M, R: Default> MessageSet<'m, M, R> {
    /// Constructor for the MessageSet.
    ///
    /// # Returns
    /// An empty set.
    #[inline]
    pub fn new() -> Self { Self { data: HashMap::new(), state: R::default() } }

    /// Constructor for the MessageSet that gives it an initial capacity.
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
impl<'m, M, R> MessageSet<'m, M, R> {
    /// Constructor for the MessageSet that uses a custom random state.
    ///
    /// # Arguments
    /// - `state`: The custom random state to use to compute hashes with.
    ///
    /// # Returns
    /// An empty set that will compute hashes using the given state.
    #[inline]
    pub fn with_random_state(state: R) -> Self { Self { data: HashMap::new(), state } }
}
// From-impls
impl<'m, M: Hash, R: Default + BuildHasher> FromIterator<&'m M> for MessageSet<'m, M, R> {
    #[inline]
    fn from_iter<T: IntoIterator<Item = &'m M>>(iter: T) -> Self {
        // See if we can get a size hint
        let iter: T::IntoIter = iter.into_iter();
        let size_hint: (usize, Option<usize>) = iter.size_hint();
        let size_hint: usize = size_hint.1.unwrap_or(size_hint.0);

        // Populate a set with at least this many elements
        let mut set: Self = Self { data: HashMap::with_capacity(size_hint), state: R::default() };
        for msg in iter {
            // Compute the hash of the message
            let hash: u64 = set.state.hash_one(msg);
            set.data.insert(hash, msg);
        }

        // OK, that's it
        set
    }
}
impl<'m, M: Hash, R: Default + BuildHasher, I: IntoIterator<Item = &'m M>> From<I> for MessageSet<'m, M, R> {
    #[inline]
    fn from(value: I) -> Self { Self::from_iter(value) }
}
