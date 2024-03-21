//  INTERPRETATION.rs
//    by Lut99
//
//  Created:
//    21 Mar 2024, 10:22:40
//  Last edited:
//    21 Mar 2024, 11:34:27
//  Auto updated?
//    Yes
//
//  Description:
//!   Defines how an interpretation $I$ looks like as given in [1].
//

use std::collections::HashSet;

use super::herbrand::HerbrandBaseIterator;
use crate::ast::{Literal, NegAtom, Not};


/***** LIBRARY *****/
/// Defines a set of values in the logical sense.
#[derive(Clone)]
pub struct Interpretation {
    /// The elements in this set.
    data: HashSet<Literal>,
}
impl Interpretation {
    /// Constructor for the LogicalSet that initializes it as empty.
    ///
    /// # Returns
    /// An empty Interpretation.
    #[inline]
    pub fn new() -> Self { Self { data: HashSet::new() } }

    /// Constructor for the Interpretation that initializes it with the given capacity.
    ///
    /// # Arguments
    /// - `capacity`: The (minimum) number of elements for which this set has space before needing to re-allocate.
    ///
    /// # Returns
    /// An empty Interpretation with enough space for at least `capacity` elements.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self { Self { data: HashSet::with_capacity(capacity) } }
}
