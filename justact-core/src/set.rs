//  SET.rs
//    by Lut99
//
//  Created:
//    16 Apr 2024, 10:14:23
//  Last edited:
//    18 Apr 2024, 15:31:44
//  Auto updated?
//    Yes
//
//  Description:
//!   Defines an abstract [`Set`] that can hold a (potentially!)
//!   unordered set of messages or actions.
//

use std::borrow::Cow;

use crate::message::{Action, Message};
use crate::policy::Policy;


/***** LIBRARY *****/
/// Defines an abstract collection of messages or actions.
///
/// The collection is conceptually unordered. Depending on implementations, though, it may be practically ordered, but this should be ignored for correct implementations.
pub trait Set {
    /// The type contained in the Set.
    type Elem;

    /// The type returned by [`Set::iter()`]'s iterator.
    type Item<'s>
    where
        Self: 's;
    /// The iterator returned by [`Set::iter()`].
    type Iter<'s>: Iterator<Item = Self::Item<'s>>
    where
        Self: 's;


    /// Returns some iterator over references to the internal element.
    ///
    /// # Returns
    /// Something of type `Self::Iter` that returns `&T`.
    fn iter<'s>(&'s self) -> Self::Iter<'s>;

    /// Inserts a new element into this collection.
    ///
    /// # Arguments
    /// - `elem`: Some element of type `T` to add.
    ///
    /// # Returns
    /// True if this element already existed, or false otherwise.
    fn add(&mut self, new_elem: Self::Elem) -> bool;
}



/// Defines a collection of messages.
///
/// This is a particular set of messages that can be interpreted as a [`Policy`].
pub trait MessageSet: From<Self::Message> + Set<Elem = Self::Message> {
    /// The type of messages which are contained in this MessageSet.
    type Message: Message;
    /// The type of policy extracted from this message.
    type Policy<'s>: 's + Policy
    where
        Self: 's;


    /// Returns some policy from the fragments contained in the messages of this set.
    ///
    /// # Returns
    /// A new policy of type [`Self::Policy`](MessageSet::Policy) that is the extracted policy.
    fn extract<'s>(&'s self) -> Self::Policy<'s>;
}

// Implement the `MessageSet` for pointer-like types.
impl<'a, T> MessageSet for Cow<'a, T>
where
    T: Clone + MessageSet,
    Cow<'a, T>: Set<Elem = T::Message>,
    Cow<'a, T>: From<T::Message>,
{
    type Message = T::Message;
    type Policy<'s> = T::Policy<'s> where Self: 's;

    #[inline]
    fn extract<'s>(&'s self) -> Self::Policy<'s> { T::extract(self) }
}



/// Defines a collection of actions.
pub trait ActionSet: From<Self::Action> + Set<Elem = Self::Action> {
    /// The type of actions which are contained in this ActionSet.
    type Action: Action;
}
