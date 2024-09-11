//  AST.rs
//    by Lut99
//
//  Created:
//    09 Sep 2024, 14:22:15
//  Last edited:
//    11 Sep 2024, 15:11:36
//  Auto updated?
//    Yes
//
//  Description:
//!   Defines the `Acting` Abstract Syntax Tree (AST).
//

use ast_toolkit_punctuated::Punctuated;
use ast_toolkit_span::{Span, SpannableEq};
use ast_toolkit_tokens::{utf8_delimiter, utf8_token};
use regex::Regex;


/***** TOPLEVEL *****/
/// Defines an action spec as a whole.
#[derive(Clone, Debug)]
pub struct Acting<F, S> {
    /// A file is a list of statements.
    pub stmts: Vec<Stmt<F, S>>,
}

/// Defines a trigger -> action rule.
#[derive(Clone, Debug)]
pub struct Stmt<F, S> {
    /// The label, if any.
    pub label:     Option<StmtLabel<F, S>>,
    /// The `on`-keyword.
    pub on_token:  On<F, S>,
    /// The trigger that this rule triggers on.
    pub trigger:   Trigger<F, S>,
    /// Any actions that are performed by this fule.
    pub actions:   Vec<StmtAction<F, S>>,
    /// The terminating dot.
    pub dot_token: Dot<F, S>,
}

/// Defines the (optional) label preceding rules.
#[derive(Clone, Debug)]
pub struct StmtLabel<F, S> {
    /// The string value of the label.
    pub ident: LitStr<F, S>,
    /// The colon token.
    pub colon_token: Colon<F, S>,
}

/// Defines any action that can be taken.
#[derive(Clone, Debug)]
pub struct StmtAction<F, S> {
    /// The 'do'-token.
    pub do_token: Do<F, S>,
    /// The actual action that can be taken.
    pub action:   Action<F, S>,
}





/***** TRIGGERS *****/
/// Defines the toplevel node of all possible triggers.
#[derive(Clone, Debug)]
pub enum Trigger<F, S> {
    /// It's a trigger that never occurs by itself, only (possibly) by another rule.
    Never(TriggerNever<F, S>),
    /// It's a trigger that always occurs at the start of every scenario.
    Start(TriggerStart<F, S>),
    /// It's a trigger that occurs on the tick of the globally synchronized time.
    Tick(TriggerTick<F, S>),
    /// It's a trigger that occurs when a particular message is received.
    Message(TriggerMessage<F, S>),
    /// It's a trigger that occurs when a message from a particular author is received.
    MessageBy(TriggerMessageBy<F, S>),
    /// It's a trigger that occurs when a message with a particular body is received.
    MessageContains(TriggerMessageContains<F, S>),
}

/// A trigger that never occurs by itself, only (possibly) by another rule.
#[derive(Clone, Copy, Debug)]
pub struct TriggerNever<F, S> {
    /// The never keyword itself.
    pub never_token: Never<F, S>,
}

/// Something that is immediately triggered on scenario start.
#[derive(Clone, Copy, Debug)]
pub struct TriggerStart<F, S> {
    /// The `start`-token itself.
    pub start_token: Start<F, S>,
}

/// Something that is triggered on every tick.
#[derive(Clone, Debug)]
pub struct TriggerTick<F, S> {
    /// The `time`-token.
    pub time_token: Time<F, S>,
    /// The optional expression that limits which ticks are triggered on.
    pub expr: Option<Expr<F, S>>,
}

/// Something that is triggered when a message is sent.
#[derive(Clone, Debug)]
pub struct TriggerMessage<F, S> {
    /// The `message`-keyword.
    pub message_token: Message<F, S>,
    /// The identifier that identifies the message.
    pub id: MessageId<F, S>,
}

/// Something that is triggered when a message by a particular author is sent.
#[derive(Clone, Debug)]
pub struct TriggerMessageBy<F, S> {
    /// The `message`-keyword.
    pub message_token: Message<F, S>,
    /// The `by`-keyword.
    pub by_token: By<F, S>,
    /// The identifier that identifies the author.
    pub author: LitStr<F, S>,
}

