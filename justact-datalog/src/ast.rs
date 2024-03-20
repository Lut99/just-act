//  AST.rs
//    by Lut99
//
//  Created:
//    13 Mar 2024, 16:43:37
//  Last edited:
//    20 Mar 2024, 13:52:16
//  Auto updated?
//    Yes
//
//  Description:
//!   Defines the datalog-with-negation AST.
//

use std::hash::{Hash, Hasher};

pub use ast_toolkit_punctuated::punct;
use ast_toolkit_punctuated::Punctuated;
pub use ast_toolkit_span::Span;
use enum_debug::EnumDebug;
// Re-export the derive macro
#[cfg(feature = "derive")]
pub use justact_datalog_derive::datalog;
use paste::paste;


/***** HELPER MACROS *****/
/// Automatically implements `Eq`, `Hash` and `PartialEq` for the given fields in the given struct.
macro_rules! impl_map {
    ($for:ident, $($fields:ident),+) => {
        impl Eq for $for {}

        impl Hash for $for {
            #[inline]
            fn hash<H: Hasher>(&self, state: &mut H) {
                $(
                    self.$fields.hash(state);
                )+
            }
        }

        impl PartialEq for $for {
            #[inline]
            fn eq(&self, other: &Self) -> bool {
                $(
                    self.$fields == other.$fields
                )&&+
            }
        }
    };
}
/// Automatically implements `Eq`, `Hash` and `PartialEq` for a type that is semantically always the same.
///
/// Examples: tokens (no value to change them).
macro_rules! impl_map_invariant {
    ($name:ident) => {
        impl Eq for $name {}
        impl Hash for $name {
            #[inline]
            fn hash<H: Hasher>(&self, _state: &mut H) {}
        }
        impl PartialEq for $name {
            #[inline]
            fn eq(&self, _other: &Self) -> bool { true }

            #[inline]
            fn ne(&self, _other: &Self) -> bool { false }
        }
    };
}
macro_rules! impl_enum_map {
    ($for:ident, $($variants:ident($($fields:ident),+)),+) => {
        impl Eq for $for {}

        impl Hash for $for {
            #[inline]
            fn hash<H: Hasher>(&self, state: &mut H) {
                match self {
                    $(
                        Self::$variants ( $($fields),+ ) => {
                            stringify!($variants).hash(state);
                            $($fields.hash(state);)+
                        }
                    ),+
                }
            }
        }

        paste! {
            impl PartialEq for $for {
                #[inline]
                fn eq(&self, other: &Self) -> bool {
                    match (self, other) {
                        $(
                            (Self::$variants ( $([< $fields _lhs >]),+ ), Self::$variants ( $([< $fields _rhs >]),+ )) => {
                                $([< $fields _lhs >] == [< $fields _rhs >])&&+
                            }
                        ),+

                        // Any other variant is inequal by default
                        (_, _) => false,
                    }
                }
            }
        }
    };
}





/***** LIBRARY *****/
/// The root node that specifies a policy.
///
/// # Syntax
/// ```plain
/// foo :- bar, baz(quz).
/// foo.
/// ```
#[derive(Clone, Debug)]
pub struct Spec {
    /// The list of rules in this program.
    pub rules: Vec<Rule>,
}
impl_map!(Spec, rules);



/// Specifies a single rule.
///
/// # Syntax
/// ```plain
/// foo :- bar, baz(quz).
/// foo.
/// ```
#[derive(Clone, Debug)]
pub struct Rule {
    /// A list of consequences (i.e., instances produced by this rule).
    pub consequences: Punctuated<Atom, Comma>,
    /// An optional second part that describes the antecedents.
    pub tail: Option<RuleAntecedents>,
    /// The closing dot after each rule.
    pub dot: Dot,
}
impl_map!(Rule, consequences, tail, dot);

/// Defines the second half of the rule, if any.
///
/// # Syntax
/// ```plain
/// :- foo, bar(baz)
/// ```
#[derive(Clone, Debug)]
pub struct RuleAntecedents {
    /// The arrow token.
    pub arrow_token: Arrow,
    /// The list of antecedents.
    pub antecedents: Punctuated<Literal, Comma>,
}
impl_map!(RuleAntecedents, arrow_token, antecedents);



/// Represents a single antecedent, as it were.
///
/// # Syntax
/// ```plain
/// foo
/// foo(bar)
/// not foo
/// ```
#[derive(Clone, Debug, EnumDebug)]
pub enum Literal {
    /// Non-negated atom.
    ///
    /// # Syntax
    /// ```plain
    /// foo
    /// foo(bar)
    /// ```
    Atom(Atom),
    /// Negated atom.
    ///
    /// # Syntax
    /// ```plain
    /// not foo
    /// ```
    NegAtom(NegAtom),
}
impl Literal {
    /// Returns the atom that appears in all variants of the literal.
    ///
    /// # Returns
    /// A reference to the [`Atom`] contained within.
    pub fn atom(&self) -> &Atom {
        match self {
            Self::Atom(a) => a,
            Self::NegAtom(na) => &na.atom,
        }
    }

    /// Returns the atom that appears in all variants of the literal.
    ///
    /// # Returns
    /// A mutable reference to the [`Atom`] contained within.
    pub fn atom_mut(&mut self) -> &mut Atom {
        match self {
            Self::Atom(a) => a,
            Self::NegAtom(na) => &mut na.atom,
        }
    }
}
impl_enum_map!(Literal, Atom(atom), NegAtom(atom));

