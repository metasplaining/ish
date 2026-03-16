use reedline::{ValidationResult, Validator};

pub struct IshValidator;

impl Validator for IshValidator {
    fn validate(&self, line: &str) -> ValidationResult {
        let program = ish_parser::parse(line).unwrap_or_else(|_| ish_ast::Program {
            statements: vec![],
        });
        if program.has_incomplete_continuable() {
            ValidationResult::Incomplete
        } else {
            ValidationResult::Complete
        }
    }
}