/// Something that is triggered when a message with a particular payload is sent.
#[derive(Clone, Debug)]
pub struct TriggerMessageContains<F, S> {
    /// The `message`-keyword.
    pub message_token: Message<F, S>,
    /// The `contains`-keyword.
    pub contains_token: Contains<F, S>,
    /// The regular expression that contains the auhtor.
    pub regex: LitRegex<F, S>,
}





/***** ACTIONS *****/
/// Defines the toplevel node of all possible actions.
#[derive(Clone, Debug)]
pub enum Action<F, S> {
    /// Nothing will happen.
    Nop(ActionNop<F, S>),
    /// Another rule is triggered.
    Trigger(ActionTrigger<F, S>),
    /// Time is advanced.
    Tick(ActionTick<F, S>),
    /// A new agreement is forged.
    Agree(ActionAgree<F, S>),
    /// A new message is stated.
    State(ActionState<F, S>),
    /// A new action is enacted.
    Enact(ActionEnact<F, S>),
}

/// An action that never does anything.
#[derive(Clone, Copy, Debug)]
pub struct ActionNop<F, S> {
    /// The `nop`-token itself.
    pub nop_token: Nop<F, S>,
}

/// An action that triggers another rule.
#[derive(Clone, Debug)]
pub struct ActionTrigger<F, S> {
    /// The `trigger`-keyword itself.
    pub trigger_token: TriggerToken<F, S>,
    /// The rule that was triggered.
    pub rule: RuleId<F, S>,
}

/// An action that moves time forward.
#[derive(Clone, Copy, Debug)]
pub struct ActionTick<F, S> {
    /// The `tick`-keyword itself.
    pub tick_token: Tick<F, S>,
}

/// An action that emits a new agreement.
#[derive(Clone, Debug)]
pub struct ActionAgree<F, S> {
    /// The `agree`-keyword itself.
    pub agree_token: Agree<F, S>,
    /// The identifier of this new message.
    pub id: MessageId<F, S>,
    /// The optional language identifier.
    pub lang: Option<LitLang<F, S>>,
    /// The code to send.
    pub contents: Contents<F, S>,
}

/// An action that states a new message.
#[derive(Clone, Debug)]
pub struct ActionState<F, S> {
    /// The `state`-keyword itself.
    pub state_token: State<F, S>,
    /// The optional scoper to an agent.
    pub to: Option<ActionTo<F, S>>,
    /// The identifier of this new message.
    pub id: MessageId<F, S>,
    /// The optional language identifier.
    pub lang: Option<LitLang<F, S>>,
    /// The code to send.
    pub contents: Contents<F, S>,
}

/// An action that enacts a set of messages as an action.
#[derive(Clone, Debug)]
pub struct ActionEnact<F, S> {
    /// The `enact`-keyword itself.
    pub enact_token: Enact<F, S>,
    /// The optional scoper to an agent.
    pub to: Option<ActionTo<F, S>>,
    /// The comma-separated list of message IDs.
    pub ids: Punctuated<MessageId<F, S>, Comma<F, S>>,
}

/// Defines who to state a message to.
#[derive(Clone, Debug)]
pub struct ActionTo<F, S> {
    /// The `to`-token.
    pub to_token: To<F, S>,
    /// The agent identifier to state it to.
    pub agent:    LitStr<F, S>,
}





/***** CONTENTS *****/
/// Represents the toplevel node for message contents.
#[derive(Clone, Debug)]
pub enum Contents<F, S> {
    /// It's an external file.
    External(ContentsExternal<F, S>),
    /// It's an internal file.
    Inline(ContentsInline<F, S>),
}

/// Represents an externally referenced file.
#[derive(Clone, Debug)]
pub struct ContentsExternal<F, S> {
    /// The `#file`-token.
    pub file_token: File<F, S>,
    /// The string encoding the file's (relative) path.
    pub path: LitStr<F, S>,
}

