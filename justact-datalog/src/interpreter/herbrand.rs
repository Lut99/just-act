//  HERBRAND.rs
//    by Lut99
//
//  Created:
//    21 Mar 2024, 10:55:27
//  Last edited:
//    25 Mar 2024, 17:31:59
//  Auto updated?
//    Yes
//
//  Description:
//!   Implements iterators for the Herbrand instantiation of a program.
//


/***** TESTS *****/
#[cfg(all(test, feature = "derive"))]
mod tests {
    use ast_toolkit_punctuated::Punctuated;
    use ast_toolkit_span::Span;

    use super::*;
    use crate::ast::{Atom, AtomArg, AtomArgs, Comma, Ident, Parens};


    /// Sets up a logger if wanted.
    #[cfg(feature = "log")]
    fn setup_logger() {
        use humanlog::{DebugMode, HumanLogger};

        // Check if the envs tell us to
        if let Ok(logger) = std::env::var("LOGGER") {
            if logger == "1" || logger == "true" {
                // Create the logger
                if let Err(err) = HumanLogger::terminal(DebugMode::Full).init() {
                    eprintln!("WARNING: Failed to setup logger: {err} (no logging for this session)");
                }
            }
        }
    }

    /// Makes an [`Ident`] conveniently.
    fn make_ident(name: &'static str) -> Ident { Ident { value: Span::new("make_ident::value", name) } }

    /// Makes an [`Atom`] conveniently.
    fn make_atom(name: &'static str, args: impl IntoIterator<Item = &'static str>) -> Atom {
        // Make the punctuation
        let mut punct: Punctuated<AtomArg, Comma> = Punctuated::new();
        for (i, arg) in args.into_iter().enumerate() {
            if i == 0 {
                punct.push_first(AtomArg::Atom(Ident { value: Span::new("make_atom::arg", arg) }));
            } else {
                punct.push(Comma { span: Span::new("make_atom::arg::comma", ",") }, AtomArg::Atom(Ident { value: Span::new("make_atom::arg", arg) }));
            }
        }

        // Make the atom
        Atom {
            ident: Ident { value: Span::new("make_atom::name", name) },
            args:  if !punct.is_empty() {
                Some(AtomArgs {
                    paren_tokens: Parens { open: Span::new("make_atom::parens::open", "("), close: Span::new("make_atom::parens::close", ")") },
                    args: punct,
                })
            } else {
                None
            },
        }
    }
}





/***** LIBRARY *****/
/// Finds the Herbrand Instantiation of a particular [`Spec`].
/// 
/// This is simply 
