//! Module for tokenizing JMESPath expressions.
//!
//! The lexer borrows identifier strings directly from the expression where
//! possible, avoiding allocations for the most common token types.

use std::iter::Peekable;
use std::str::CharIndices;

use serde_json::Value;

use self::Token::*;
use crate::{ErrorReason, JmespathError};

/// Represents a lexical token of a JMESPath expression.
#[derive(Clone, PartialEq, Debug)]
pub enum Token<'a> {
    Identifier(&'a str),
    QuotedIdentifier(String),
    Number(i32),
    Literal(Value),
    Dot,
    Star,
    Flatten,
    And,
    Or,
    Pipe,
    Filter,
    Lbracket,
    Rbracket,
    Comma,
    Colon,
    Not,
    Ne,
    Eq,
    Gt,
    Gte,
    Lt,
    Lte,
    At,
    Ampersand,
    Lparen,
    Rparen,
    Lbrace,
    Rbrace,
    Eof,
    /// Assignment operator for let bindings (JEP-18): =
    #[cfg(feature = "let-expr")]
    Assign,
    /// Variable reference (JEP-18): $name
    #[cfg(feature = "let-expr")]
    Variable(&'a str),
}

impl Token<'_> {
    /// Provides the left binding power of the token.
    #[inline]
    pub fn lbp(&self) -> usize {
        match *self {
            Pipe => 1,
            Or => 2,
            And => 3,
            Eq => 5,
            Gt => 5,
            Lt => 5,
            Gte => 5,
            Lte => 5,
            Ne => 5,
            Flatten => 9,
            Star => 20,
            Filter => 21,
            Dot => 40,
            Not => 45,
            Lbrace => 50,
            Lbracket => 55,
            Lparen => 60,
            _ => 0,
        }
    }
}

/// A tuple of the token position and the token.
pub type TokenTuple<'a> = (usize, Token<'a>);

/// Tokenizes a JMESPath expression.
pub fn tokenize<'a>(expr: &'a str) -> Result<Vec<TokenTuple<'a>>, JmespathError> {
    Lexer::new(expr).tokenize()
}

struct Lexer<'a> {
    iter: Peekable<CharIndices<'a>>,
    expr: &'a str,
}

impl<'a> Lexer<'a> {
    fn new(expr: &'a str) -> Lexer<'a> {
        Lexer {
            iter: expr.char_indices().peekable(),
            expr,
        }
    }

    fn tokenize(&mut self) -> Result<Vec<TokenTuple<'a>>, JmespathError> {
        let mut tokens = Vec::new();
        let last_position = self.expr.len();
        loop {
            match self.iter.next() {
                Some((pos, ch)) => {
                    match ch {
                        'a'..='z' | 'A'..='Z' | '_' => {
                            tokens.push((pos, self.consume_identifier(pos)))
                        }
                        '.' => tokens.push((pos, Dot)),
                        '[' => tokens.push((pos, self.consume_lbracket())),
                        '*' => tokens.push((pos, Star)),
                        '|' => tokens.push((pos, self.alt('|', Or, Pipe))),
                        '@' => tokens.push((pos, At)),
                        ']' => tokens.push((pos, Rbracket)),
                        '{' => tokens.push((pos, Lbrace)),
                        '}' => tokens.push((pos, Rbrace)),
                        '&' => tokens.push((pos, self.alt('&', And, Ampersand))),
                        '(' => tokens.push((pos, Lparen)),
                        ')' => tokens.push((pos, Rparen)),
                        ',' => tokens.push((pos, Comma)),
                        ':' => tokens.push((pos, Colon)),
                        '"' => tokens.push((pos, self.consume_quoted_identifier(pos)?)),
                        '\'' => tokens.push((pos, self.consume_raw_string(pos)?)),
                        '`' => tokens.push((pos, self.consume_literal(pos)?)),
                        '=' => match self.iter.peek() {
                            Some(&(_, '=')) => {
                                self.iter.next();
                                tokens.push((pos, Eq))
                            }
                            #[cfg(feature = "let-expr")]
                            _ => tokens.push((pos, Assign)),
                            #[cfg(not(feature = "let-expr"))]
                            _ => {
                                let message = "'=' is not valid. Did you mean '=='?";
                                let reason = ErrorReason::Parse(message.to_owned());
                                return Err(JmespathError::new(self.expr, pos, reason));
                            }
                        },
                        '>' => tokens.push((pos, self.alt('=', Gte, Gt))),
                        '<' => tokens.push((pos, self.alt('=', Lte, Lt))),
                        '!' => tokens.push((pos, self.alt('=', Ne, Not))),
                        '0'..='9' => tokens.push((pos, self.consume_number(pos, ch, false)?)),
                        '-' => tokens.push((pos, self.consume_negative_number(pos)?)),
                        #[cfg(feature = "let-expr")]
                        '$' => tokens.push((pos, self.consume_variable(pos)?)),
                        // Skip whitespace tokens
                        ' ' | '\n' | '\t' | '\r' => {}
                        c => {
                            let reason = ErrorReason::Parse(format!("Invalid character: {c}"));
                            return Err(JmespathError::new(self.expr, pos, reason));
                        }
                    }
                }
                None => {
                    tokens.push((last_position, Eof));
                    return Ok(tokens);
                }
            }
        }
    }

