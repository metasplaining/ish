use reedline::{Highlighter, StyledText};
use nu_ansi_term::{Color, Style};

const KEYWORDS: &[&str] = &[
    "let", "mut", "fn", "if", "else", "while", "for", "in", "return",
    "throw", "try", "catch", "finally", "with", "defer", "match",
    "use", "mod", "pub", "type", "and", "or", "not", "entry", "standard",
];
const LITERALS: &[&str] = &["true", "false", "null"];

pub struct IshHighlighter;

impl Highlighter for IshHighlighter {
    fn highlight(&self, line: &str, _cursor: usize) -> StyledText {
        let mut styled = StyledText::new();
        let chars: Vec<char> = line.chars().collect();
        let len = chars.len();
        let mut i = 0;

        while i < len {
            let ch = chars[i];

            // Line comments: // or #
            if ch == '/' && i + 1 < len && chars[i + 1] == '/' {
                let rest: String = chars[i..].iter().collect();
                styled.push((Style::new().fg(Color::DarkGray), rest));
                break;
            }
            if ch == '#' {
                let rest: String = chars[i..].iter().collect();
                styled.push((Style::new().fg(Color::DarkGray), rest));
                break;
            }

            // Double-quoted strings
            if ch == '"' {
                let start = i;
                i += 1;
                while i < len && chars[i] != '"' {
                    if chars[i] == '\\' && i + 1 < len {
                        i += 1;
                    }
                    i += 1;
                }
                if i < len {
                    i += 1; // closing quote
                }
                let s: String = chars[start..i].iter().collect();
                styled.push((Style::new().fg(Color::Green), s));
                continue;
            }

            // Single-quoted strings
            if ch == '\'' {
                let start = i;
                i += 1;
                while i < len && chars[i] != '\'' {
                    if chars[i] == '\\' && i + 1 < len {
                        i += 1;
                    }
                    i += 1;
                }
                if i < len {
                    i += 1;
                }
                let s: String = chars[start..i].iter().collect();
                styled.push((Style::new().fg(Color::Green), s));
                continue;
            }

            // Numbers
            if ch.is_ascii_digit() {
                let start = i;
                while i < len && (chars[i].is_ascii_digit() || chars[i] == '.') {
                    i += 1;
                }
                let s: String = chars[start..i].iter().collect();
                styled.push((Style::new().fg(Color::Cyan), s));
                continue;
            }

            // Identifiers and keywords
            if ch.is_ascii_alphabetic() || ch == '_' {
                let start = i;
                while i < len && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
                    i += 1;
                }
                let word: String = chars[start..i].iter().collect();
                let style = if KEYWORDS.contains(&word.as_str()) {
                    Style::new().bold().fg(Color::Blue)
                } else if LITERALS.contains(&word.as_str()) {
                    Style::new().bold().fg(Color::Cyan)
                } else {
                    Style::default()
                };
                styled.push((style, word));
                continue;
            }

            // Operators
            if "+-*/%=<>!&|".contains(ch) {
                let start = i;
                // Consume multi-char operators like ==, !=, <=, >=
                i += 1;
                if i < len && "=>&|".contains(chars[i]) {
                    i += 1;
                }
                let s: String = chars[start..i].iter().collect();
                styled.push((Style::new().fg(Color::Yellow), s));
                continue;
            }

            // Everything else (whitespace, brackets, etc.)
            styled.push((Style::default(), ch.to_string()));
            i += 1;
        }

        styled
    }
}
