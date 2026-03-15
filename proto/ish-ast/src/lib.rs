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

// ── Type annotations ────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TypeAnnotation {
    Simple(String),           // e.g. "int", "string", "bool"
    List(Box<TypeAnnotation>),
    Object(Vec<(String, TypeAnnotation)>),
    Function {
        params: Vec<TypeAnnotation>,
        ret: Box<TypeAnnotation>,
    },
    Union(Vec<TypeAnnotation>),
    Tuple(Vec<TypeAnnotation>),
    Generic {
        base: String,                           // e.g. "List"
        type_args: Vec<TypeAnnotation>,         // e.g. [Simple("int")]
    },
}

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
    },
    StringInterpolation(Vec<StringPart>),
    CommandSubstitution(Box<Statement>), // $(...) — wraps a ShellCommand or pipeline
    EnvVar(String), // $HOME or ${PATH}
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
    },
    ForEach {
        variable: String,
        iterable: Expression,
        body: Box<Statement>, // must be a Block
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
        path: Vec<String>,
    },
    ModDecl {
        name: String,
        body: Option<Box<Statement>>, // None = file module, Some = inline block
        visibility: Option<Visibility>,
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
    Private,
    Public,
    PubScope(String), // e.g. pub(super), pub(global)
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
        }
    }
    pub fn for_each(variable: impl Into<String>, iterable: Expression, body: Statement) -> Self {
        Statement::ForEach {
            variable: variable.into(),
            iterable,
            body: Box::new(body),
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
}
