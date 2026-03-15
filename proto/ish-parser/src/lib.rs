pub mod ast_builder;
pub mod error;

use pest::Parser;
use pest_derive::Parser;

use ish_ast::Program;
use error::ParseError;

#[derive(Parser)]
#[grammar = "ish.pest"]
struct IshParser;

/// Parse ish source text into an AST Program.
pub fn parse(input: &str) -> Result<Program, Vec<ParseError>> {
    let pairs = IshParser::parse(Rule::program, input).map_err(|e| {
        vec![ParseError::new(0, input.len(), e.to_string())]
    })?;

    ast_builder::build_program(pairs)
}
