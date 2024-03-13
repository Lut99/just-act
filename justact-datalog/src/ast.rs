//  AST.rs
//    by Lut99
//
//  Created:
//    13 Mar 2024, 16:43:37
//  Last edited:
//    13 Mar 2024, 17:59:19
//  Auto updated?
//    Yes
//
//  Description:
//!   Defines the datalog-with-negation AST.
//

use ast_toolkit_punctuated::Punctuated;
use ast_toolkit_span::Span;
use enum_debug::EnumDebug;


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
    /// An optional second part that describes the antecedants.
    pub antecedent: Option<RuleAntecedent<F, S>>,
    /// The closing dot after each rule.
    pub dot: Dot<F, S>,
}

/// Defines the second half of the rule, if any.
///
/// # Syntax
/// ```plain
/// :- foo, bar(baz)
/// ```
#[derive(Clone, Debug)]
pub struct RuleAntecedent<F, S> {
    /// The arrow token.
    pub arrow_token: Arrow<F, S>,
    /// The list of antecedents.
    pub antecedants: Punctuated<Literal<F, S>, Comma<F, S>>,
}



/// Represents a single antecedant, as it were.
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
    /// ```
    /// foo
    /// foo(bar)
    /// ```
    Atom(Atom<F, S>),
    /// Negated atom.
    ///
    /// # Syntax
    /// ```
    /// not foo
    /// ```
    NegAtom(NegAtom<F, S>),
}

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
    /// # Syntax
    /// ```plain
    /// foo
    /// foo(bar)
    /// ```
    Atom(Atom<F, S>),
    /// It's a variable.
    ///
    /// # Syntax
    /// ```plain
    /// Foo
    /// ```
    Var(Ident<F, S>),
}

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
