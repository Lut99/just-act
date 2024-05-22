//  STATEMENTS.rs
//    by Lut99
//
//  Created:
//    21 May 2024, 16:48:17
//  Last edited:
//    22 May 2024, 11:04:25
//  Auto updated?
//    Yes
//
//  Description:
//!   Implements the globally synchronized set of stated messages.
//

use std::collections::HashMap;
use std::error::Error;
use std::hash::{BuildHasher, Hash, RandomState};

use crate::auxillary::{Authored, Identifiable};


/***** LIBRARY *****/
/// Defines the framework's notion of policy.
///
/// This is usually accompanied by [`Extractable`] in order to communicate that policy can be
/// extracted from message sets.
///
/// # Generics
/// - `'v`: The lifetime of the [`SystemView`](crate::SystemView) where the policy's data lives.
pub trait Policy<'v> {
    /// The type of error emitted when the policy is not valid (**semantically** incorrect).
    type SemanticError: Error;

    /// Checks whether this policy is valid according to its own semantics.
    ///
    /// # Errors
    /// If the policy is not valid, this function errors. The resulting
    /// [`Self::SemanticError`](Policy::SemanticError) encodes some explanation of why the policy
    /// wasn't valid.
    fn assert_validity(&self) -> Result<(), Self::SemanticError>;
}

/// Extends [`Policy`] with the power to be extracted from [`MessageSet`]s.
///
/// # Generics
/// - `'v`: The lifetime of the [`SystemView`](crate::SystemView) where the set's (and therefore
///   resulting policy's) data lives.
pub trait Extractable<'v> {
    /// The type of error emitted when the policy is **syntactically** incorrect.
    type SyntaxError: Error;

    /// Extracts this policy from a given [`MessageSet`].
    ///
    /// # Arguments
    /// - `set`: The [`MessageSet`] to extract from.
    ///
    /// # Returns
    /// A new instance of Self which represents the parsed policy.
    ///
    /// # Errors
    /// This function should throw a [`Self::SyntaxError`](Extractable::SyntaxError) if and only if
    /// the combined messages' payloads did not make a **syntactically** correct policy.
    ///
    /// Semantic correctness is conventionally modelled by returning a legal policy, but that fails
    /// the [`Policy::check_validity()`]-check.
    fn extract_form<'m, M, R>(set: &MessageSet<'v, M, R>) -> Result<Self, Self::SyntaxError>
    where
        Self: Sized,
        M: Message<'v>;
}



/// Implements a representation of messages in the framework.
///
/// There's a lot of leeway for implementation w.r.t. identifying authors and
/// message identifiers. However, all messages are expected to somehow carry
/// their policies as raw bytes.
///
/// # Generics
/// - `'v`: The lifetime of the [`SystemView`](crate::SystemView) where the message's data lives.
pub trait Message<'v>: Authored<'v> + Identifiable<'v> {
    /// Returns the payload of the message.
    ///
    /// The payload of the message must always be a continious series of raw bytes. What these
    /// bytes mean is up to the relevant agents, who decide which policy language to use.
    ///
    /// # Returns
    /// A byte slice ([`&[u8]`](u8)) that represents the message's full payload.
    fn payload(&self) -> &'v [u8];
}

/// Implements a(n unordered) set of messages.
///
/// The implementation for this set is pre-provided, as we expect this to be the same across
/// implementations.
///
/// Note that the set is always over references to messages, which are stored in the agent's
/// [`SystemView`](crate::SystemView).
///
/// # Generics
/// - `'v`: The lifetime of the [`SystemView`](crate::SystemView) where the message's data lives.
/// - `M`: The type of [`Message`]s stored in this set.
/// - `R`: Some kind of [`BuildHasher`] that is used to compute randomized hashes. This means that
///   hashes are **not** comparable between set instances, only within.
pub struct MessageSet<'v, M, R = RandomState> {
    /// The elements in this set.
    data:  HashMap<u64, &'v M>,
    /// The random state used to compute hashes.
    state: R,
}
// Constructors
impl<'v, M, R: Default> Default for MessageSet<'v, M, R> {
    #[inline]
    fn default() -> Self { Self::new() }
}
impl<'v, M, R: Default> MessageSet<'v, M, R> {
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
impl<'v, M, R> MessageSet<'v, M, R> {
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
// Read-only map functions
impl<'v, M: Identifiable<'v>, R: BuildHasher> MessageSet<'v, M, R> {
    /// Retrieves a message with the given identifier from the set.
    ///
    /// # Arguments
    /// - `id`: The identifier of the message to retrieve.
    ///
    /// # Returns
    /// The referred message if it was known, or else [`None`].
    #[inline]
    pub fn get(&self, id: &M::Id) -> Option<&M> {
        // Hash the key and use that to access the map
        let hash: u64 = self.state.hash_one(id);
        self.data.get(&hash).cloned()
    }