    /// Advances past characters matching the predicate, returning the end byte offset.
    #[inline]
    fn skip_while<F>(&mut self, predicate: F) -> usize
    where
        F: Fn(char) -> bool,
    {
        loop {
            match self.iter.peek() {
                Some(&(_, c)) if predicate(c) => {
                    self.iter.next();
                    // Continue -- we'll use the *next* peek or EOF to get the end
                }
                Some(&(end, _)) => return end,
                None => return self.expr.len(),
            }
        }
    }

    #[inline]
    fn consume_lbracket(&mut self) -> Token<'a> {
        match self.iter.peek() {
            Some(&(_, ']')) => {
                self.iter.next();
                Flatten
            }
            Some(&(_, '?')) => {
                self.iter.next();
                Filter
            }
            _ => Lbracket,
        }
    }

    /// Zero-copy: slices directly from the expression string.
    #[inline]
    fn consume_identifier(&mut self, start: usize) -> Token<'a> {
        let end = self.skip_while(|c| matches!(c, 'a'..='z' | '_' | 'A'..='Z' | '0'..='9'));
        Identifier(&self.expr[start..end])
    }

    /// Parses a number directly via arithmetic, no String allocation.
    #[inline]
    fn consume_number(
        &mut self,
        pos: usize,
        first_char: char,
        is_negative: bool,
    ) -> Result<Token<'a>, JmespathError> {
        let mut value: i32 = (first_char as i32) - ('0' as i32);
        loop {
            match self.iter.peek() {
                Some(&(_, c)) if c.is_ascii_digit() => {
                    value = value
                        .checked_mul(10)
                        .and_then(|v| v.checked_add((c as i32) - ('0' as i32)))
                        .ok_or_else(|| {
                            let reason = ErrorReason::Parse("Expected valid number".to_owned());
                            JmespathError::new(self.expr, pos, reason)
                        })?;
                    self.iter.next();
                }
                _ => break,
            }
        }
        Ok(if is_negative {
            Number(-value)
        } else {
            Number(value)
        })
    }

    #[inline]
    fn consume_negative_number(&mut self, pos: usize) -> Result<Token<'a>, JmespathError> {
        match self.iter.next() {
            Some((_, c)) if c.is_numeric() && c != '0' => Ok(self.consume_number(pos, c, true)?),
            _ => {
                let reason = ErrorReason::Parse("'-' must be followed by numbers 1-9".to_owned());
                Err(JmespathError::new(self.expr, pos, reason))
            }
        }
    }

    /// Zero-copy: slices the variable name directly from the expression.
    #[cfg(feature = "let-expr")]
    #[inline]
    fn consume_variable(&mut self, pos: usize) -> Result<Token<'a>, JmespathError> {
        match self.iter.peek() {
            Some(&(start, 'a'..='z' | 'A'..='Z' | '_')) => {
                self.iter.next();
                let end = self.skip_while(|c| matches!(c, 'a'..='z' | '_' | 'A'..='Z' | '0'..='9'));
                Ok(Variable(&self.expr[start..end]))
            }
            _ => {
                let reason =
                    ErrorReason::Parse("'$' must be followed by a valid identifier".to_owned());
                Err(JmespathError::new(self.expr, pos, reason))
            }
        }
    }

    #[inline]
    fn consume_inside<F>(
        &mut self,
        pos: usize,
        wrapper: char,
        invoke: F,
    ) -> Result<Token<'a>, JmespathError>
    where
        F: Fn(String) -> Result<Token<'a>, String>,
    {
        let mut buffer = String::new();
        while let Some((_, c)) = self.iter.next() {
            if c == wrapper {
                return invoke(buffer)
                    .map_err(|e| JmespathError::new(self.expr, pos, ErrorReason::Parse(e)));
            } else if c == '\\' {
                buffer.push(c);
                if let Some((_, c)) = self.iter.next() {
                    buffer.push(c);
                }
            } else {
                buffer.push(c)
            }
        }
        let message = format!("Unclosed {wrapper} delimiter: {wrapper}{buffer}");
        Err(JmespathError::new(
            self.expr,
            pos,
            ErrorReason::Parse(message),
        ))
    }

    #[inline]
    fn consume_quoted_identifier(&mut self, pos: usize) -> Result<Token<'a>, JmespathError> {
        self.consume_inside(pos, '"', |s| {
            // JSON decode the string to expand escapes
            let json_str = format!(r#""{s}""#);
            match serde_json::from_str::<Value>(&json_str) {
                Ok(Value::String(decoded)) => Ok(QuotedIdentifier(decoded)),
                Ok(_) => Err("consume_quoted_identifier expected a string".to_owned()),
                Err(e) => Err(format!("Unable to parse quoted identifier {s}: {e}")),
            }
        })
    }

    #[inline]
    fn consume_raw_string(&mut self, pos: usize) -> Result<Token<'a>, JmespathError> {
        self.consume_inside(pos, '\'', |s| {
            Ok(Literal(Value::String(s.replace("\\'", "'"))))
        })
    }

    #[inline]
    fn consume_literal(&mut self, pos: usize) -> Result<Token<'a>, JmespathError> {
        self.consume_inside(pos, '`', |s| {
            let unescaped = s.replace("\\`", "`");
            match serde_json::from_str::<Value>(&unescaped) {
                Ok(v) => Ok(Literal(v)),
                Err(err) => Err(format!("Unable to parse literal JSON {s}: {err}")),
            }
        })
    }

    #[inline]
    fn alt(&mut self, expected: char, match_type: Token<'a>, else_type: Token<'a>) -> Token<'a> {
        match self.iter.peek() {
            Some(&(_, c)) if c == expected => {
                self.iter.next();
                match_type
            }
            _ => else_type,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tokenize_vec(expr: &str) -> Vec<(usize, Token<'_>)> {
        tokenize(expr).unwrap()
    }

    #[test]
    fn tokenize_basic_test() {
        assert_eq!(tokenize_vec("."), vec![(0, Dot), (1, Eof)]);
        assert_eq!(tokenize_vec("*"), vec![(0, Star), (1, Eof)]);
        assert_eq!(tokenize_vec("@"), vec![(0, At), (1, Eof)]);
        assert_eq!(tokenize_vec("]"), vec![(0, Rbracket), (1, Eof)]);
        assert_eq!(tokenize_vec("{"), vec![(0, Lbrace), (1, Eof)]);
        assert_eq!(tokenize_vec("}"), vec![(0, Rbrace), (1, Eof)]);
        assert_eq!(tokenize_vec("("), vec![(0, Lparen), (1, Eof)]);
        assert_eq!(tokenize_vec(")"), vec![(0, Rparen), (1, Eof)]);
        assert_eq!(tokenize_vec(","), vec![(0, Comma), (1, Eof)]);
    }

    #[test]
    fn tokenize_lbracket_test() {
        assert_eq!(tokenize_vec("["), vec![(0, Lbracket), (1, Eof)]);
        assert_eq!(tokenize_vec("[]"), vec![(0, Flatten), (2, Eof)]);
        assert_eq!(tokenize_vec("[?"), vec![(0, Filter), (2, Eof)]);
    }

    #[test]
    fn tokenize_pipe_test() {
        assert_eq!(tokenize_vec("|"), vec![(0, Pipe), (1, Eof)]);
        assert_eq!(tokenize_vec("||"), vec![(0, Or), (2, Eof)]);
    }

    #[test]
    fn tokenize_and_ampersand_test() {
        assert_eq!(tokenize_vec("&"), vec![(0, Ampersand), (1, Eof)]);
        assert_eq!(tokenize_vec("&&"), vec![(0, And), (2, Eof)]);
    }

    #[test]
    fn tokenize_lt_gt_test() {
        assert_eq!(tokenize_vec("<"), vec![(0, Lt), (1, Eof)]);
        assert_eq!(tokenize_vec("<="), vec![(0, Lte), (2, Eof)]);
        assert_eq!(tokenize_vec(">"), vec![(0, Gt), (1, Eof)]);
        assert_eq!(tokenize_vec(">="), vec![(0, Gte), (2, Eof)]);
    }

    #[test]
    fn tokenize_eq_ne_test() {
        assert_eq!(tokenize_vec("=="), vec![(0, Eq), (2, Eof)]);
        assert_eq!(tokenize_vec("!"), vec![(0, Not), (1, Eof)]);
        assert_eq!(tokenize_vec("!="), vec![(0, Ne), (2, Eof)]);
    }

    #[test]
    #[cfg(feature = "let-expr")]
    fn ensures_eq_valid_with_let_expr() {
        assert!(tokenize("=").is_ok());
    }

    #[test]
    #[cfg(not(feature = "let-expr"))]
    fn ensures_eq_invalid_without_let_expr() {
        assert!(tokenize("=").is_err());
    }

    #[test]
    fn skips_whitespace() {
        let tokens = tokenize_vec(" \t\n\r\t. (");
        assert_eq!(tokens, vec![(5, Dot), (7, Lparen), (8, Eof)]);
    }

    #[test]
    fn tokenize_single_error_test() {
        assert!(
            tokenize("~")
                .unwrap_err()
                .to_string()
                .contains("Invalid character: ~")
        );
    }

    #[test]
    fn tokenize_unclosed_errors_test() {
        assert!(
            tokenize("\"foo")
                .unwrap_err()
                .to_string()
                .contains("Unclosed \" delimiter: \"foo")
        );
        assert!(
            tokenize("`foo")
                .unwrap_err()
                .to_string()
                .contains("Unclosed ` delimiter: `foo")
        );
    }

    #[test]
    fn tokenize_identifier_test() {
        assert_eq!(
            tokenize_vec("foo_bar"),
            vec![(0, Identifier("foo_bar")), (7, Eof)]
        );
        assert_eq!(tokenize_vec("a"), vec![(0, Identifier("a")), (1, Eof)]);
        assert_eq!(tokenize_vec("_a"), vec![(0, Identifier("_a")), (2, Eof)]);
    }

    #[test]
    fn tokenize_quoted_identifier_test() {
        assert_eq!(
            tokenize_vec("\"foo\""),
            vec![(0, QuotedIdentifier("foo".to_string())), (5, Eof)]
        );
        assert_eq!(
            tokenize_vec("\"\""),
            vec![(0, QuotedIdentifier("".to_string())), (2, Eof)]
        );
        assert_eq!(
            tokenize_vec("\"a_b\""),
            vec![(0, QuotedIdentifier("a_b".to_string())), (5, Eof)]
        );
        assert_eq!(
            tokenize_vec("\"a\\nb\""),
            vec![(0, QuotedIdentifier("a\nb".to_string())), (6, Eof)]
        );
        assert_eq!(
            tokenize_vec("\"a\\\\nb\""),
            vec![(0, QuotedIdentifier("a\\nb".to_string())), (7, Eof)]
        );
    }

    #[test]
    fn tokenize_raw_string_test() {
        assert_eq!(
            tokenize_vec("'foo'"),
            vec![(0, Literal(Value::String("foo".to_string()))), (5, Eof)]
        );
        assert_eq!(
            tokenize_vec("''"),
            vec![(0, Literal(Value::String("".to_string()))), (2, Eof)]
        );
        assert_eq!(
            tokenize_vec("'a\\nb'"),
            vec![(0, Literal(Value::String("a\\nb".to_string()))), (6, Eof)]
        );
    }

    #[test]
    fn tokenize_literal_test() {
        assert!(
            tokenize("`a`")
                .unwrap_err()
                .to_string()
                .contains("Unable to parse")
        );
        assert_eq!(
            tokenize_vec("`\"a\"`"),
            vec![(0, Literal(Value::String("a".to_string()))), (5, Eof)]
        );
        assert_eq!(
            tokenize_vec("`\"a b\"`"),
            vec![(0, Literal(Value::String("a b".to_string()))), (7, Eof)]
        );
    }

    #[test]
    fn tokenize_number_test() {
        assert_eq!(tokenize_vec("0"), vec![(0, Number(0)), (1, Eof)]);
        assert_eq!(tokenize_vec("1"), vec![(0, Number(1)), (1, Eof)]);
        assert_eq!(tokenize_vec("123"), vec![(0, Number(123)), (3, Eof)]);
    }

    #[test]
    fn tokenize_negative_number_test() {
        assert_eq!(tokenize_vec("-10"), vec![(0, Number(-10)), (3, Eof)]);
    }

    #[test]
    fn tokenize_negative_number_test_failure() {
        assert!(tokenize("-01").unwrap_err().to_string().contains("'-'"));
    }

    #[test]
    fn tokenize_successive_test() {
        let expr = "foo.bar || `\"a\"` | 10";
        let tokens = tokenize_vec(expr);
        assert_eq!(tokens[0], (0, Identifier("foo")));
        assert_eq!(tokens[1], (3, Dot));
        assert_eq!(tokens[2], (4, Identifier("bar")));
        assert_eq!(tokens[3], (8, Or));
        assert_eq!(tokens[4], (11, Literal(Value::String("a".to_string()))));
        assert_eq!(tokens[5], (17, Pipe));
        assert_eq!(tokens[6], (19, Number(10)));
        assert_eq!(tokens[7], (21, Eof));
    }

    #[test]
    fn tokenizes_slices() {
        let tokens = tokenize_vec("foo[0::-1]");
        assert_eq!(
            "[(0, Identifier(\"foo\")), (3, Lbracket), (4, Number(0)), (5, Colon), \
                     (6, Colon), (7, Number(-1)), (9, Rbracket), (10, Eof)]",
            format!("{tokens:?}")
        );
    }
}
