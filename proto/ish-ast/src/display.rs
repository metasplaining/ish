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
            Statement::While { condition, body, .. } => {
                indent(f, d)?;
                write!(f, "while {} ", ExprDisplay(condition))?;
                write!(f, "{}", StmtDisplay(body, d))
            }
            Statement::ForEach { variable, iterable, body, .. } => {
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
            Statement::FunctionDecl { name, params, body, is_async, .. } => {
                indent(f, d)?;
                if *is_async {
                    write!(f, "async fn {}(", name)?;
                } else {
                    write!(f, "fn {}(", name)?;
                }
                for (i, p) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", p.name)?;
                }
                write!(f, ") ")?;
                write!(f, "{}", StmtDisplay(body, d))
            }
            Statement::Throw { value } => {
                indent(f, d)?;
                write!(f, "throw {};", ExprDisplay(value))
            }
            Statement::TryCatch { body, catches, finally } => {
                indent(f, d)?;
                write!(f, "try {}", StmtDisplay(body, d))?;
                for clause in catches {
                    write!(f, " catch ({}) {}", clause.param, StmtDisplay(&clause.body, d))?;
                }
                if let Some(fin) = finally {
                    write!(f, " finally {}", StmtDisplay(fin, d))?;
                }
                Ok(())
            }
            Statement::WithBlock { resources, body } => {
                indent(f, d)?;
                write!(f, "with (")?;
                for (i, (name, expr)) in resources.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{} = {}", name, ExprDisplay(expr))?;
                }
                write!(f, ") ")?;
                write!(f, "{}", StmtDisplay(body, d))
            }
            Statement::Defer { body } => {
                indent(f, d)?;
                write!(f, "defer {}", StmtDisplay(body, d))
            }
            Statement::TypeAlias { name, .. } => {
                indent(f, d)?;
                write!(f, "type {} = ...", name)
            }
            Statement::Use { path } => {
                indent(f, d)?;
                write!(f, "use {}", path.join("::"))
            }
            Statement::ModDecl { name, body, .. } => {
                indent(f, d)?;
                if let Some(b) = body {
                    write!(f, "mod {} {}", name, StmtDisplay(b, d))
                } else {
                    write!(f, "mod {}", name)
                }
            }
            Statement::ShellCommand { command, args, background, .. } => {
                indent(f, d)?;
                write!(f, "{}", command)?;
                for arg in args {
                    match arg {
                        ShellArg::Bare(s) | ShellArg::Glob(s) => write!(f, " {}", s)?,
                        ShellArg::Quoted(s) => write!(f, " \"{}\"", s)?,
                        ShellArg::EnvVar(s) => write!(f, " ${}", s)?,
                        ShellArg::CommandSub(cmd) => write!(f, " $({})", StmtDisplay(cmd, 0))?,
                    }
                }
                if *background { write!(f, " &")?; }
                Ok(())
            }
            Statement::Annotated { annotations, inner } => {
                for ann in annotations {
                    indent(f, d)?;
                    match ann {
                        Annotation::Standard(name) => writeln!(f, "@standard[{}]", name)?,
                        Annotation::Entry(items) => {
                            write!(f, "@[")?;
                            for (i, item) in items.iter().enumerate() {
                                if i > 0 { write!(f, ", ")?; }
                                write!(f, "{}", item.name)?;
                                if let Some(v) = &item.value {
                                    write!(f, "({})", v)?;
                                }
                            }
                            writeln!(f, "]")?;
                        }
                    }
                }
                write!(f, "{}", StmtDisplay(inner, d))
            }
            Statement::StandardDef { name, extends, features } => {
                indent(f, d)?;
                write!(f, "standard {}", name)?;
                if let Some(base) = extends {
                    write!(f, " extends {}", base)?;
                }
                write!(f, " [")?;
                for (i, feat) in features.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", feat.name)?;
                    if !feat.params.is_empty() {
                        write!(f, "({})", feat.params.join(", "))?;
                    }
                }
                write!(f, "]")
            }
            Statement::EntryTypeDef { name, .. } => {
                indent(f, d)?;
                write!(f, "entry type {}", name)
            }
            Statement::Match { subject, arms } => {
                indent(f, d)?;
                writeln!(f, "match {} {{", ExprDisplay(subject))?;
                for arm in arms {
                    indent(f, d + 1)?;
                    let pat = match &arm.pattern {
                        MatchPattern::Literal(lit) => match lit {
                            Literal::Bool(b) => format!("{}", b),
                            Literal::Int(n) => format!("{}", n),
                            Literal::Float(n) => format!("{}", n),
                            Literal::String(s) => format!("'{}'", s),
                            Literal::Char(c) => format!("c'{}'", c),
                            Literal::Null => "null".to_string(),
                        },
                        MatchPattern::Identifier(name) => name.clone(),
                        MatchPattern::Wildcard => "_".to_string(),
                    };
                    writeln!(f, "{} => {}", pat, StmtDisplay(&arm.body, d + 2))?;
                }
                indent(f, d)?;
                write!(f, "}}")
            }
            Statement::Incomplete { kind } => {
                indent(f, d)?;
                write!(f, "<incomplete: {:?}>", kind)
            }
            Statement::Yield => {
                indent(f, d)?;
                write!(f, "yield;")
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
                Literal::String(s) => write!(f, "'{}'", s),
                Literal::Char(c) => write!(f, "c'{}'", c),
                Literal::Null => write!(f, "null"),
            },
            Expression::Identifier(name) => write!(f, "{}", name),
            Expression::BinaryOp { op, left, right } => {
                write!(f, "({} {} {})", ExprDisplay(left), op_str(op), ExprDisplay(right))
            }
            Expression::UnaryOp { op, operand } => {
                match op {
                    UnaryOperator::Try => write!(f, "{}?", ExprDisplay(operand)),
                    _ => {
                        let op_s = match op {
                            UnaryOperator::Not => "!",
                            UnaryOperator::Negate => "-",
                            UnaryOperator::Try => unreachable!(),
                        };
                        write!(f, "{}{}", op_s, ExprDisplay(operand))
                    }
                }
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
            Expression::Lambda { params, body, is_async } => {
                if *is_async {
                    write!(f, "async fn(")?;
                } else {
                    write!(f, "fn(")?;
                }
                for (i, p) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", p.name)?;
                }
                write!(f, ") ")?;
                write!(f, "{}", StmtDisplay(body, 0))
            }
            Expression::StringInterpolation(parts) => {
                write!(f, "\"")?;
                for part in parts {
                    match part {
                        StringPart::Text(text) => write!(f, "{}", text)?,
                        StringPart::Expr(Expression::EnvVar(name)) => write!(f, "${}", name)?,
                        StringPart::Expr(expr) => write!(f, "{{{}}}", ExprDisplay(expr))?,
                    }
                }
                write!(f, "\"")
            }
            Expression::CommandSubstitution(cmd) => {
                write!(f, "$({})", StmtDisplay(cmd, 0))
            }
            Expression::EnvVar(name) => {
                write!(f, "${}", name)
            }
            Expression::Incomplete { kind } => {
                write!(f, "<incomplete: {:?}>", kind)
            }
            Expression::Await { callee, args } => {
                write!(f, "await {}({})", ExprDisplay(callee),
                    args.iter().map(|a| format!("{}", ExprDisplay(a))).collect::<Vec<_>>().join(", "))
            }
            Expression::Spawn { callee, args } => {
                write!(f, "spawn {}({})", ExprDisplay(callee),
                    args.iter().map(|a| format!("{}", ExprDisplay(a))).collect::<Vec<_>>().join(", "))
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
