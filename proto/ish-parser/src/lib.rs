pub mod ast_builder;
pub mod error;

use pest::Parser;
use pest_derive::Parser;

use ish_ast::{Program, Statement, IncompleteKind};
use error::ParseError;

#[derive(Parser)]
#[grammar = "ish.pest"]
struct IshParser;

/// Parse ish source text into an AST Program.
///
/// Parser-matches-everything philosophy: always returns `Ok(Program)`.
/// If the input is incomplete or malformed, the AST contains `Incomplete`
/// nodes rather than returning parse errors.
pub fn parse(input: &str) -> Result<Program, Vec<ParseError>> {
    let pairs = match IshParser::parse(Rule::program, input) {
        Ok(pairs) => pairs,
        Err(_) => {
            // Parser failed — wrap as an incomplete block statement.
            // This makes the parser always succeed from the caller's perspective.
            return Ok(Program {
                statements: vec![Statement::Incomplete {
                    kind: IncompleteKind::Block,
                }],
            });
        }
    };

    match ast_builder::build_program(pairs) {
        Ok(program) => Ok(program),
        Err(_) => {
            // AST builder failed — wrap as incomplete
            Ok(Program {
                statements: vec![Statement::Incomplete {
                    kind: IncompleteKind::Block,
                }],
            })
        }
    }
}
