//  AST.rs
//    by Lut99
//
//  Created:
//    13 Mar 2024, 16:43:37
//  Last edited:
//    19 Mar 2024, 11:25:52
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
use ast_toolkit_span::Spannable;
use enum_debug::EnumDebug;
// Re-export the derive macro
#[cfg(feature = "derive")]
pub use justact_datalog_derive::datalog;
use paste::paste;


/***** HELPER MACROS *****/
/// Automatically implements `Eq`, `Hash` and `PartialEq` for the given fields in the given struct.
macro_rules! impl_map {
    ($for:ident, $($fields:ident),+) => {
        impl<F, S> Eq for $for<F, S>
        where
            S: Spannable,
            for<'s> S::Slice<'s>: Eq,
        {}

        impl<F, S> Hash for $for<F, S>
        where
            S: Spannable,
            for<'s> S::Slice<'s>: Hash,
        {
            #[inline]
            fn hash<H: Hasher>(&self, state: &mut H) {
                $(
                    self.$fields.hash(state);
                )+
            }
        }

        impl<F, S> PartialEq for $for<F, S>
        where
            S: Spannable,
            for<'s> S::Slice<'s>: PartialEq,
        {
            #[inline]
            fn eq(&self, other: &Self) -> bool {
                $(
                    self.$fields == other.$fields
                )&&+
            }
        }
    };
}
macro_rules! impl_enum_map {
    ($for:ident, $($variants:ident($($fields:ident),+)),+) => {
        impl<F, S> Eq for $for<F, S>
        where
            S: Spannable,
            for<'s> S::Slice<'s>: Eq,
        {}

        impl<F, S> Hash for $for<F, S>
        where
            S: Spannable,
            for<'s> S::Slice<'s>: Hash,
        {
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
            impl<F, S> PartialEq for $for<F, S>
            where
                S: Spannable,
                for<'s> S::Slice<'s>: PartialEq,
            {
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
pub struct Spec<F, S> {
    /// The list of rules in this program.
    pub rules: Vec<Rule<F, S>>,
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
pub struct Rule<F, S> {
    /// A list of consequences (i.e., instances produced by this rule).
    pub consequences: Punctuated<Atom<F, S>, Comma<F, S>>,
    /// An optional second part that describes the antecedents.
    pub tail: Option<RuleAntecedents<F, S>>,
    /// The closing dot after each rule.
    pub dot: Dot<F, S>,
}
impl_map!(Rule, consequences, tail, dot);

/// Defines the second half of the rule, if any.
///
/// # Syntax
/// ```plain
/// :- foo, bar(baz)
/// ```
#[derive(Clone, Debug)]
pub struct RuleAntecedents<F, S> {
    /// The arrow token.
    pub arrow_token: Arrow<F, S>,
    /// The list of antecedents.
    pub antecedents: Punctuated<Literal<F, S>, Comma<F, S>>,
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
pub enum Literal<F, S> {
    /// Non-negated atom.
    ///
    /// # Syntax
    /// ```plain
    /// foo
    /// foo(bar)
    /// ```
    Atom(Atom<F, S>),
    /// Negated atom.
    ///
    /// # Syntax
    /// ```plain
    /// not foo
    /// ```
    NegAtom(NegAtom<F, S>),
}
impl<F, S> Literal<F, S> {
    /// Returns the atom that appears in all variants of the literal.
    ///
    /// # Returns
    /// A reference to the [`Atom`] contained within.
    pub fn atom(&self) -> &Atom<F, S> {
        match self {
            Self::Atom(a) => a,
            Self::NegAtom(na) => &na.atom,
        }
    }

    /// Returns the atom that appears in all variants of the literal.
    ///
    /// # Returns
    /// A mutable reference to the [`Atom`] contained within.
    pub fn atom_mut(&mut self) -> &mut Atom<F, S> {
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
pub struct NegAtom<F, S> {
    /// The not-token.
    pub not_token: Not<F, S>,
    /// The atom that was negated.
    pub atom:      Atom<F, S>,
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
pub struct Atom<F, S> {
    /// The identifier itself.
    pub ident: Ident<F, S>,
    /// Any arguments.
    pub args:  Option<AtomArgs<F, S>>,
}
impl<F: Clone, S: Clone + Spannable> Atom<F, S> {
    /// Creates a new [`Span`] that covers the entire Atom.
    ///
    /// # Returns
    /// A new [`Span`] that is this atom.
    pub fn span(&self) -> Span<F, S> {
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
pub struct AtomArgs<F, S> {
    /// The parenthesis wrapping the arguments.
    pub paren_tokens: Parens<F, S>,
    /// The arguments contained within.
    pub args: Punctuated<AtomArg<F, S>, Comma<F, S>>,
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
pub enum AtomArg<F, S> {
    /// It's a nested atom.
    ///
    /// Note that $Datalog^\neg$ does not support full nesting, so only direct identifiers allowed.
    ///
    /// # Syntax
    /// ```plain
    /// foo
    /// ```
    Atom(Ident<F, S>),
    /// It's a variable.
    ///
    /// # Syntax
    /// ```plain
    /// Foo
    /// ```
    Var(Ident<F, S>),
}
impl_enum_map!(AtomArg, Atom(ident), Var(ident));

/// Represents identifiers.
///
/// # Syntax
/// ```plain
/// foo
/// ```
#[derive(Clone, Copy, Debug)]
pub struct Ident<F, S> {
    /// The value of the identifier itself.
    pub value: Span<F, S>,
}
impl_map!(Ident, value);



/// Defines an arrow token.
///
/// # Syntax
/// ```plain
/// :-
/// ```
#[derive(Clone, Copy, Debug)]
pub struct Arrow<F, S> {
    /// The source of this arrow in the source.
    pub span: Span<F, S>,
}
impl_map!(Arrow, span);

/// Defines a comma token.
///
/// # Syntax
/// ```plain
/// ,
/// ```
#[derive(Clone, Copy, Debug)]
pub struct Comma<F, S> {
    /// The source of this comma in the source.
    pub span: Span<F, S>,
}
impl_map!(Comma, span);

/// Defines a dot token.
///
/// # Syntax
/// ```plain
/// .
/// ```
#[derive(Clone, Copy, Debug)]
pub struct Dot<F, S> {
    /// The source of this dot in the source.
    pub span: Span<F, S>,
}
impl_map!(Dot, span);

/// Defines a not token.
///
/// # Syntax
/// ```plain
/// not
/// ```
#[derive(Clone, Copy, Debug)]
pub struct Not<F, S> {
    /// The source of this not in the source.
    pub span: Span<F, S>,
}
impl_map!(Not, span);

/// Defines parenthesis.
///
/// # Syntax
/// ```plain
/// ()
/// ```
#[derive(Clone, Copy, Debug)]
pub struct Parens<F, S> {
    /// The opening-parenthesis.
    pub open:  Span<F, S>,
    /// The closing-parenthesis.
    pub close: Span<F, S>,
}
impl<F: Clone, S: Clone + Spannable> Parens<F, S> {
    /// Creates a new [`Span`] that covers the entire parentheses' range.
    ///
    /// # Returns
    /// A new [`Span`] that wraps these parenthesis.
    #[inline]
    pub fn span(&self) -> Span<F, S> { self.open.join(&self.close).unwrap_or_else(|| self.open.clone()) }
}
impl_map!(Parens, open, close);
