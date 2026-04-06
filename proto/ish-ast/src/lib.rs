// ish-ast: Core AST node types for the ish language prototype.
//
// Programs are represented as trees of Expression, Statement, and Program nodes.
// All nodes support serde serialization for JSON round-tripping.

pub mod builder;
pub mod display;

use serde::{Deserialize, Serialize};

// ── Source location tracking ────────────────────────────────────────────────

/// Optional source span for error reporting.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

// ── Incomplete input detection ──────────────────────────────────────────────

/// Identifies which delimited construct was left unterminated.
/// The parser produces `Incomplete` AST nodes instead of errors so it always
/// succeeds (parser-matches-everything philosophy).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum IncompleteKind {
    // Brace-delimited (6)
    Block,
    ObjectLiteral,
    Match,
    EntryTypeDef,
    ObjectType,
    DeclareBlock,
    // Bracket-delimited (5)
    ListLiteral,
    StandardDef,
    StandardAnnotation,
    EntryAnnotation,
    IndexAccess,
    // Paren-delimited (9)
    ParenExpr,
    CallArgs,
    FnParams,
    LambdaParams,
    WithResources,
    CatchParam,
    CommandSubstitution,
    TupleType,
    FunctionType,
    // String-delimited (11)
    StringLiteral,
    InterpString,
    TripleSingleString,
    TripleDoubleString,
    CharLiteral,
    ExtendedDoubleString,
    ExtendedSingleString,
    ExtendedTripleDoubleString,
    ExtendedTripleSingleString,
    ShellQuotedString,
    ShellSingleString,
    // Comment / angle-bracket (3)
    BlockComment,
    GenericParams,
    GenericType,
    // Concurrency (2)
    AwaitNonCall,
    SpawnNonCall,
}

impl IncompleteKind {
    /// Returns `true` if the REPL should wait for more input (multiline
    /// continuation). Returns `false` for single-line constructs that
    /// cannot span lines — those are reported as immediate errors.
    pub fn is_continuable(&self) -> bool {
        !matches!(
            self,
            IncompleteKind::StringLiteral
                | IncompleteKind::InterpString
                | IncompleteKind::CharLiteral
                | IncompleteKind::ExtendedDoubleString
                | IncompleteKind::ExtendedSingleString
                | IncompleteKind::ShellQuotedString
                | IncompleteKind::ShellSingleString
        )
    }
}

// ── Type annotations ────────────────────────────────────────────────────────

pub use ish_core::TypeAnnotation;

// ── AST node types ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Program {
    pub statements: Vec<Statement>,
}

