//  INTERPRETATION.rs
//    by Lut99
//
//  Created:
//    21 Mar 2024, 10:22:40
//  Last edited:
//    28 Mar 2024, 12:07:48
//  Auto updated?
//    Yes
//
//  Description:
//!   Defines how an interpretation $I$ looks like as given in [1].
//

use std::collections::HashMap;
use std::fmt::{Display, Formatter, Result as FResult};
use std::hash::{BuildHasher, DefaultHasher, Hash as _, Hasher, RandomState};

use crate::ast::{Atom, AtomArg, Ident, Literal};
use crate::log::warn;


/***** LIBRARY *****/
/// Defines a set of values in the logical sense.
#[derive(Clone)]
pub struct Interpretation<R = RandomState> {
    /// The elements in this set.
    data:  HashMap<u64, bool>,
    /// Explicit mapping of atoms necessary for iteration.
    defs:  HashMap<u64, Atom>,
    /// The random state used to compute hashes.
    state: R,
}
impl<R: Default> Interpretation<R> {
    /// Constructor for the LogicalSet that initializes it as empty.
    ///
    /// # Returns
    /// An empty Interpretation.
    #[inline]
    pub fn new() -> Self { Self { data: HashMap::new(), defs: HashMap::new(), state: R::default() } }

    /// Constructor for the Interpretation that initializes it with the given capacity.
    ///
    /// # Arguments
    /// - `capacity`: The (minimum) number of elements for which this set has space before needing to re-allocate.
    ///
    /// # Returns
    /// An empty Interpretation with enough space for at least `capacity` elements.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self { data: HashMap::with_capacity(capacity), defs: HashMap::with_capacity(capacity), state: R::default() }
    }
}
impl<R> Interpretation<R> {
    /// Constructor for the Interpretation that initializes it with the given hash state.
    ///
    /// # Arguments
    /// - `state`: The random state used for hashing to initialize the Interpretation with.
    ///
    /// # Returns
    /// An empty Interpretation with the given state for hashes.
    #[inline]
    pub fn with_state(state: R) -> Self { Self { data: HashMap::new(), defs: HashMap::new(), state } }

    /// Gets the truth value of a given atom by hash.
    ///
    /// This essentially skips the hashy part of the hashmap, instead doing it manually. This is useful in case, say, the assignment is external to the atom.
    ///
    /// See [`Self::hash_atom_with_assign()`](Interpretation::hash_atom_with_assign()) to find a correct way of hashing the atom. Doing it manually does not work due to the internal hashmap's random state.
    ///
    /// # Arguments
    /// - `hash`: The hash of the [`Atom`] to get the truth value of.
    ///
    /// # Returns
    /// True if it was in the interpretation, false if we _know_ of its negative variant, or [`None`] if we don't have any evidence.
    #[inline]
    pub fn truth_of_atom_by_hash(&self, hash: u64) -> Option<bool> { self.data.get(&hash).map(|t| *t) }

    /// Removes all knowledge in the interpretation. Sad :'(.
    ///
    /// Note that this does not change the capacity of the interpretation.
    #[inline]
    pub fn clear(&mut self) {
        self.defs.clear();
        self.data.clear();
    }

    /// Returns whether we have ANY knowledge.
    ///
    /// # Returns
    /// True if we know the truth of EXACTLY NONE atom, or false otherwise.
    #[inline]
    pub fn is_empty(&self) -> bool { self.data.is_empty() }

    /// Returns the number of facts of which we know the truth value.
    ///
    /// # Returns
    /// The number of atom of which we explicitly know they are true or false.
    #[inline]
    pub fn len(&self) -> usize { self.data.len() }