/// Represents an internally stated message.
#[derive(Clone, Copy, Debug)]
pub struct ContentsInline<F, S> {
    /// The contents themselves, as a span referring the content.
    pub contents:     Span<F, S>,
    /// The delimiting curly brackets.
    pub curly_tokens: Curlies<F, S>,
}





/***** EXPRESSIONS *****/
/// The toplevel node representing an expression.
#[derive(Clone, Debug)]
pub enum Expr<F, S> {
    /// An operator with two operators.
    BinOp(ExprBinOp<F, S>),
    /// A literal was written down.
    Lit(ExprLit<F, S>),
    /// An expression written in parenthesis.
    Parens(ExprParens<F, S>),
}

/// A literal.
#[derive(Clone, Copy, Debug)]
pub struct ExprLit<F, S> {
    /// The literal that was used.
    pub lit: Lit<F, S>,
}

/// An operator.
#[derive(Clone, Debug)]
pub struct ExprBinOp<F, S> {
    /// The operator executed
    pub op:  BinOp<F, S>,
    /// The lefthand-side of the expression.
    pub lhs: Box<Expr<F, S>>,
    /// The righthand-side of the expression.
    pub rhs: Box<Expr<F, S>>,
}
/// Possible binary operations.
#[derive(Clone, Copy, Debug)]
pub enum BinOp<F, S> {
    // Arithmetic
    /// Addition.
    Add(Plus<F, S>),
    /// Subtraction.
    Sub(Dash<F, S>),
    /// Multiplication.
    Mul(Star<F, S>),
    /// Division.
    Div(Slash<F, S>),
    /// Modulo.
    Mod(Percent<F, S>),

    // Logic
    /// Conjunction.
    And(AmperAmper<F, S>),
    /// Disjunction.
    Or(BarBar<F, S>),

    // Equality
    /// Equality.
    Eq(EqualsEquals<F, S>),
    /// Inequality.
    Ne(ExclaimEquals<F, S>),
    /// Greater-than.
    Gt(RightTriangle<F, S>),
    /// Greater-than-or-equals-to.
    Ge(RightTriangleEquals<F, S>),
    /// Less-than.
    Lt(LeftTriangle<F, S>),
    /// Less-than-or-equals-to.
    Le(LeftTriangleEquals<F, S>),
}

/// An expression wrapped in parenthesis.
#[derive(Clone, Debug)]
pub struct ExprParens<F, S> {
    /// The expression in the middle.
    pub expr: Box<Expr<F, S>>,
    /// The delimiter.
    pub paren_tokens: Parens<F, S>,
}





/***** LITERALS *****/
/// The toplevel node representing any kind of literal that can be used in expressions.
#[derive(Clone, Copy, Debug)]
pub enum Lit<F, S> {
    /// It's a boolean.
    Bool(LitBool<F, S>),
    /// It's an integer.
    Int(LitInt<F, S>),
    /// It's the `now`-marker.
    Now(LitNow<F, S>),
}

/// Defines literals that serve as message IDs.
#[derive(Clone, Debug)]
pub enum MessageId<F, S> {
    /// It's an integer literal.
    Int(LitInt<F, S>),
    /// It's a string literal.
    Str(LitStr<F, S>),
}

/// Defines literals that serve as rule IDs.
#[derive(Clone, Debug)]
pub enum RuleId<F, S> {
    /// It's an integer literal.
    Int(LitInt<F, S>),
    /// It's a string literal.
    Str(LitStr<F, S>),
}



/// Defines a boolean literal.
#[derive(Clone, Copy, Debug)]
pub struct LitBool<F, S> {
    /// The value of the literal.
    pub value: bool,
    /// The span where this literal may be found.
    pub span:  Span<F, S>,
}
impl<F, S> Eq for LitBool<F, S> {}
impl<F, S> PartialEq for LitBool<F, S> {
    #[inline]
    fn eq(&self, other: &Self) -> bool { self.value == other.value }
}

