//  MESSAGE.rs
//    by Lut99
//
//  Created:
//    23 May 2024, 13:51:32
//  Last edited:
//    23 May 2024, 17:32:25
//  Auto updated?
//    Yes
//
//  Description:
//!   Defines the prototype's notion of a message.
//

use justact_core::auxillary::{Authored, Identifiable};
use justact_core::statements::Message as JMessage;


/***** LIBRARY *****/
/// Defines the prototype's notion of a message.
///
/// This means that it is assumed agents _cannot_ lie about their authorship of a message.
#[derive(Clone, Copy, Debug)]
pub struct Message<'v> {
    /// The identifier of the message.
    pub id:      &'v str,
    /// The author of the message.
    pub author:  &'v str,
    /// The payload of the message.
    pub payload: &'v [u8],
}

impl<'v> Identifiable for Message<'v> {
    type Id = str;

    #[inline]
    fn id(&self) -> &Self::Id { self.id }
}
impl<'v> Authored for Message<'v> {
    type AuthorId = str;

    #[inline]
    fn author(&self) -> &Self::AuthorId { self.author }
}
impl<'v> JMessage<'v> for Message<'v> {
    #[inline]
    fn id_v(&self) -> &'v Self::Id { self.id }

    #[inline]
    fn author_v(&self) -> &'v Self::AuthorId { self.author }

    #[inline]
    fn payload(&self) -> &'v [u8] { self.payload }
}

impl<'v> From<&'v OwnedMessage> for Message<'v> {
    #[inline]
    fn from(value: &'v OwnedMessage) -> Self { Self { id: value.id.as_str(), author: value.author.as_str(), payload: value.payload.as_slice() } }
}



/// Defines the prototype's notion of a message in case it's owned by one of the sets.
///
/// This means that it is assumed agents _cannot_ lie about their authorship of a message.
#[derive(Clone, Debug)]
pub struct OwnedMessage {
    /// The identifier of the message.
    pub id:      String,
    /// The author of the message.
    pub author:  String,
    /// The payload of the message.
    pub payload: Vec<u8>,
}
impl Identifiable for OwnedMessage {
    type Id = str;

    #[inline]
    fn id(&self) -> &Self::Id { self.id.as_str() }
}
impl Authored for OwnedMessage {
    type AuthorId = str;

    #[inline]
    fn author(&self) -> &Self::AuthorId { self.author.as_str() }
}