    /// Computes some hash of this interpretation.
    ///
    /// This is computed using some general [`Hasher`] that commutes over specific instances. Thus, can be used to check if two instances are the same.
    ///
    /// # Returns
    /// A [`u64`] representing a hash of this interpretation.
    #[inline]
    pub fn hash(&self) -> u64 {
        let mut state: DefaultHasher = DefaultHasher::new();

        // Get a predictable order on the keys
        let mut keys: Vec<u64> = self.data.keys().cloned().collect();
        keys.sort();

        // Hash 'em
        for key in keys {
            state.write_u64(key);
            self.data.get(&key).unwrap().hash(&mut state);
        }

        // Done
        state.finish()
    }
}
impl<R: BuildHasher> Interpretation<R> {
    /// Computes the hash of the given atom.
    ///
    /// This is used internally but exposed for completeness.
    ///
    /// # Arguments
    /// - `atom`: The [`Atom`] to compute the hash of.
    ///
    /// # Returns
    /// Some [`u64`] encoding the `atom`'s hash.
    #[inline]
    pub fn hash_atom(&self, atom: &Atom) -> u64 {
        let mut state: R::Hasher = self.state.build_hasher();

        // Hash the identifier, then all arguments
        atom.ident.hash(&mut state);
        for arg in atom.args.iter().flat_map(|a| a.args.values()) {
            // Warn if it's a var
            if matches!(arg, AtomArg::Var(_)) {
                warn!("Hashing an `AtomArg::Var` (this is probably unintended)");
            }

            // Hash the AtomArg
            arg.hash(&mut state);
        }

        // Done
        state.finish()
    }

    /// Computes the hash of the given atom that has an external assignment.
    ///
    /// This is exposed to be useful for [`Self::truth_of_lit_by_hash()`](Interpretation::truth_of_lit_by_hash()).
    ///
    /// # Arguments
    /// - `ident`: The [`Ident`] of the literal to compute the hash of.
    /// - `assign`: Some iterator yielding assigned values to hash. Each should represent one of the atom's arguments. The first [`None`] counts as end of the list (or the physical end of it if full).
    ///
    /// # Returns
    /// Some [`u64`] encoding the atom's hash.
    #[inline]
    pub fn hash_atom_with_assign(&self, ident: &Ident, assign: impl Iterator<Item = Ident>) -> u64 {
        let mut state: R::Hasher = self.state.build_hasher();

        // Hash the identifier, then all arguments
        ident.hash(&mut state);
        for ass in assign {
            // Hash it as an [`AtomArg`]
            AtomArg::Atom(ass).hash(&mut state);
        }

        // Done
        state.finish()
    }

    /// Learns the truth value of a new atom.
    ///
    /// # Arguments
    /// - `atom`: The [`Atom`] to add to the knowledge base.
    /// - `truth`: The truth value of the Atom to add it under.
    ///
    /// # Returns
    /// The old truth value of the atom if we already knew about it, or else [`None`] if it's completely new.
    #[inline]
    pub fn learn(&mut self, atom: Atom, truth: bool) -> Option<bool> {
        let hash: u64 = self.hash_atom(&atom);

        // Insert the atom now in both maps
        self.defs.insert(hash, atom);
        self.data.insert(hash, truth)
    }

    /// Extends this Interpretation with multiple new atoms.
    ///
    /// # Arguments
    /// - `iter`: Some [iterable](IntoIterator) type that yields the things to extend ourselves with.
    #[inline]
    pub fn extend(&mut self, iter: impl IntoIterator<Item = (Atom, bool)>) {
        let iter = iter.into_iter();

        // Attempt to extend our buffers
        let size_hint: (usize, Option<usize>) = iter.size_hint();
        let additional: usize = size_hint.1.unwrap_or(size_hint.0);
        self.defs.reserve(additional);
        self.data.reserve(additional);

        // Now add all the items
        for (atom, truth) in iter {
            self.learn(atom, truth);
        }
    }