/// Defines an integer literal.
#[derive(Clone, Copy, Debug)]
pub struct LitInt<F, S> {
    /// The value of the literal.
    pub value: i64,
    /// The span where this literal may be found.
    pub span:  Span<F, S>,
}
impl<F, S> Eq for LitInt<F, S> {}
impl<F, S> PartialEq for LitInt<F, S> {
    #[inline]
    fn eq(&self, other: &Self) -> bool { self.value == other.value }
}

/// Defines a language literal.
#[derive(Clone, Copy, Debug)]
pub struct LitLang<F, S> {
    /// The value of the literal.
    pub value: Span<F, S>,
    /// The delimiting `<>`-tokens.
    pub triangle_tokens: Triangles<F, S>,
}
impl<F, S: SpannableEq> Eq for LitLang<F, S> {}
impl<F, S: SpannableEq> PartialEq for LitLang<F, S> {
    #[inline]
    fn eq(&self, other: &Self) -> bool { self.value == other.value }
}

/// Defines the `now`-literal.
#[derive(Clone, Copy, Debug)]
pub struct LitNow<F, S> {
    /// The token itself.
    pub now_token: Now<F, S>,
}
impl<F, S> Eq for LitNow<F, S> {}
impl<F, S> PartialEq for LitNow<F, S> {
    #[inline]
    fn eq(&self, _other: &Self) -> bool { true }
    #[inline]
    fn ne(&self, _other: &Self) -> bool { false }
}

/// Defines a regex literal.
#[derive(Clone, Debug)]
pub struct LitRegex<F, S> {
    /// The parsed regex value.
    pub value: Regex,
    /// The span pointing to the value of the regex literal.
    pub value_span: Span<F, S>,
    /// The quotes.
    pub quote_tokens: RQuotes<F, S>,
}
impl<F, S> Eq for LitRegex<F, S> {}
impl<F, S> PartialEq for LitRegex<F, S> {
    #[inline]
    fn eq(&self, other: &Self) -> bool { self.value.as_str() == other.value.as_str() }
}

/// Defines a string literal.
#[derive(Clone, Debug)]
pub struct LitStr<F, S> {
    /// The value of the string literal.
    pub value: String,
    /// The quotes.
    pub quote_tokens: Quotes<F, S>,
}
impl<F, S> Eq for LitStr<F, S> {}
impl<F, S> PartialEq for LitStr<F, S> {
    #[inline]
    fn eq(&self, other: &Self) -> bool { self.value == other.value }
}





/***** TOKENS *****/
utf8_token!(Agree, "agree");
utf8_token!(AmperAmper, "&&");
utf8_token!(BarBar, "||");
utf8_token!(By, "by");
utf8_token!(Colon, ":");
utf8_token!(Comma, ",");
utf8_token!(Contains, "contains");
utf8_token!(Dash, "-");
utf8_token!(Do, "do");
utf8_token!(Dot, ".");
utf8_token!(Enact, "enact");
utf8_token!(EqualsEquals, "==");
utf8_token!(ExclaimEquals, "!=");
utf8_token!(File, "#file");
utf8_token!(LeftTriangle, "<");
utf8_token!(LeftTriangleEquals, "<=");
utf8_token!(Message, "message");
utf8_token!(Never, "never");
utf8_token!(Nop, "nop");
utf8_token!(Now, "now");
utf8_token!(On, "on");
utf8_token!(Percent, "%");
utf8_token!(Plus, "+");
utf8_token!(RightTriangle, ">");
utf8_token!(RightTriangleEquals, ">=");
utf8_token!(Slash, "/");
utf8_token!(Star, "*");
utf8_token!(Start, "start");
utf8_token!(State, "state");
utf8_token!(Tick, "tick");
utf8_token!(Time, "time");
utf8_token!(To, "to");
utf8_token!(TriggerToken, "trigger");

utf8_delimiter!(Curlies, "{", "}");
utf8_delimiter!(Parens, "(", ")");
utf8_delimiter!(Quotes, "\"", "\"");
utf8_delimiter!(RQuotes, "r\"", "\"");
utf8_delimiter!(Triangles, "<", ">");