/// Wraps around an [`Atom`] to express its non-existance.
///
/// # Syntax
/// ```plain
/// not foo
/// not foo(bar)
/// ```
#[derive(Clone, Debug)]
pub struct NegAtom {
    /// The not-token.
    pub not_token: Not,
    /// The atom that was negated.
    pub atom:      Atom,
}
impl_map!(NegAtom, not_token, atom);



/// Defines a constructor application of facts.
///
/// # Syntax
/// ```plain
/// foo
/// foo(bar, Baz)
/// ```
#[derive(Clone, Debug)]
pub struct Atom {
    /// The identifier itself.
    pub ident: Ident,
    /// Any arguments.
    pub args:  Option<AtomArgs>,
}
impl Atom {
    /// Creates a new [`Span`] that covers the entire Atom.
    ///
    /// # Returns
    /// A new [`Span`] that is this atom.
    pub fn span(&self) -> Span<&'static str, &'static str> {
        match &self.args {
            Some(args) => self.ident.value.join(&args.paren_tokens.span()).unwrap_or_else(|| self.ident.value.clone()),
            None => self.ident.value.clone(),
        }
    }
}
impl_map!(Atom, ident, args);

/// Defines the (optional) arguments-part of the constructor application.
///
/// # Syntax
/// ```plain
/// (foo, bar(baz))
/// ```
#[derive(Clone, Debug)]
pub struct AtomArgs {
    /// The parenthesis wrapping the arguments.
    pub paren_tokens: Parens,
    /// The arguments contained within.
    pub args: Punctuated<AtomArg, Comma>,
}
impl_map!(AtomArgs, paren_tokens, args);

/// Represents an argument to an Atom, which is either a variable or a nested atom.
///
/// # Syntax
/// ```plain
/// foo
/// foo(bar)
/// Baz
/// ```
#[derive(Clone, Debug, EnumDebug)]
pub enum AtomArg {
    /// It's a nested atom.
    ///
    /// Note that $Datalog^\neg$ does not support full nesting, so only direct identifiers allowed.
    ///
    /// # Syntax
    /// ```plain
    /// foo
    /// ```
    Atom(Ident),
    /// It's a variable.
    ///
    /// # Syntax
    /// ```plain
    /// Foo
    /// ```
    Var(Ident),
}
impl AtomArg {
    /// Returns the identifier that appears in all variants of the AtomArg.
    ///
    /// # Returns
    /// A reference to the [`Ident`] contained within.
    pub fn ident(&self) -> &Ident {
        match self {
            Self::Atom(a) => a,
            Self::Var(v) => v,
        }
    }

    /// Returns the identifier that appears in all variants of the AtomArg.
    ///
    /// # Returns
    /// A mutable reference to the [`Ident`] contained within.
    pub fn ident_mut(&mut self) -> &mut Ident {
        match self {
            Self::Atom(a) => a,
            Self::Var(v) => v,
        }
    }
}
impl_enum_map!(AtomArg, Atom(ident), Var(ident));

/// Represents identifiers.
///
/// # Syntax
/// ```plain
/// foo
/// ```
#[derive(Clone, Copy, Debug)]
pub struct Ident {
    /// The value of the identifier itself.
    pub value: Span<&'static str, &'static str>,
}
impl_map!(Ident, value);



/// Defines an arrow token.
///
/// # Syntax
/// ```plain
/// :-
/// ```
#[derive(Clone, Copy, Debug)]
pub struct Arrow {
    /// The source of this arrow in the source.
    pub span: Span<&'static str, &'static str>,
}
impl_map_invariant!(Arrow);

/// Defines a comma token.
///
/// # Syntax
/// ```plain
/// ,
/// ```
#[derive(Clone, Copy, Debug)]
pub struct Comma {
    /// The source of this comma in the source.
    pub span: Span<&'static str, &'static str>,
}
impl_map_invariant!(Comma);

/// Defines a dot token.
///
/// # Syntax
/// ```plain
/// .
/// ```
#[derive(Clone, Copy, Debug)]
pub struct Dot {
    /// The source of this dot in the source.
    pub span: Span<&'static str, &'static str>,
}
impl_map_invariant!(Dot);

/// Defines a not token.
///
/// # Syntax
/// ```plain
/// not
/// ```
#[derive(Clone, Copy, Debug)]
pub struct Not {
    /// The source of this not in the source.
    pub span: Span<&'static str, &'static str>,
}
impl_map_invariant!(Not);

/// Defines parenthesis.
///
/// # Syntax
/// ```plain
/// ()
/// ```
#[derive(Clone, Copy, Debug)]
pub struct Parens {
    /// The opening-parenthesis.
    pub open:  Span<&'static str, &'static str>,
    /// The closing-parenthesis.
    pub close: Span<&'static str, &'static str>,
}
impl Parens {
    /// Creates a new [`Span`] that covers the entire parentheses' range.
    ///
    /// # Returns
    /// A new [`Span`] that wraps these parenthesis.
    #[inline]
    pub fn span(&self) -> Span<&'static str, &'static str> { self.open.join(&self.close).unwrap_or_else(|| self.open.clone()) }
}
impl_map_invariant!(Parens);
