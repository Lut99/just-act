//  AFPS.rs
//    by Lut99
//
//  Created:
//    21 Mar 2024, 11:19:14
//  Last edited:
//    21 Mar 2024, 11:34:38
//  Auto updated?
//    Yes
//
//  Description:
//!   Implements the Alternating Fixed-Point Semantics for $Datalog^\neg$
//!   programs.
//

use super::herbrand::HerbrandInstantiationIterator;
use super::interpretation::Interpretation;
use crate::ast::{Literal, Spec};


/***** TRANSFORMATIONS *****/
/// Implements a simple positive-only derivation procedure for a given [`Spec`] and knowledge base.
///
/// In addition, a particular set of negative literals can be given that are assumed to be true.
///
/// # Arguments
/// - `rules`: A [`HerbrandInstantiationIterator`] that iterates of all the concrete rules in a [`Spec`].
/// - `int`: Some interpretation that contains facts we assume to be true.
pub fn consequence_trans(mut rules: HerbrandInstantiationIterator, int: &Interpretation) {}





/***** LIBRARY FUNCTIONS *****/
/// Given a $Datalog^\neg$ specification, computes the well-founded model given to us by the alternating fixed-point semantics.
///
/// # Arguments
/// - `spec`: The $Datalog^\neg$-program to consider.
///
/// # Returns
/// Three new [`LogicalSet`] that represents the knowledge base of the program.
pub fn evaluate(spec: &Spec) -> Interpretation { todo!() }
