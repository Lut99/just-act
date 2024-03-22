//  INTERPRETATION.rs
//    by Lut99
//
//  Created:
//    21 Mar 2024, 10:22:40
//  Last edited:
//    22 Mar 2024, 17:16:21
//  Auto updated?
//    Yes
//
//  Description:
//!   Defines how an interpretation $I$ looks like as given in [1].
//

use std::collections::HashMap;

use crate::ast::{Atom, Literal};


/***** LIBRARY *****/
/// Defines a set of values in the logical sense.
#[derive(Clone)]
pub struct Interpretation {
    /// The elements in this set.
    data: HashMap<Atom, bool>,
}
impl Interpretation {
    /// Constructor for the LogicalSet that initializes it as empty.
    ///
    /// # Returns
    /// An empty Interpretation.
    #[inline]
    pub fn new() -> Self { Self { data: HashMap::new() } }

    /// Constructor for the Interpretation that initializes it with the given capacity.
    ///
    /// # Arguments
    /// - `capacity`: The (minimum) number of elements for which this set has space before needing to re-allocate.
    ///
    /// # Returns
    /// An empty Interpretation with enough space for at least `capacity` elements.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self { Self { data: HashMap::with_capacity(capacity) } }

    /// Learns the truth value of a new atom.
    ///
    /// # Arguments
    /// - `atom`: The [`Atom`] to add to the knowledge base.
    /// - `truth`: The truth value of the Atom to add it under.
    ///
    /// # Returns
    /// The old truth value of the atom if we already knew about it, or else [`None`] if it's completely new.
    #[inline]
    pub fn learn(&mut self, atom: Atom, truth: bool) -> Option<bool> { self.data.insert(atom, truth) }

    /// Gets the truth value of the given atom.
    ///
    /// # Arguments
    /// - `atom`: The [`Atom`] to get the truth value of.
    ///
    /// # Returns
    /// True if it was in the interpretation, false if we _know_ of its negative variant, or [`None`] if we don't have any evidence.
    #[inline]
    pub fn truth_of_atom(&self, atom: &Atom) -> Option<bool> { self.data.get(&atom).map(|t| *t) }

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

    /// Removes all knowledge in the interpretation. Sad :'(.
    ///
    /// Note that this does not change the capacity of the interpretation.
    #[inline]
    pub fn clear(&mut self) { self.data.clear() }
}
impl<T: IntoIterator<Item = (Atom, bool)>> From<T> for Interpretation {
    #[inline]
    fn from(value: T) -> Self { Self::from_iter(value.into_iter()) }
}
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
