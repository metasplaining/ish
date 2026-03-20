// ish-vm/src/ledger/narrowing.rs — Type narrowing condition analysis.
//
// Analyzes if-condition expressions to detect narrowing opportunities:
// - is_type(x, T)  → narrows x to T in true branch
// - x != null      → excludes null from x in true branch
// - x == null      → excludes null from x in false (else) branch

use ish_ast::{Expression, BinaryOperator};

/// A narrowing fact extracted from a condition expression.
#[derive(Debug, Clone, PartialEq)]
pub enum NarrowingFact {
    /// `is_type(variable, type_name)` — variable is narrowed to type_name in true branch.
    IsType { variable: String, type_name: String },
    /// `variable != null` — null is excluded from variable in true branch.
    NotNull { variable: String },
    /// `variable == null` — null is excluded from variable in false (else) branch.
    IsNull { variable: String },
}

/// Analyze a condition expression and extract narrowing facts.
///
/// Returns a list of narrowing facts that can be applied to the true branch.
/// Callers should invert the facts for the else branch (e.g., IsNull becomes
/// NotNull in the else branch).
pub fn analyze_condition(expr: &Expression) -> Vec<NarrowingFact> {
    let mut facts = Vec::new();
    analyze_expr(expr, &mut facts);
    facts
}

fn analyze_expr(expr: &Expression, facts: &mut Vec<NarrowingFact>) {
    match expr {
        // is_type(x, "T") — function call with identifier and string literal
        Expression::FunctionCall { callee, args } => {
            if let Expression::Identifier(name) = callee.as_ref() {
                if name == "is_type" && args.len() == 2 {
                    if let (Expression::Identifier(var), Some(type_name)) =
                        (&args[0], extract_string_literal(&args[1]))
                    {
                        facts.push(NarrowingFact::IsType {
                            variable: var.clone(),
                            type_name,
                        });
                    }
                }
            }
        }

        // x != null → NotNull
        Expression::BinaryOp { left, op: BinaryOperator::NotEq, right } => {
            if let Some(var) = extract_null_comparison(left, right) {
                facts.push(NarrowingFact::NotNull { variable: var });
            }
        }

        // x == null → IsNull (narrows in else branch, not true branch)
        Expression::BinaryOp { left, op: BinaryOperator::Eq, right } => {
            if let Some(var) = extract_null_comparison(left, right) {
                facts.push(NarrowingFact::IsNull { variable: var });
            }
        }

        _ => {}
    }
}

/// Extract a string from a string literal expression.
fn extract_string_literal(expr: &Expression) -> Option<String> {
    match expr {
        Expression::Literal(ish_ast::Literal::String(s)) => Some(s.clone()),
        _ => None,
    }
}

/// Check if binary op is a null comparison (x op null or null op x).
/// Returns the variable name if one side is an identifier and the other is null.
fn extract_null_comparison(left: &Expression, right: &Expression) -> Option<String> {
    match (left, right) {
        (Expression::Identifier(var), Expression::Literal(ish_ast::Literal::Null)) => {
            Some(var.clone())
        }
        (Expression::Literal(ish_ast::Literal::Null), Expression::Identifier(var)) => {
            Some(var.clone())
        }
        _ => None,
    }
}

/// Invert narrowing facts for the else branch.
/// - IsType has no useful inversion (we don't narrow in else for is_type)
/// - NotNull → (no narrowing in else)
/// - IsNull → NotNull (in else branch, we know it's not null)
pub fn invert_for_else(facts: &[NarrowingFact]) -> Vec<NarrowingFact> {
    let mut inverted = Vec::new();
    for fact in facts {
        match fact {
            NarrowingFact::IsNull { variable } => {
                inverted.push(NarrowingFact::NotNull {
                    variable: variable.clone(),
                });
            }
            // IsType and NotNull have no useful else-branch narrowing
            _ => {}
        }
    }
    inverted
}

#[cfg(test)]
mod tests {
    use super::*;
    use ish_ast::{Expression, Literal, BinaryOperator};

    #[test]
    fn detect_is_type() {
        let expr = Expression::FunctionCall {
            callee: Box::new(Expression::Identifier("is_type".into())),
            args: vec![
                Expression::Identifier("x".into()),
                Expression::Literal(Literal::String("i32".into())),
            ],
        };
        let facts = analyze_condition(&expr);
        assert_eq!(facts, vec![NarrowingFact::IsType {
            variable: "x".into(),
            type_name: "i32".into(),
        }]);
    }

    #[test]
    fn detect_not_null() {
        let expr = Expression::BinaryOp {
            left: Box::new(Expression::Identifier("x".into())),
            op: BinaryOperator::NotEq,
            right: Box::new(Expression::Literal(Literal::Null)),
        };
        let facts = analyze_condition(&expr);
        assert_eq!(facts, vec![NarrowingFact::NotNull { variable: "x".into() }]);
    }

    #[test]
    fn detect_is_null_inverts() {
        let expr = Expression::BinaryOp {
            left: Box::new(Expression::Identifier("x".into())),
            op: BinaryOperator::Eq,
            right: Box::new(Expression::Literal(Literal::Null)),
        };
        let facts = analyze_condition(&expr);
        assert_eq!(facts, vec![NarrowingFact::IsNull { variable: "x".into() }]);

        let else_facts = invert_for_else(&facts);
        assert_eq!(else_facts, vec![NarrowingFact::NotNull { variable: "x".into() }]);
    }

    #[test]
    fn null_on_left_side() {
        let expr = Expression::BinaryOp {
            left: Box::new(Expression::Literal(Literal::Null)),
            op: BinaryOperator::NotEq,
            right: Box::new(Expression::Identifier("y".into())),
        };
        let facts = analyze_condition(&expr);
        assert_eq!(facts, vec![NarrowingFact::NotNull { variable: "y".into() }]);
    }

    #[test]
    fn no_narrowing_for_general_expression() {
        let expr = Expression::BinaryOp {
            left: Box::new(Expression::Identifier("x".into())),
            op: BinaryOperator::Gt,
            right: Box::new(Expression::Literal(Literal::Int(0))),
        };
        let facts = analyze_condition(&expr);
        assert!(facts.is_empty());
    }
}