// ── Expressions ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Literal {
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Char(char),
    Null,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BinaryOperator {
    // Arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    // Comparison
    Eq,
    NotEq,
    Lt,
    Gt,
    LtEq,
    GtEq,
    // Logical
    And,
    Or,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UnaryOperator {
    Not,
    Negate,
    Try,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    pub type_annotation: Option<TypeAnnotation>,
    pub default_value: Option<Expression>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Expression {
    Literal(Literal),
    Identifier(String),
    BinaryOp {
        op: BinaryOperator,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    UnaryOp {
        op: UnaryOperator,
        operand: Box<Expression>,
    },
    FunctionCall {
        callee: Box<Expression>,
        args: Vec<Expression>,
    },
    ObjectLiteral(Vec<(String, Expression)>),
    ListLiteral(Vec<Expression>),
    PropertyAccess {
        object: Box<Expression>,
        property: String,
    },
    IndexAccess {
        object: Box<Expression>,
        index: Box<Expression>,
    },
    Lambda {
        params: Vec<Parameter>,
        body: Box<Statement>, // must be a Block
        is_async: bool,
    },
    Await {
        expr: Box<Expression>,
    },
    Spawn {
        callee: Box<Expression>,
        args: Vec<Expression>,
    },
    StringInterpolation(Vec<StringPart>),
    CommandSubstitution(Box<Statement>), // $(...) — wraps a ShellCommand or pipeline
    EnvVar(String), // $HOME or ${PATH}
    Incomplete {
        kind: IncompleteKind,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StringPart {
    Text(String),
    Expr(Expression),
}

// ── Statements ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Statement {
    VariableDecl {
        name: String,
        mutable: bool,
        type_annotation: Option<TypeAnnotation>,
        value: Expression,
        visibility: Option<Visibility>,
    },
    Assignment {
        target: AssignTarget,
        value: Expression,
    },
    Block {
        statements: Vec<Statement>,
    },
    If {
        condition: Expression,
        then_block: Box<Statement>, // must be a Block
        else_block: Option<Box<Statement>>, // Block or another If
    },
    While {
        condition: Expression,
        body: Box<Statement>, // must be a Block
        yield_every: Option<Expression>,
    },
    ForEach {
        variable: String,
        iterable: Expression,
        body: Box<Statement>, // must be a Block
        yield_every: Option<Expression>,
    },
    Return {
        value: Option<Expression>,
    },
    ExpressionStmt(Expression),
    FunctionDecl {
        name: String,
        params: Vec<Parameter>,
        return_type: Option<TypeAnnotation>,
        body: Box<Statement>, // must be a Block
        visibility: Option<Visibility>,
        type_params: Vec<String>, // generic type parameters: <T, U>
        is_async: bool,
    },
    Throw {
        value: Expression,
    },
    TryCatch {
        body: Box<Statement>, // must be a Block
        catches: Vec<CatchClause>,
        finally: Option<Box<Statement>>, // must be a Block if present
    },
    WithBlock {
        resources: Vec<(String, Expression)>,
        body: Box<Statement>, // must be a Block
    },
    Defer {
        body: Box<Statement>,
    },
    TypeAlias {
        name: String,
        definition: TypeAnnotation,
        visibility: Option<Visibility>,
    },
    Use {
        module_path: Vec<String>,
        alias: Option<String>,
        selective: Option<Vec<SelectiveImport>>,
    },
    DeclareBlock {
        body: Vec<Statement>,
    },
    Bootstrap {
        source: BootstrapSource,
    },
    ShellCommand {
        command: String,
        args: Vec<ShellArg>,
        pipes: Vec<ShellPipeline>,     // | chaining
        redirections: Vec<Redirection>,
        background: bool,              // trailing &
    },
    /// @standard[name] or @[entry, ...] before a declaration
    Annotated {
        annotations: Vec<Annotation>,
        inner: Box<Statement>,
    },
    /// standard name extends? base [features]
    StandardDef {
        name: String,
        extends: Option<String>,
        features: Vec<FeatureSpec>,
    },
    /// entry type name { ... }
    EntryTypeDef {
        name: String,
        fields: Vec<(String, Expression)>,
    },
    Match {
        subject: Expression,
        arms: Vec<MatchArm>,
    },
    /// Explicit yield statement — yields control to other tasks
    Yield,
    Incomplete {
        kind: IncompleteKind,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MatchArm {
    pub pattern: MatchPattern,
    pub body: Statement, // block or expression_stmt
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MatchPattern {
    Literal(Literal),                   // 0, "hello", true, null
    Identifier(String),                 // x (variable binding)
    Wildcard,                           // _
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ShellArg {
    Bare(String),           // bare-word argument (including flags like -la)
    Quoted(String),         // "double-quoted" string
    Glob(String),           // *.rs, file?.txt
    EnvVar(String),         // $HOME, ${PATH}
    CommandSub(Box<Statement>), // $(cmd)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ShellPipeline {
    pub command: String,
    pub args: Vec<ShellArg>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Redirection {
    pub kind: RedirectKind,
    pub target: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RedirectKind {
    StdoutWrite,      // >
    StdoutAppend,     // >>
    StderrWrite,      // 2>
    StderrAndStdout,  // 2>&1
    AllWrite,         // &>
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Visibility {
    Priv,   // priv — current module only
    Pkg,    // pkg — all project members (default when omitted)
    Pub,    // pub — external dependents
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SelectiveImport {
    pub name: String,
    pub alias: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BootstrapSource {
    Path(String),
    Url(String),
    Inline(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Annotation {
    Standard(String),                       // @standard[name]
    Entry(Vec<EntryItem>),                  // @[entry, entry(value), ...]
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntryItem {
    pub name: String,
    pub value: Option<String>,              // e.g. type(i32) → value = Some("i32")
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FeatureSpec {
    pub name: String,
    pub params: Vec<String>,                // e.g. overflow(saturating) → params = ["saturating"]
}

/// A catch clause within a try/catch statement.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CatchClause {
    pub param: String,
    pub type_annotation: Option<TypeAnnotation>,
    pub body: Statement, // must be a Block
}

/// Target of an assignment (variable, property, or index).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AssignTarget {
    Variable(String),
    Property {
        object: Box<Expression>,
        property: String,
    },
    Index {
        object: Box<Expression>,
        index: Box<Expression>,
    },
}

// ── Convenience constructors ────────────────────────────────────────────────

impl Expression {
    pub fn int(v: i64) -> Self {
        Expression::Literal(Literal::Int(v))
    }
    pub fn float(v: f64) -> Self {
        Expression::Literal(Literal::Float(v))
    }
    pub fn bool(v: bool) -> Self {
        Expression::Literal(Literal::Bool(v))
    }
    pub fn string(v: impl Into<String>) -> Self {
        Expression::Literal(Literal::String(v.into()))
    }
    pub fn char_lit(c: char) -> Self {
        Expression::Literal(Literal::Char(c))
    }
    pub fn null() -> Self {
        Expression::Literal(Literal::Null)
    }
    pub fn ident(name: impl Into<String>) -> Self {
        Expression::Identifier(name.into())
    }
    pub fn binary(op: BinaryOperator, left: Expression, right: Expression) -> Self {
        Expression::BinaryOp {
            op,
            left: Box::new(left),
            right: Box::new(right),
        }
    }
    pub fn unary(op: UnaryOperator, operand: Expression) -> Self {
        Expression::UnaryOp {
            op,
            operand: Box::new(operand),
        }
    }
    pub fn call(callee: Expression, args: Vec<Expression>) -> Self {
        Expression::FunctionCall {
            callee: Box::new(callee),
            args,
        }
    }
    pub fn object(pairs: Vec<(impl Into<String>, Expression)>) -> Self {
        Expression::ObjectLiteral(pairs.into_iter().map(|(k, v)| (k.into(), v)).collect())
    }
    pub fn list(elements: Vec<Expression>) -> Self {
        Expression::ListLiteral(elements)
    }
    pub fn property(object: Expression, property: impl Into<String>) -> Self {
        Expression::PropertyAccess {
            object: Box::new(object),
            property: property.into(),
        }
    }
    pub fn index(object: Expression, idx: Expression) -> Self {
        Expression::IndexAccess {
            object: Box::new(object),
            index: Box::new(idx),
        }
    }
    pub fn lambda(params: Vec<Parameter>, body: Statement) -> Self {
        Expression::Lambda {
            params,
            body: Box::new(body),
            is_async: false,
        }
    }
}

impl Statement {
    pub fn var_decl(name: impl Into<String>, value: Expression) -> Self {
        Statement::VariableDecl {
            name: name.into(),
            mutable: false,
            type_annotation: None,
            value,
            visibility: None,
        }
    }
    pub fn assign(name: impl Into<String>, value: Expression) -> Self {
        Statement::Assignment {
            target: AssignTarget::Variable(name.into()),
            value,
        }
    }
    pub fn assign_property(object: Expression, property: impl Into<String>, value: Expression) -> Self {
        Statement::Assignment {
            target: AssignTarget::Property {
                object: Box::new(object),
                property: property.into(),
            },
            value,
        }
    }
    pub fn assign_index(object: Expression, index: Expression, value: Expression) -> Self {
        Statement::Assignment {
            target: AssignTarget::Index {
                object: Box::new(object),
                index: Box::new(index),
            },
            value,
        }
    }
    pub fn block(stmts: Vec<Statement>) -> Self {
        Statement::Block { statements: stmts }
    }
    pub fn if_stmt(condition: Expression, then_block: Statement, else_block: Option<Statement>) -> Self {
        Statement::If {
            condition,
            then_block: Box::new(then_block),
            else_block: else_block.map(Box::new),
        }
    }
    pub fn while_stmt(condition: Expression, body: Statement) -> Self {
        Statement::While {
            condition,
            body: Box::new(body),
            yield_every: None,
        }
    }
    pub fn for_each(variable: impl Into<String>, iterable: Expression, body: Statement) -> Self {
        Statement::ForEach {
            variable: variable.into(),
            iterable,
            body: Box::new(body),
            yield_every: None,
        }
    }
    pub fn ret(value: Option<Expression>) -> Self {
        Statement::Return { value }
    }
    pub fn expr_stmt(expr: Expression) -> Self {
        Statement::ExpressionStmt(expr)
    }
    pub fn function_decl(
        name: impl Into<String>,
        params: Vec<Parameter>,
        body: Statement,
    ) -> Self {
        Statement::FunctionDecl {
            name: name.into(),
            params,
            return_type: None,
            body: Box::new(body),
            visibility: None,
            type_params: vec![],
            is_async: false,
        }
    }
    pub fn throw(value: Expression) -> Self {
        Statement::Throw { value }
    }
    pub fn try_catch(
        body: Statement,
        catches: Vec<CatchClause>,
        finally: Option<Statement>,
    ) -> Self {
        Statement::TryCatch {
            body: Box::new(body),
            catches,
            finally: finally.map(Box::new),
        }
    }
    pub fn with_block(
        resources: Vec<(impl Into<String>, Expression)>,
        body: Statement,
    ) -> Self {
        Statement::WithBlock {
            resources: resources.into_iter().map(|(n, e)| (n.into(), e)).collect(),
            body: Box::new(body),
        }
    }
    pub fn defer(body: Statement) -> Self {
        Statement::Defer {
            body: Box::new(body),
        }
    }
}

impl CatchClause {
    pub fn new(param: impl Into<String>, body: Statement) -> Self {
        CatchClause {
            param: param.into(),
            type_annotation: None,
            body,
        }
    }
    pub fn typed(param: impl Into<String>, ty: TypeAnnotation, body: Statement) -> Self {
        CatchClause {
            param: param.into(),
            type_annotation: Some(ty),
            body,
        }
    }
}

impl Parameter {
    pub fn new(name: impl Into<String>) -> Self {
        Parameter {
            name: name.into(),
            type_annotation: None,
            default_value: None,
        }
    }
    pub fn typed(name: impl Into<String>, ty: TypeAnnotation) -> Self {
        Parameter {
            name: name.into(),
            type_annotation: Some(ty),
            default_value: None,
        }
    }
}

impl Program {
    pub fn new(statements: Vec<Statement>) -> Self {
        Program { statements }
    }

    /// Returns `true` if the AST contains any `Incomplete` node whose kind
    /// is continuable (i.e. the REPL should wait for more input).
    pub fn has_incomplete_continuable(&self) -> bool {
        self.statements.iter().any(|s| s.has_incomplete_continuable())
    }

    /// Returns `true` if the AST contains any `Incomplete` node at all.
    pub fn has_any_incomplete(&self) -> bool {
        self.statements.iter().any(|s| s.has_any_incomplete())
    }
}

impl Statement {
    /// Recursively check for continuable incomplete nodes.
    pub fn has_incomplete_continuable(&self) -> bool {
        match self {
            Statement::Incomplete { kind } => kind.is_continuable(),
            Statement::Block { statements } => {
                statements.iter().any(|s| s.has_incomplete_continuable())
            }
            Statement::If { condition, then_block, else_block } => {
                condition.has_incomplete_continuable()
                    || then_block.has_incomplete_continuable()
                    || else_block.as_ref().map_or(false, |e| e.has_incomplete_continuable())
            }
            Statement::While { condition, body, .. } => {
                condition.has_incomplete_continuable() || body.has_incomplete_continuable()
            }
            Statement::ForEach { iterable, body, .. } => {
                iterable.has_incomplete_continuable() || body.has_incomplete_continuable()
            }
            Statement::VariableDecl { value, .. } => value.has_incomplete_continuable(),
            Statement::Assignment { value, .. } => value.has_incomplete_continuable(),
            Statement::Return { value } => {
                value.as_ref().map_or(false, |v| v.has_incomplete_continuable())
            }
            Statement::ExpressionStmt(expr) => expr.has_incomplete_continuable(),
            Statement::FunctionDecl { body, .. } => body.has_incomplete_continuable(),
            Statement::Throw { value } => value.has_incomplete_continuable(),
            Statement::TryCatch { body, catches, finally } => {
                body.has_incomplete_continuable()
                    || catches.iter().any(|c| c.body.has_incomplete_continuable())
                    || finally.as_ref().map_or(false, |f| f.has_incomplete_continuable())
            }
            Statement::WithBlock { body, .. } => body.has_incomplete_continuable(),
            Statement::Defer { body } => body.has_incomplete_continuable(),
            Statement::Annotated { inner, .. } => inner.has_incomplete_continuable(),
            Statement::Match { subject, arms } => {
                subject.has_incomplete_continuable()
                    || arms.iter().any(|a| a.body.has_incomplete_continuable())
            }
            Statement::DeclareBlock { body } => {
                body.iter().any(|s| s.has_incomplete_continuable())
            }
            Statement::ShellCommand { .. }
            | Statement::TypeAlias { .. }
            | Statement::Use { .. }
            | Statement::Bootstrap { .. }
            | Statement::StandardDef { .. }
            | Statement::EntryTypeDef { .. }
            | Statement::Yield => false,
        }
    }

    /// Recursively check for any incomplete nodes (continuable or not).
    pub fn has_any_incomplete(&self) -> bool {
        match self {
            Statement::Incomplete { .. } => true,
            Statement::Block { statements } => {
                statements.iter().any(|s| s.has_any_incomplete())
            }
            Statement::If { condition, then_block, else_block } => {
                condition.has_any_incomplete()
                    || then_block.has_any_incomplete()
                    || else_block.as_ref().map_or(false, |e| e.has_any_incomplete())
            }
            Statement::While { condition, body, .. } => {
                condition.has_any_incomplete() || body.has_any_incomplete()
            }
            Statement::ForEach { iterable, body, .. } => {
                iterable.has_any_incomplete() || body.has_any_incomplete()
            }
            Statement::VariableDecl { value, .. } => value.has_any_incomplete(),
            Statement::Assignment { value, .. } => value.has_any_incomplete(),
            Statement::Return { value } => {
                value.as_ref().map_or(false, |v| v.has_any_incomplete())
            }
            Statement::ExpressionStmt(expr) => expr.has_any_incomplete(),
            Statement::FunctionDecl { body, .. } => body.has_any_incomplete(),
            Statement::Throw { value } => value.has_any_incomplete(),
            Statement::TryCatch { body, catches, finally } => {
                body.has_any_incomplete()
                    || catches.iter().any(|c| c.body.has_any_incomplete())
                    || finally.as_ref().map_or(false, |f| f.has_any_incomplete())
            }
            Statement::WithBlock { body, .. } => body.has_any_incomplete(),
            Statement::Defer { body } => body.has_any_incomplete(),
            Statement::Annotated { inner, .. } => inner.has_any_incomplete(),
            Statement::Match { subject, arms } => {
                subject.has_any_incomplete()
                    || arms.iter().any(|a| a.body.has_any_incomplete())
            }
            Statement::DeclareBlock { body } => {
                body.iter().any(|s| s.has_any_incomplete())
            }
            Statement::ShellCommand { .. }
            | Statement::TypeAlias { .. }
            | Statement::Use { .. }
            | Statement::Bootstrap { .. }
            | Statement::StandardDef { .. }
            | Statement::EntryTypeDef { .. }
            | Statement::Yield => false,
        }
    }
}

impl Expression {
    /// Recursively check for continuable incomplete nodes.
    pub fn has_incomplete_continuable(&self) -> bool {
        match self {
            Expression::Incomplete { kind } => kind.is_continuable(),
            Expression::BinaryOp { left, right, .. } => {
                left.has_incomplete_continuable() || right.has_incomplete_continuable()
            }
            Expression::UnaryOp { operand, .. } => operand.has_incomplete_continuable(),
            Expression::FunctionCall { callee, args } => {
                callee.has_incomplete_continuable()
                    || args.iter().any(|a| a.has_incomplete_continuable())
            }
            Expression::ObjectLiteral(pairs) => {
                pairs.iter().any(|(_, v)| v.has_incomplete_continuable())
            }
            Expression::ListLiteral(elems) => {
                elems.iter().any(|e| e.has_incomplete_continuable())
            }
            Expression::IndexAccess { object, index } => {
                object.has_incomplete_continuable() || index.has_incomplete_continuable()
            }
            Expression::PropertyAccess { object, .. } => object.has_incomplete_continuable(),
            Expression::Lambda { body, .. } => body.has_incomplete_continuable(),
            Expression::StringInterpolation(parts) => parts.iter().any(|p| match p {
                StringPart::Expr(e) => e.has_incomplete_continuable(),
                _ => false,
            }),
            Expression::CommandSubstitution(cmd) => cmd.has_incomplete_continuable(),
            Expression::Await { expr } => expr.has_incomplete_continuable(),
            Expression::Spawn { callee, args } => {
                callee.has_incomplete_continuable() || args.iter().any(|a| a.has_incomplete_continuable())
            }
            _ => false,
        }
    }

    /// Recursively check for any incomplete nodes.
    pub fn has_any_incomplete(&self) -> bool {
        match self {
            Expression::Incomplete { .. } => true,
            Expression::BinaryOp { left, right, .. } => {
                left.has_any_incomplete() || right.has_any_incomplete()
            }
            Expression::UnaryOp { operand, .. } => operand.has_any_incomplete(),
            Expression::FunctionCall { callee, args } => {
                callee.has_any_incomplete()
                    || args.iter().any(|a| a.has_any_incomplete())
            }
            Expression::ObjectLiteral(pairs) => {
                pairs.iter().any(|(_, v)| v.has_any_incomplete())
            }
            Expression::ListLiteral(elems) => {
                elems.iter().any(|e| e.has_any_incomplete())
            }
            Expression::IndexAccess { object, index } => {
                object.has_any_incomplete() || index.has_any_incomplete()
            }
            Expression::PropertyAccess { object, .. } => object.has_any_incomplete(),
            Expression::Lambda { body, .. } => body.has_any_incomplete(),
            Expression::StringInterpolation(parts) => parts.iter().any(|p| match p {
                StringPart::Expr(e) => e.has_any_incomplete(),
                _ => false,
            }),
            Expression::CommandSubstitution(cmd) => cmd.has_any_incomplete(),
            Expression::Await { expr } => expr.has_any_incomplete(),
            Expression::Spawn { callee, args } => {
                callee.has_any_incomplete() || args.iter().any(|a| a.has_any_incomplete())
            }
            _ => false,
        }
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_literal_construction() {
        assert_eq!(Expression::int(42), Expression::Literal(Literal::Int(42)));
        assert_eq!(Expression::float(3.14), Expression::Literal(Literal::Float(3.14)));
        assert_eq!(Expression::bool(true), Expression::Literal(Literal::Bool(true)));
        assert_eq!(Expression::string("hello"), Expression::Literal(Literal::String("hello".into())));
        assert_eq!(Expression::null(), Expression::Literal(Literal::Null));
    }

    #[test]
    fn test_binary_op() {
        let expr = Expression::binary(
            BinaryOperator::Add,
            Expression::int(1),
            Expression::int(2),
        );
        if let Expression::BinaryOp { op, left, right } = expr {
            assert_eq!(op, BinaryOperator::Add);
            assert_eq!(*left, Expression::int(1));
            assert_eq!(*right, Expression::int(2));
        } else {
            panic!("expected BinaryOp");
        }
    }

    #[test]
    fn test_function_decl() {
        let func = Statement::function_decl(
            "add",
            vec![Parameter::new("a"), Parameter::new("b")],
            Statement::block(vec![
                Statement::ret(Some(Expression::binary(
                    BinaryOperator::Add,
                    Expression::ident("a"),
                    Expression::ident("b"),
                ))),
            ]),
        );
        if let Statement::FunctionDecl { name, params, body, .. } = func {
            assert_eq!(name, "add");
            assert_eq!(params.len(), 2);
            assert!(matches!(*body, Statement::Block { .. }));
        } else {
            panic!("expected FunctionDecl");
        }
    }

    #[test]
    fn test_json_roundtrip() {
        let program = Program::new(vec![
            Statement::var_decl("x", Expression::int(42)),
            Statement::function_decl(
                "double",
                vec![Parameter::new("n")],
                Statement::block(vec![
                    Statement::ret(Some(Expression::binary(
                        BinaryOperator::Mul,
                        Expression::ident("n"),
                        Expression::int(2),
                    ))),
                ]),
            ),
            Statement::expr_stmt(Expression::call(
                Expression::ident("double"),
                vec![Expression::ident("x")],
            )),
        ]);

        let json = serde_json::to_string_pretty(&program).unwrap();
        let parsed: Program = serde_json::from_str(&json).unwrap();
        assert_eq!(program, parsed);
    }

    #[test]
    fn test_all_node_types_roundtrip() {
        // Construct an AST exercising every node type
        let program = Program::new(vec![
            // VariableDecl
            Statement::var_decl("x", Expression::int(10)),
            // FunctionDecl with all expression types
            Statement::function_decl(
                "test_fn",
                vec![Parameter::new("a"), Parameter::new("b")],
                Statement::block(vec![
                    // UnaryOp
                    Statement::var_decl("neg", Expression::unary(UnaryOperator::Negate, Expression::ident("a"))),
                    // BinaryOp
                    Statement::var_decl("sum", Expression::binary(BinaryOperator::Add, Expression::ident("a"), Expression::ident("b"))),
                    // ObjectLiteral
                    Statement::var_decl("obj", Expression::object(vec![
                        ("key", Expression::string("value")),
                    ])),
                    // ListLiteral
                    Statement::var_decl("lst", Expression::list(vec![Expression::int(1), Expression::int(2)])),
                    // PropertyAccess
                    Statement::var_decl("prop", Expression::property(Expression::ident("obj"), "key")),
                    // IndexAccess
                    Statement::var_decl("elem", Expression::index(Expression::ident("lst"), Expression::int(0))),
                    // Lambda
                    Statement::var_decl("f", Expression::lambda(
                        vec![Parameter::new("x")],
                        Statement::block(vec![
                            Statement::ret(Some(Expression::binary(BinaryOperator::Mul, Expression::ident("x"), Expression::int(2)))),
                        ]),
                    )),
                    // FunctionCall
                    Statement::expr_stmt(Expression::call(Expression::ident("f"), vec![Expression::int(5)])),
                    // Assignment
                    Statement::assign("x", Expression::int(20)),
                    // PropertyAssignment
                    Statement::assign_property(Expression::ident("obj"), "key", Expression::string("new_value")),
                    // IndexAssignment
                    Statement::assign_index(Expression::ident("lst"), Expression::int(0), Expression::int(99)),
                    // If
                    Statement::if_stmt(
                        Expression::binary(BinaryOperator::Gt, Expression::ident("a"), Expression::int(0)),
                        Statement::block(vec![Statement::ret(Some(Expression::ident("a")))]),
                        Some(Statement::block(vec![Statement::ret(Some(Expression::unary(UnaryOperator::Negate, Expression::ident("a"))))])),
                    ),
                    // While
                    Statement::while_stmt(
                        Expression::binary(BinaryOperator::Lt, Expression::ident("x"), Expression::int(100)),
                        Statement::block(vec![
                            Statement::assign("x", Expression::binary(BinaryOperator::Add, Expression::ident("x"), Expression::int(1))),
                        ]),
                    ),
                    // ForEach
                    Statement::for_each(
                        "item",
                        Expression::ident("lst"),
                        Statement::block(vec![
                            Statement::expr_stmt(Expression::call(
                                Expression::ident("println"),
                                vec![Expression::ident("item")],
                            )),
                        ]),
                    ),
                    // Return
                    Statement::ret(Some(Expression::null())),
                ]),
            ),
        ]);

        let json = serde_json::to_string(&program).unwrap();
        let parsed: Program = serde_json::from_str(&json).unwrap();
        assert_eq!(program, parsed);
    }

    #[test]
    fn test_visibility_roundtrip() {
        for vis in [Visibility::Priv, Visibility::Pkg, Visibility::Pub] {
            let json = serde_json::to_string(&vis).unwrap();
            let parsed: Visibility = serde_json::from_str(&json).unwrap();
            assert_eq!(vis, parsed);
        }
    }

    #[test]
    fn test_use_plain_roundtrip() {
        let stmt = Statement::Use {
            module_path: vec!["net".into(), "http".into()],
            alias: None,
            selective: None,
        };
        let json = serde_json::to_string(&stmt).unwrap();
        let parsed: Statement = serde_json::from_str(&json).unwrap();
        assert_eq!(stmt, parsed);
    }

    #[test]
    fn test_use_aliased_roundtrip() {
        let stmt = Statement::Use {
            module_path: vec!["net".into(), "http".into()],
            alias: Some("h".into()),
            selective: None,
        };
        let json = serde_json::to_string(&stmt).unwrap();
        let parsed: Statement = serde_json::from_str(&json).unwrap();
        assert_eq!(stmt, parsed);
    }

    #[test]
    fn test_use_selective_roundtrip() {
        let stmt = Statement::Use {
            module_path: vec!["net".into(), "http".into()],
            alias: None,
            selective: Some(vec![
                SelectiveImport { name: "Client".into(), alias: None },
                SelectiveImport { name: "Request".into(), alias: Some("Req".into()) },
            ]),
        };
        let json = serde_json::to_string(&stmt).unwrap();
        let parsed: Statement = serde_json::from_str(&json).unwrap();
        assert_eq!(stmt, parsed);
    }

    #[test]
    fn test_declare_block_construction() {
        let block = Statement::DeclareBlock {
            body: vec![
                Statement::function_decl("even", vec![Parameter::new("n")], Statement::block(vec![])),
                Statement::function_decl("odd", vec![Parameter::new("n")], Statement::block(vec![])),
            ],
        };
        let json = serde_json::to_string(&block).unwrap();
        let parsed: Statement = serde_json::from_str(&json).unwrap();
        assert_eq!(block, parsed);
    }
}
