//  LIB.rs
//    by Lut99
//
//  Created:
//    04 Sep 2024, 14:43:58
//  Last edited:
//    11 Sep 2024, 14:53:45
//  Auto updated?
//    Yes
//
//  Description:
//!   This crate implements the Acting language (`.act`), which is a
//!   declarative programming language that programs agents in JustAct in
//!   a simple way.
//

// Declare the modules
pub mod ast;
#[cfg(feature = "parser")]
pub mod parser;
