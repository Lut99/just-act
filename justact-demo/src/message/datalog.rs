//  DATALOG.rs
//    by Lut99
//
//  Created:
//    15 Apr 2024, 16:22:55
//  Last edited:
//    15 Apr 2024, 19:07:25
//  Auto updated?
//    Yes
//
//  Description:
//!   Defines how messages carrying $Datalog^\neg$ look like in the demo environment.
//

use std::collections::HashSet;

use justact_core::message as justact;
use justact_policy::datalog;
use justact_policy::datalog::ast::Rule;


/***** LIBRARY *****/
/// Defines a [`Message`](justact::Message) that carries $Datalog^\neg$ policy information.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Message {
    /// The author of this message.
    author: &'static str,
    /// Some policy that is emitted here.
    policy: datalog::Policy<'static>,
}
impl justact::Message for Message {
    type Author = &'static str;

    #[inline]
    fn author(&self) -> Self::Author { self.author }
}



/// Implements a _meaningful_ collection of $Datalog^\neg$ policy messages.
#[derive(Clone, Debug)]
pub struct MessageSet {
    /// The actual messages wrapped.
    messages: HashSet<Message>,
}
impl justact::MessageSet for MessageSet {
    type Message = Message;
    type Policy<'s> = datalog::Policy<'s>;

    fn extract<'s>(&'s self) -> Self::Policy<'s> {
        // Combine all the policies
        let mut rules: Vec<&Rule> = Vec::with_capacity(self.messages.len());
        for message in &self.messages {
            rules.extend(message.policy.0.iter());
        }
        Policy
    }
}
