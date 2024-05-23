//  WORLD.rs
//    by Lut99
//
//  Created:
//    21 May 2024, 16:23:20
//  Last edited:
//    23 May 2024, 11:11:28
//  Auto updated?
//    Yes
//
//  Description:
//!   Contributes the [`SystemView`].
//

use std::fmt::{Debug, Formatter, Result as FResult};

use crate::statements::Statements;
use crate::times::Times;


/***** LIBRARY *****/
/// Defines a particular view that agents can have on the system: the [`SystemView`].
///
/// This serves two purposes. On one hand, the [`SystemView`] aggregates all inter-agents sets in
/// one convenient struct. At the other, it provides a basis for ownership of the actual data,
/// which the rest of the ontology provides a particular view on.
///
/// Implemented this way, one can think of the [`SystemView`] as the "root" of the "lifetime" tree,
/// which other structs only refer to.
pub struct SystemView<T, A, S, E> {
    // Global sets
    /// Defines the times synchronized between agents.
    ///
    /// This is a _globally synchronized_ set, meaning that the framework requires agents to be in
    /// agreement at all times about this set's contents.
    ///
    /// See [`crate::times`] for more information.
    pub times:  T,
    /// Defines the agreements synchronized between agents.
    ///
    /// This is a _globally synchronized_ set, meaning that the framework requires agents to be in
    /// agreement at all times about this set's contents.
    ///
    /// See [`crate::agreements`] for more information.
    pub agreed: A,

    // Local sets
    /// Defines the messages which are _stated_.
    ///
    /// This is a _local_ set, meaning that the view presented here may be incomplete and in
    /// conflict with other agents.
    ///
    /// See [`crate::statements`] for more information.
    pub stated:  S,
    /// Defines the messages which are _enacted_.
    ///
    /// This is a _local_ set, meaning that the view presented here may be incomplete and in
    /// conflict with other agents.
    ///
    /// See [`crate::actions`] for more information.
    pub enacted: E,
}

// Some std impls
impl<T: Clone, A: Clone, S: Clone, E: Clone> Clone for SystemView<T, A, S, E> {
    #[inline]
    fn clone(&self) -> Self {
        Self { times: self.times.clone(), agreed: self.agreed.clone(), stated: self.stated.clone(), enacted: self.enacted.clone() }
    }
}
impl<T: Debug, A: Debug, S: Debug, E: Debug> Debug for SystemView<T, A, S, E> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        let mut fmt = f.debug_struct("SystemView");
        fmt.field("times", &self.times);
        fmt.field("agreed", &self.agreed);
        fmt.field("stated", &self.stated);
        fmt.field("enacted", &self.enacted);
        fmt.finish()
    }
}

// JustAct impls
impl<T: Times, A, S, E> Times for SystemView<T, A, S, E> {
    type Time = T::Time;
    type Error = T::Error;

    #[inline]
    fn current(&self) -> &Self::Time { self.times.current() }

    #[inline]
    fn advance_to(&mut self, timestamp: Self::Time) -> Result<(), Self::Error> { self.times.advance_to(timestamp) }
}
impl<T, A, S: Statements, E> Statements for SystemView<T, A, S, E> {
    type Message<'s> = S::Message<'s> where Self: 's;
    type Target = S::Target;
    type Status = S::Status;
    type State = S::State;

    #[inline]
    fn state<'s>(&'s mut self, target: Self::Target, msg: impl Into<Self::Message<'s>>) -> Self::Status { self.stated.state(target, msg) }

    #[inline]
    fn stated<'s>(&'s self) -> crate::statements::MessageSet<Self::Message<'s>, Self::State> { self.stated.stated() }
}
