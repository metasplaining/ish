// Display implementations for AST nodes — produce human-readable pseudo-code.

use crate::*;
use std::fmt;

impl fmt::Display for Program {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (i, stmt) in self.statements.iter().enumerate() {
            if i > 0 {
                writeln!(f)?;
            }
            write!(f, "{}", StmtDisplay(stmt, 0))?;
        }
        Ok(())
    }
}

struct StmtDisplay<'a>(&'a Statement, usize);
struct ExprDisplay<'a>(&'a Expression);

fn indent(f: &mut fmt::Formatter<'_>, level: usize) -> fmt::Result {
    for _ in 0..level {
        write!(f, "    ")?;
    }
    Ok(())
}

impl<'a> fmt::Display for StmtDisplay<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let StmtDisplay(stmt, depth) = self;
        let d = *depth;
        match stmt {
            Statement::VariableDecl { name, value, .. } => {
                indent(f, d)?;
                write!(f, "let {} = {};", name, ExprDisplay(value))
            }
            Statement::Assignment { target, value } => {
                indent(f, d)?;
                match target {
                    AssignTarget::Variable(name) => write!(f, "{}", name)?,
                    AssignTarget::Property { object, property } => {
                        write!(f, "{}.{}", ExprDisplay(object), property)?
                    }
                    AssignTarget::Index { object, index } => {
                        write!(f, "{}[{}]", ExprDisplay(object), ExprDisplay(index))?
                    }
                }
                write!(f, " = {};", ExprDisplay(value))
            }
            Statement::Block { statements } => {
                writeln!(f, "{{")?;
                for s in statements {
                    writeln!(f, "{}", StmtDisplay(s, d + 1))?;
                }
                indent(f, d)?;
                write!(f, "}}")
            }
            Statement::If { condition, then_block, else_block } => {
                indent(f, d)?;
                write!(f, "if {} ", ExprDisplay(condition))?;
                write!(f, "{}", StmtDisplay(then_block, d))?;
                if let Some(eb) = else_block {
                    write!(f, " else {}", StmtDisplay(eb, d))?;
                }
                Ok(())
            }
            Statement::While { condition, body } => {
                indent(f, d)?;
                write!(f, "while {} ", ExprDisplay(condition))?;
                write!(f, "{}", StmtDisplay(body, d))
            }
            Statement::ForEach { variable, iterable, body } => {
                indent(f, d)?;
                write!(f, "for {} in {} ", variable, ExprDisplay(iterable))?;
                write!(f, "{}", StmtDisplay(body, d))
            }
            Statement::Return { value: Some(expr) } => {
                indent(f, d)?;
                write!(f, "return {};", ExprDisplay(expr))
            }
            Statement::Return { value: None } => {
                indent(f, d)?;
                write!(f, "return;")
            }
            Statement::ExpressionStmt(expr) => {
                indent(f, d)?;
                write!(f, "{};", ExprDisplay(expr))
            }
            Statement::FunctionDecl { name, params, body, .. } => {
                indent(f, d)?;
                write!(f, "fn {}(", name)?;
                for (i, p) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", p.name)?;
                }
                write!(f, ") ")?;
                write!(f, "{}", StmtDisplay(body, d))
            }
        }
    }
}

impl<'a> fmt::Display for ExprDisplay<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            Expression::Literal(lit) => match lit {
                Literal::Bool(b) => write!(f, "{}", b),
                Literal::Int(n) => write!(f, "{}", n),
                Literal::Float(n) => write!(f, "{}", n),
                Literal::String(s) => write!(f, "\"{}\"", s),
                Literal::Null => write!(f, "null"),
            },
            Expression::Identifier(name) => write!(f, "{}", name),
            Expression::BinaryOp { op, left, right } => {
                write!(f, "({} {} {})", ExprDisplay(left), op_str(op), ExprDisplay(right))
            }
            Expression::UnaryOp { op, operand } => {
                let op_s = match op {
                    UnaryOperator::Not => "!",
                    UnaryOperator::Negate => "-",
                };
                write!(f, "{}{}", op_s, ExprDisplay(operand))
            }
            Expression::FunctionCall { callee, args } => {
                write!(f, "{}(", ExprDisplay(callee))?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", ExprDisplay(arg))?;
                }
                write!(f, ")")
            }
            Expression::ObjectLiteral(pairs) => {
                write!(f, "{{ ")?;
                for (i, (k, v)) in pairs.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", k, ExprDisplay(v))?;
                }
                write!(f, " }}")
            }
            Expression::ListLiteral(elems) => {
                write!(f, "[")?;
                for (i, e) in elems.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", ExprDisplay(e))?;
                }
                write!(f, "]")
            }
            Expression::PropertyAccess { object, property } => {
                write!(f, "{}.{}", ExprDisplay(object), property)
            }
            Expression::IndexAccess { object, index } => {
                write!(f, "{}[{}]", ExprDisplay(object), ExprDisplay(index))
            }
            Expression::Lambda { params, body } => {
                write!(f, "fn(")?;
                for (i, p) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", p.name)?;
                }
                write!(f, ") ")?;
                write!(f, "{}", StmtDisplay(body, 0))
            }
        }
    }
}

fn op_str(op: &BinaryOperator) -> &'static str {
    match op {
        BinaryOperator::Add => "+",
        BinaryOperator::Sub => "-",
        BinaryOperator::Mul => "*",
        BinaryOperator::Div => "/",
        BinaryOperator::Mod => "%",
        BinaryOperator::Eq => "==",
        BinaryOperator::NotEq => "!=",
        BinaryOperator::Lt => "<",
        BinaryOperator::Gt => ">",
        BinaryOperator::LtEq => "<=",
        BinaryOperator::GtEq => ">=",
        BinaryOperator::And => "&&",
        BinaryOperator::Or => "||",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builder::ProgramBuilder;

    #[test]
    fn test_display_factorial() {
        let program = ProgramBuilder::new()
            .function("factorial", &["n"], |b| {
                b.if_else(
                    Expression::binary(BinaryOperator::LtEq, Expression::ident("n"), Expression::int(1)),
                    |b| b.ret(Expression::int(1)),
                    |b| b.ret(Expression::binary(
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
                    )),
                )
            })
            .build();

        let display = format!("{}", program);
        assert!(display.contains("fn factorial(n)"));
        assert!(display.contains("return"));
    }
}
