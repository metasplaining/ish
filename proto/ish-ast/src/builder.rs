// Fluent builder API for constructing ish AST programs.
//
// Example usage:
//   ProgramBuilder::new()
//       .var_decl("x", Expression::int(42))
//       .function("factorial", &["n"], |b| {
//           b.if_else(
//               Expression::binary(BinaryOperator::LtEq, Expression::ident("n"), Expression::int(1)),
//               |b| b.ret(Expression::int(1)),
//               |b| b.ret(Expression::binary(
//                   BinaryOperator::Mul,
//                   Expression::ident("n"),
//                   Expression::call(Expression::ident("factorial"), vec![
//                       Expression::binary(BinaryOperator::Sub, Expression::ident("n"), Expression::int(1)),
//                   ]),
//               )),
//           )
//       })
//       .build()

use crate::*;

/// Builder for constructing a Program (top-level list of statements).
pub struct ProgramBuilder {
    statements: Vec<Statement>,
}

impl ProgramBuilder {
    pub fn new() -> Self {
        ProgramBuilder { statements: vec![] }
    }

    pub fn stmt(mut self, stmt: Statement) -> Self {
        self.statements.push(stmt);
        self
    }

    pub fn var_decl(self, name: impl Into<String>, value: Expression) -> Self {
        self.stmt(Statement::var_decl(name, value))
    }

    pub fn function(
        self,
        name: impl Into<String>,
        params: &[&str],
        body_fn: impl FnOnce(BlockBuilder) -> BlockBuilder,
    ) -> Self {
        let params: Vec<Parameter> = params.iter().map(|p| Parameter::new(*p)).collect();
        let block = body_fn(BlockBuilder::new()).build();
        self.stmt(Statement::FunctionDecl {
            name: name.into(),
            params,
            return_type: None,
            body: Box::new(block),
            visibility: None,
            type_params: vec![],
        })
    }

    pub fn expr_stmt(self, expr: Expression) -> Self {
        self.stmt(Statement::expr_stmt(expr))
    }

    pub fn build(self) -> Program {
        Program::new(self.statements)
    }
}

impl Default for ProgramBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for constructing a Block statement (sequence of statements).
pub struct BlockBuilder {
    statements: Vec<Statement>,
}

impl BlockBuilder {
    pub fn new() -> Self {
        BlockBuilder { statements: vec![] }
    }

    pub fn stmt(mut self, stmt: Statement) -> Self {
        self.statements.push(stmt);
        self
    }

    pub fn var_decl(self, name: impl Into<String>, value: Expression) -> Self {
        self.stmt(Statement::var_decl(name, value))
    }

    pub fn assign(self, name: impl Into<String>, value: Expression) -> Self {
        self.stmt(Statement::assign(name, value))
    }

    pub fn ret(self, value: Expression) -> Self {
        self.stmt(Statement::ret(Some(value)))
    }

    pub fn ret_void(self) -> Self {
        self.stmt(Statement::ret(None))
    }

    pub fn expr_stmt(self, expr: Expression) -> Self {
        self.stmt(Statement::expr_stmt(expr))
    }

    pub fn if_then(
        self,
        condition: Expression,
        then_fn: impl FnOnce(BlockBuilder) -> BlockBuilder,
    ) -> Self {
        let then_block = then_fn(BlockBuilder::new()).build();
        self.stmt(Statement::If {
            condition,
            then_block: Box::new(then_block),
            else_block: None,
        })
    }

    pub fn if_else(
        self,
        condition: Expression,
        then_fn: impl FnOnce(BlockBuilder) -> BlockBuilder,
        else_fn: impl FnOnce(BlockBuilder) -> BlockBuilder,
    ) -> Self {
        let then_block = then_fn(BlockBuilder::new()).build();
        let else_block = else_fn(BlockBuilder::new()).build();
        self.stmt(Statement::If {
            condition,
            then_block: Box::new(then_block),
            else_block: Some(Box::new(else_block)),
        })
    }

    pub fn while_loop(
        self,
        condition: Expression,
        body_fn: impl FnOnce(BlockBuilder) -> BlockBuilder,
    ) -> Self {
        let body = body_fn(BlockBuilder::new()).build();
        self.stmt(Statement::While {
            condition,
            body: Box::new(body),
        })
    }

    pub fn for_each(
        self,
        variable: impl Into<String>,
        iterable: Expression,
        body_fn: impl FnOnce(BlockBuilder) -> BlockBuilder,
    ) -> Self {
        let body = body_fn(BlockBuilder::new()).build();
        self.stmt(Statement::ForEach {
            variable: variable.into(),
            iterable,
            body: Box::new(body),
        })
    }

    pub fn function(
        self,
        name: impl Into<String>,
        params: &[&str],
        body_fn: impl FnOnce(BlockBuilder) -> BlockBuilder,
    ) -> Self {
        let params: Vec<Parameter> = params.iter().map(|p| Parameter::new(*p)).collect();
        let block = body_fn(BlockBuilder::new()).build();
        self.stmt(Statement::FunctionDecl {
            name: name.into(),
            params,
            return_type: None,
            body: Box::new(block),
            visibility: None,
            type_params: vec![],
        })
    }

    pub fn build(self) -> Statement {
        Statement::Block {
            statements: self.statements,
        }
    }

    pub fn throw(self, value: Expression) -> Self {
        self.stmt(Statement::throw(value))
    }

    pub fn try_catch(
        self,
        body_fn: impl FnOnce(BlockBuilder) -> BlockBuilder,
        catches: Vec<CatchClause>,
        finally_fn: Option<impl FnOnce(BlockBuilder) -> BlockBuilder>,
    ) -> Self {
        let body = body_fn(BlockBuilder::new()).build();
        let finally = finally_fn.map(|f| f(BlockBuilder::new()).build());
        self.stmt(Statement::try_catch(body, catches, finally))
    }

    pub fn defer(self, stmt: Statement) -> Self {
        self.stmt(Statement::defer(stmt))
    }
}

impl Default for BlockBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_program_builder_factorial() {
        let program = ProgramBuilder::new()
            .function("factorial", &["n"], |b| {
                b.if_else(
                    Expression::binary(
                        BinaryOperator::LtEq,
                        Expression::ident("n"),
                        Expression::int(1),
                    ),
                    |b| b.ret(Expression::int(1)),
                    |b| {
                        b.ret(Expression::binary(
                            BinaryOperator::Mul,
                            Expression::ident("n"),
                            Expression::call(
                                Expression::ident("factorial"),
                                vec![Expression::binary(
                                    BinaryOperator::Sub,
                                    Expression::ident("n"),
                                    Expression::int(1),
                                )],
                            ),
                        ))
                    },
                )
            })
            .build();

        assert_eq!(program.statements.len(), 1);
        if let Statement::FunctionDecl { name, params, .. } = &program.statements[0] {
            assert_eq!(name, "factorial");
            assert_eq!(params.len(), 1);
        } else {
            panic!("expected FunctionDecl");
        }
    }

    #[test]
    fn test_block_builder_nested() {
        let program = ProgramBuilder::new()
            .var_decl("sum", Expression::int(0))
            .function("accumulate", &["items"], |b| {
                b.for_each("item", Expression::ident("items"), |b| {
                    b.assign(
                        "sum",
                        Expression::binary(
                            BinaryOperator::Add,
                            Expression::ident("sum"),
                            Expression::ident("item"),
                        ),
                    )
                })
                .ret(Expression::ident("sum"))
            })
            .build();

        assert_eq!(program.statements.len(), 2);
    }
}
