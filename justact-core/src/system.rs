//  WORLD.rs
//    by Lut99
//
//  Created:
//    21 May 2024, 16:23:20
//  Last edited:
//    23 May 2024, 13:36:26
//  Auto updated?
//    Yes
//
//  Description:
//!   Contributes the [`SystemView`].
//

use std::fmt::{Debug, Formatter, Result as FResult};

use crate::agreements::{Agreement, Agreements};
use crate::set::Set;
use crate::statements::{Action, Statements};
use crate::times::{Times, Timestamp};


/***** LIBRARY *****/
/// Defines a particular view that agents can have on the system: the [`SystemView`].
///
/// This serves two purposes. On one hand, the [`SystemView`] aggregates all inter-agents sets in
/// one convenient struct. At the other, it provides a basis for ownership of the actual data,
/// which the rest of the ontology provides a particular view on.
///
/// Implemented this way, one can think of the [`SystemView`] as the "root" of the "lifetime" tree,
/// which other structs only refer to.
pub struct SystemView<T, A, S> {
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
    /// Defines the messages which are _stated_, and which of those are _enacted_.
    ///
    /// This is a _local_ set, meaning that the view presented here may be incomplete and in
    /// conflict with other agents.
    ///
    /// See [`crate::statements`] for more information.
    pub stated: S,
}

// Some std impls
impl<T: Clone, A: Clone, S: Clone> Clone for SystemView<T, A, S> {
    #[inline]
    fn clone(&self) -> Self { Self { times: self.times.clone(), agreed: self.agreed.clone(), stated: self.stated.clone() } }
}
impl<T: Debug, A: Debug, S: Debug> Debug for SystemView<T, A, S> {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FResult {
        let mut fmt = f.debug_struct("SystemView");
        fmt.field("times", &self.times);
        fmt.field("agreed", &self.agreed);
        fmt.field("stated", &self.stated);
        fmt.finish()
    }
}

// JustAct impls
impl<T: Times, A, S> Times for SystemView<T, A, S> {
    type Error = T::Error;

    #[inline]
    fn current(&self) -> Timestamp { self.times.current() }

    #[inline]
    fn advance_to(&mut self, timestamp: Timestamp) -> Result<(), Self::Error> { self.times.advance_to(timestamp) }
}
impl<T, A: Agreements, S> Agreements for SystemView<T, A, S> {
    type Message<'s> = A::Message<'s> where Self: 's;
    type Error = A::Error;

    #[inline]
    fn agree<'s>(&'s mut self, agr: impl Into<Agreement<Self::Message<'s>>>) -> Result<(), Self::Error> { self.agreed.agree(agr) }

    #[inline]
    fn agreed<'s>(&'s self) -> Set<Agreement<Self::Message<'s>>> { self.agreed.agreed() }
}
impl<T, A, S: Statements> Statements for SystemView<T, A, S> {
    type Message<'s> = S::Message<'s> where Self: 's;
    type Target = S::Target;
    type Status = S::Status;

    #[inline]
    fn state<'s>(&'s mut self, target: Self::Target, msg: impl Into<Self::Message<'s>>) -> Self::Status { self.stated.state(target, msg) }

    #[inline]
    fn stated<'s>(&'s self) -> Set<Self::Message<'s>> { self.stated.stated() }

    #[inline]
    fn enact<'s>(&'s mut self, target: Self::Target, act: impl Into<Action<Self::Message<'s>>>) -> Self::Status { self.stated.enact(target, act) }

    #[inline]
    fn enacted<'s>(&'s self) -> Set<Action<Self::Message<'s>>> { self.stated.enacted() }
}
