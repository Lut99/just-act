//  ITER.rs
//    by Lut99
//
//  Created:
//    23 May 2024, 10:08:17
//  Last edited:
//    23 May 2024, 10:21:53
//  Auto updated?
//    Yes
//
//  Description:
//!   Defines a couple of useful iterator transformations when working
//!   with streams of messages.
//

use crate::statements::Message;


/***** LIBRARY *****/
/// Allows one to call [`FindIdentifierIterator::find_id()`] on iterators over [`Message`]s.
pub trait FindIdentifierIterator {
    /// Returns the first [`Message`] yielded by this iterator which has the given identifier.
    ///
    /// # Arguments
    /// - `id`: The identifier to search for.
    ///
    /// # Returns
    /// [`Some`] with the message that has the given `id`entifier, or else [`None`] if the iterator yielded [`None`] before any such message was found.
    fn find_id<'v, M, I>(self, id: I) -> Option<&'v M>
    where
        Self: Sized + Iterator<Item = &'v M>,
        M: 'v + Message<'v>,
        I: PartialEq<M::Id>;
}
impl<I2> FindIdentifierIterator for I2 {
    #[inline]
    fn find_id<'v, M, I>(mut self, id: I) -> Option<&'v M>
    where
        Self: Sized + Iterator<Item = &'v M>,
        M: 'v + Message<'v>,
        I: PartialEq<M::Id>,
    {
        // Iterate over `self` to find a matching message
        while let Some(next) = self.next() {
            if id == *next.id() {
                return Some(next);
            }
        }

        // No found
        None
    }
}



/// Filters a stream of [`Message`]s by author.
#[derive(Debug)]
pub struct FilterAuthor<I, A> {
    /// The iterator we filter.
    iter:      I,
    /// The IDs we filter by.
    author_id: A,
}
impl<'v, M, I, A> Iterator for FilterAuthor<I, A>
where
    M: 'v + Message<'v>,
    I: Iterator<Item = &'v M>,
    A: PartialEq<M::AuthorId>,
{
    type Item = &'v M;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        // Find the next one that is made by the internal author
        while let Some(next) = self.iter.next() {
            if self.author_id == *next.author() {
                return Some(next);
            }
        }

        // Else, nothing left
        None
    }
}

/// Allows one to call [`FilterAuthorIterator::filter_author()`] on iterators over [`Message`]s.
pub trait FilterAuthorIterator {
    /// Filters the messages in this iterator by author.
    ///
    /// # Arguments
    /// - `id`: The author's ID to filter on.
    ///
    /// # Returns
    /// An [`Iterator`] that only yields messages that are authored by the author with the given `id`.
    fn filter_author<'v, M, A>(self, id: A) -> FilterAuthor<Self, A>
    where
        Self: Sized + Iterator<Item = &'v M>,
        M: 'v + Message<'v>,
        A: PartialEq<M::AuthorId>;
}
impl<I> FilterAuthorIterator for I {
    #[inline]
    fn filter_author<'v, M, A>(self, id: A) -> FilterAuthor<Self, A>
    where
        Self: Sized + Iterator<Item = &'v M>,
        M: 'v + Message<'v>,
        A: PartialEq<M::AuthorId>,
    {
        FilterAuthor { iter: self, author_id: id }
    }
}