    /// Checks if a message with the given identifier exists in the set.
    ///
    /// # Arguments
    /// - `id`: The identifier of the message to check for existance.
    ///
    /// # Returns
    /// True if the message existed, or false otherwise.
    #[inline]
    pub fn contains(&self, id: &M::Id) -> bool { self.get(id).is_some() }



    /// Returns the number of messages this set can accept before resizing.
    ///
    /// Note that this is the _total_ amount of messages. So subtract [`Self::len()`](MessageSet::len()) from this number to find how many are left to go.
    ///
    /// # Returns
    /// A [`usize`] describing the total capacity of the inner memory block.
    #[inline]
    pub fn capacity(&self) -> usize { self.data.capacity() }

    /// Returns the number of messages in the set.
    ///
    /// # Returns
    /// A [`usize`] describing how many elements are in this set.
    #[inline]
    pub fn len(&self) -> usize { self.data.len() }

    /// Checks if there are any messages in the set.
    ///
    /// # Returns
    /// True if there are **none**, or false otherwise.
    #[inline]
    pub fn is_empty(&self) -> bool { self.len() == 0 }
}
// Mutable map functions
impl<'v, M: Identifiable<'v>, R: BuildHasher> MessageSet<'v, M, R> {
    /// Adds a new message to the set.
    ///
    /// # Arguments
    /// - `msg`: The [`Message`] (reference) to add to this set.
    ///
    /// # Returns
    /// The old [`Message`] if one with the same identifier already existed, or else [`None`].
    #[inline]
    pub fn add(&mut self, msg: &'v M) -> Option<&'v M> {
        // Hash the identifier, then use that as index
        let hash: u64 = self.state.hash_one(msg.id());
        self.data.insert(hash, msg)
    }

    /// Removes an element from the set.
    ///
    /// # Arguments
    /// - `id`: The identifier of the message to remove.
    ///
    /// # Returns
    /// The removed [`Message`], or else [`None`] if there was nothing to remove.
    #[inline]
    pub fn remove(&mut self, id: &M::Id) -> Option<&'v M> {
        // Hash the identifier, then use that as index
        let hash: u64 = self.state.hash_one(id);
        self.data.remove(&hash)
    }



    /// Re-allocates the underlying memory block in order to fascilitate more messages.
    ///
    /// Note that this re-allocation only happens if the set doesn't already have enough space.
    ///
    /// # Arguments
    /// - `additional`: The number of messages for which there should be space **in addition to the ones already there**.
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        if self.len() + additional > self.capacity() {
            self.data.reserve(additional)
        }
    }
}
// JustAct functions
impl<'v, M: Message<'v>, R> MessageSet<'v, M, R> {
    /// Extracts the policy contained within this set.
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
        P::extract_form(self)
    }
}
// Iterator implementations
impl<'v, M, R> MessageSet<'v, M, R> {
    /// Returns an iterator-by-reference for the message set.
    ///
    /// This returns exactly the same elements as a [`Self::from_iter()`](MessageSet::from_iter())-call, except that it does not consume the set itself.
    ///
    /// # Returns
    /// An iterator that returns `&'v M` message references.
    #[inline]
    pub fn iter(&self) -> <&Self as IntoIterator>::IntoIter { self.into_iter() }
}
impl<'v, M, R> IntoIterator for MessageSet<'v, M, R> {
    type Item = &'v M;
    type IntoIter = std::collections::hash_map::IntoValues<u64, &'v M>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter { self.data.into_values() }
}
impl<'a, 'v, M, R> IntoIterator for &'a MessageSet<'v, M, R> {
    type Item = &'v M;
    type IntoIter = std::iter::Cloned<std::collections::hash_map::Values<'a, u64, &'v M>>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter { self.data.values().cloned() }
}
// From-impls
impl<'v, M: Hash, R: Default + BuildHasher> FromIterator<&'v M> for MessageSet<'v, M, R> {
    #[inline]
    fn from_iter<T: IntoIterator<Item = &'v M>>(iter: T) -> Self {
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
impl<'v, const LEN: usize, M: Hash, R: Default + BuildHasher> From<[&'v M; LEN]> for MessageSet<'v, M, R> {
    #[inline]
    fn from(value: [&'v M; LEN]) -> Self { Self::from_iter(value.into_iter()) }
}
impl<'v, M: Hash, R: Default + BuildHasher> From<&[&'v M]> for MessageSet<'v, M, R> {
    #[inline]
    fn from(value: &[&'v M]) -> Self { Self::from_iter(value.iter().cloned()) }
}
impl<'v, M: Hash, R: Default + BuildHasher> From<Vec<&'v M>> for MessageSet<'v, M, R> {
    #[inline]
    fn from(value: Vec<&'v M>) -> Self { Self::from_iter(value.into_iter()) }
}



/// Defines the set of messages that are stated by agents.
///
/// Note that this set is _local_, meaning that its contents may differ per-agent.
///
/// # Generics
/// - `'v`: The lifetime of the [`SystemView`](crate::SystemView) where the message's data lives.
/// - `M`: The type of [`Message`]s that can be stated.
pub trait Statements<'v, M> {}
