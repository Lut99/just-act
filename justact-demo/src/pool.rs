//  POOL.rs
//    by Lut99
//
//  Created:
//    15 Apr 2024, 16:16:19
//  Last edited:
//    15 Apr 2024, 16:21:35
//  Auto updated?
//    Yes
//
//  Description:
//!   Provides an implementation for the JustAct version of a
//!   [`MessagePool`](justact_core::world::MessagePool).
//


/***** LIBRARY *****/
/// Implements a [`MessagePool`](justact_core::world::MessagePool) with a _total_ view on all messages.
///
/// This means that all agents see the exact same messages.
pub struct MessagePool {}
// impl justact_core::world::MessagePool for MessagePool {
//     type Collection<'s> = ();

//     fn all<'s>(&'s mut self) -> Self::Collection<'s> {}
// }