    /// Turns in this Interpretation into an iterator over its atoms.
    ///
    /// # Returns
    /// Some [`Iterator`] that does the work.
    #[inline]
    pub fn into_iter(self) -> impl Iterator<Item = (Atom, bool)> {
        self.defs.into_iter().map(move |(hash, def)| (def, *self.data.get(&hash).unwrap()))
    }

    /// Gets the truth value of the given atom.
    ///
    /// # Arguments
    /// - `atom`: The [`Atom`] to get the truth value of.
    ///
    /// # Returns
    /// True if it was in the interpretation, false if we _know_ of its negative variant, or [`None`] if we don't have any evidence.
    #[inline]
    pub fn truth_of_atom(&self, atom: &Atom) -> Option<bool> {
        let hash: u64 = self.hash_atom(&atom);
        self.truth_of_atom_by_hash(hash)
    }

    /// Gets the truth value of the given literal.
    ///
    /// This is like getting the truth value of the given atom ([`Self::truth_of_atom()`](Interpretation::truth_of_atom())), except that this takes
    /// the negation of the literal into account.
    ///
    /// # Arguments
    /// - `lit`: The [`Literal`] to get the truth value of.
    ///
    /// # Returns
    /// True if `lit` was in the interpretation under its matching polarity (i.e., true if it was a [`Literal::Atom`] and true in the interpretation
    /// _or_ if it was [`Literal::NegAtom`] and false in the interpretation); false if `lit` was in the interpretation under its complementary
    /// polarity; or [`None`] if it didn't occur at all.
    #[inline]
    pub fn truth_of_lit(&self, lit: &Literal) -> Option<bool> {
        match lit {
            Literal::Atom(a) => self.truth_of_atom(a),
            Literal::NegAtom(na) => self.truth_of_atom(&na.atom).map(|t| !t),
        }
    }
}

// Compare
impl<R> Eq for Interpretation<R> {}
impl<R> PartialEq for Interpretation<R> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        if self.data.len() != other.data.len() {
            return false;
        }
        for (hash, lhs) in &self.data {
            if let Some(rhs) = other.data.get(&hash) {
                if *lhs != *rhs {
                    return false;
                }
            } else {
                return false;
            }
        }
        true
    }
}

// Format
impl<R> Display for Interpretation<R> {
    fn fmt(&self, f: &mut Formatter) -> FResult {
        // Separate the atoms out
        let mut pos: Vec<&Atom> = self.defs.iter().filter(|(hash, _)| *self.data.get(hash).unwrap()).map(|(_, def)| def).collect();
        let mut neg: Vec<&Atom> = self.defs.iter().filter(|(hash, _)| !*self.data.get(hash).unwrap()).map(|(_, def)| def).collect();
        pos.sort_by_key(|a| a.ident.value.value());
        neg.sort_by_key(|a| a.ident.value.value());

        // Print 'em
        writeln!(f, "Interpretation {{")?;
        writeln!(f, "    Positive: [")?;
        writeln!(f, "{}", pos.into_iter().map(|a| format!("        {a}\n")).collect::<String>())?;
        writeln!(f, "    ],")?;
        writeln!(f, "    Negative: [")?;
        writeln!(f, "{}", neg.into_iter().map(|a| format!("        {a}\n")).collect::<String>())?;
        writeln!(f, "    ],")?;
        writeln!(f, "}}")
    }
}

// From
impl FromIterator<(Atom, bool)> for Interpretation {
    #[inline]
    fn from_iter<T: IntoIterator<Item = (Atom, bool)>>(iter: T) -> Self {
        let iter = iter.into_iter();
        let size_hint: (usize, Option<usize>) = iter.size_hint();
        let mut int: Self = Self::with_capacity(size_hint.1.unwrap_or(size_hint.0));
        for (atom, truth) in iter {
            int.learn(atom, truth);
        }
        int
    }
}
impl<T: IntoIterator<Item = (Atom, bool)>> From<T> for Interpretation {
    #[inline]
    fn from(value: T) -> Self { Self::from_iter(value.into_iter()) }
}
