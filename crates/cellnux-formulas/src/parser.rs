// SPDX-License-Identifier: MPL-2.0
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Precedence-climbing parser producing an [`Expr`].

use crate::ast::{BinaryOp, Expr, UnaryOp};
use crate::error::{ParseError, ParseErrorKind};
use crate::token::{Spanned, Token, tokenize};

/// Maximum nesting depth of parentheses, unary operators and calls. Deeper
/// input is rejected with [`ParseErrorKind::TooDeeplyNested`] instead of
/// risking stack exhaustion on hostile formulas.
const MAX_DEPTH: usize = 256;

/// Parses formula text into an expression tree.
///
/// A single leading `=` is accepted and ignored. Whitespace between tokens
/// is ignored; the intersection operator (a space between references) is
/// not supported.
///
/// # Errors
///
/// Returns a [`ParseError`] with the byte position of the problem.
pub fn parse(text: &str) -> Result<Expr, ParseError> {
    let body = text.strip_prefix('=').unwrap_or(text);
    let tokens = tokenize(body)?;
    if tokens.is_empty() {
        return Err(ParseError::new(ParseErrorKind::Empty, 0));
    }
    let end = body.len();
    let mut parser = Parser {
        tokens,
        pos: 0,
        end,
        depth: 0,
    };
    let expr = parser.expression(0)?;
    if let Some(extra) = parser.peek() {
        return Err(ParseError::new(ParseErrorKind::TrailingInput, extra.start));
    }
    Ok(expr)
}

struct Parser {
    tokens: Vec<Spanned>,
    pos: usize,
    end: usize,
    depth: usize,
}

impl Parser {
    fn peek(&self) -> Option<&Spanned> {
        self.tokens.get(self.pos)
    }

    fn peek_token(&self) -> Option<&Token> {
        self.peek().map(|s| &s.token)
    }

    fn position(&self) -> usize {
        self.peek().map_or(self.end, |s| s.start)
    }

    fn advance(&mut self) -> Option<Spanned> {
        let token = self.tokens.get(self.pos).cloned();
        if token.is_some() {
            self.pos += 1;
        }
        token
    }

    fn expect(&mut self, wanted: &Token) -> Result<(), ParseError> {
        match self.advance() {
            Some(s) if &s.token == wanted => Ok(()),
            Some(s) => Err(ParseError::new(
                ParseErrorKind::UnexpectedToken(s.token.describe()),
                s.start,
            )),
            None => Err(ParseError::new(ParseErrorKind::UnexpectedEnd, self.end)),
        }
    }

    fn enter(&mut self) -> Result<(), ParseError> {
        if self.depth >= MAX_DEPTH {
            return Err(ParseError::new(
                ParseErrorKind::TooDeeplyNested,
                self.position(),
            ));
        }
        self.depth += 1;
        Ok(())
    }

    fn leave(&mut self) {
        self.depth -= 1;
    }

    /// Parses an expression whose binary operators all bind at least as
    /// tightly as `min_prec`.
    fn expression(&mut self, min_prec: u8) -> Result<Expr, ParseError> {
        let mut lhs = self.unary()?;
        while let Some(op) = self.peek_token().and_then(binary_op) {
            let prec = op.precedence();
            if prec < min_prec {
                break;
            }
            self.advance();
            // Left-associative: the right operand may only contain tighter operators.
            let rhs = self.expression(prec + 1)?;
            lhs = Expr::Binary {
                op,
                lhs: Box::new(lhs),
                rhs: Box::new(rhs),
            };
        }
        Ok(lhs)
    }

    fn unary(&mut self) -> Result<Expr, ParseError> {
        match self.peek_token() {
            Some(Token::Minus) => {
                self.advance();
                self.enter()?;
                let operand = self.unary()?;
                self.leave();
                Ok(Expr::Unary {
                    op: UnaryOp::Neg,
                    expr: Box::new(operand),
                })
            }
            Some(Token::Plus) => {
                self.advance();
                self.enter()?;
                let operand = self.unary()?;
                self.leave();
                Ok(operand)
            }
            _ => self.postfix(),
        }
    }

    fn postfix(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.primary()?;
        while self.peek_token() == Some(&Token::Percent) {
            self.advance();
            expr = Expr::Unary {
                op: UnaryOp::Percent,
                expr: Box::new(expr),
            };
        }
        Ok(expr)
    }

    fn primary(&mut self) -> Result<Expr, ParseError> {
        let Some(spanned) = self.advance() else {
            return Err(ParseError::new(ParseErrorKind::UnexpectedEnd, self.end));
        };
        match spanned.token {
            Token::Number(n) => Ok(Expr::Number(n)),
            Token::Text(s) => Ok(Expr::Text(s)),
            Token::Bool(b) => Ok(Expr::Bool(b)),
            Token::Error(e) => Ok(Expr::Error(e)),
            Token::Reference(start) => {
                if self.peek_token() == Some(&Token::Colon) {
                    self.advance();
                    match self.advance() {
                        Some(Spanned {
                            token: Token::Reference(end),
                            ..
                        }) => Ok(Expr::Range(start, end)),
                        Some(other) => Err(ParseError::new(
                            ParseErrorKind::UnexpectedToken(other.token.describe()),
                            other.start,
                        )),
                        None => Err(ParseError::new(ParseErrorKind::UnexpectedEnd, self.end)),
                    }
                } else {
                    Ok(Expr::Ref(start))
                }
            }
            Token::Ident(name) => {
                if self.peek_token() == Some(&Token::LParen) {
                    self.advance();
                    self.enter()?;
                    let args = self.arguments()?;
                    self.leave();
                    Ok(Expr::Call {
                        name: name.to_ascii_uppercase(),
                        args,
                    })
                } else {
                    Ok(Expr::Name(name))
                }
            }
            Token::LParen => {
                self.enter()?;
                let inner = self.expression(0)?;
                self.leave();
                self.expect(&Token::RParen)?;
                Ok(inner)
            }
            other => Err(ParseError::new(
                ParseErrorKind::UnexpectedToken(other.describe()),
                spanned.start,
            )),
        }
    }

    /// Parses `arg, arg, ...)` after the opening parenthesis has been consumed.
    fn arguments(&mut self) -> Result<Vec<Expr>, ParseError> {
        let mut args = Vec::new();
        if self.peek_token() == Some(&Token::RParen) {
            self.advance();
            return Ok(args);
        }
        loop {
            match self.peek_token() {
                Some(Token::Comma | Token::RParen) => args.push(Expr::Missing),
                _ => args.push(self.expression(0)?),
            }
            match self.advance() {
                Some(Spanned {
                    token: Token::Comma,
                    ..
                }) => {}
                Some(Spanned {
                    token: Token::RParen,
                    ..
                }) => return Ok(args),
                Some(other) => {
                    return Err(ParseError::new(
                        ParseErrorKind::UnexpectedToken(other.token.describe()),
                        other.start,
                    ));
                }
                None => return Err(ParseError::new(ParseErrorKind::UnexpectedEnd, self.end)),
            }
        }
    }
}

const fn binary_op(token: &Token) -> Option<BinaryOp> {
    Some(match token {
        Token::Plus => BinaryOp::Add,
        Token::Minus => BinaryOp::Sub,
        Token::Star => BinaryOp::Mul,
        Token::Slash => BinaryOp::Div,
        Token::Caret => BinaryOp::Pow,
        Token::Ampersand => BinaryOp::Concat,
        Token::Equal => BinaryOp::Eq,
        Token::NotEqual => BinaryOp::Ne,
        Token::Less => BinaryOp::Lt,
        Token::Greater => BinaryOp::Gt,
        Token::LessEqual => BinaryOp::Le,
        Token::GreaterEqual => BinaryOp::Ge,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    /// Parses and prints back; the canonical text makes tree shape visible.
    fn canon(input: &str) -> String {
        parse(input).unwrap().to_string()
    }

    #[test]
    fn precedence_and_associativity_follow_spreadsheet_rules() {
        assert_eq!(canon("1+2*3"), "1+2*3");
        assert_eq!(canon("(1+2)*3"), "(1+2)*3");
        assert_eq!(canon("1-2-3"), "1-2-3");
        assert_eq!(canon("1-(2-3)"), "1-(2-3)");
        assert_eq!(canon("2^3^2"), "2^3^2");
        assert_eq!(parse("2^3^2").unwrap(), parse("(2^3)^2").unwrap());
        assert_eq!(canon("-2^2"), "-2^2");
        assert_eq!(parse("-2^2").unwrap(), parse("(-2)^2").unwrap());
        assert_eq!(canon("1&2+3"), "1&2+3");
        assert_eq!(parse("1&2+3").unwrap(), parse("1&(2+3)").unwrap());
        assert_eq!(canon("1+2=3"), "1+2=3");
        assert_eq!(parse("1+2=3").unwrap(), parse("(1+2)=3").unwrap());
        assert_eq!(canon("50%*2"), "50%*2");
        assert_eq!(canon("(1+2)%"), "(1+2)%");
        assert_eq!(canon("--1"), "--1");
        // Found by the print/parse property test: postfix binds tighter than prefix.
        assert_eq!(canon("-5%"), "-5%");
        assert_eq!(canon("(-5)%"), "(-5)%");
        assert_ne!(parse("-5%").unwrap(), parse("(-5)%").unwrap());
        assert_eq!(canon("+1"), "1");
    }

    #[test]
    fn parses_references_ranges_calls_and_names() {
        assert_eq!(canon("=a1+$B$2"), "A1+B2");
        assert_eq!(canon("sum(a1:b10, 3)"), "SUM(A1:B10,3)");
        assert_eq!(canon("IF(A1,,2)"), "IF(A1,,2)");
        assert_eq!(canon("IF(A1,1,)"), "IF(A1,1,)");
        assert_eq!(canon("NOW()"), "NOW()");
        assert_eq!(canon("MyName*2"), "MyName*2");
        assert_eq!(parse("MyName").unwrap(), Expr::Name("MyName".to_owned()));
        assert_eq!(canon(r#""say ""hi"""&TRUE"#), r#""say ""hi"""&TRUE"#);
        assert_eq!(canon("#N/A"), "#N/A");
    }

    #[test]
    fn reports_errors_with_positions() {
        let cases = [
            ("", ParseErrorKind::Empty, 0),
            ("=", ParseErrorKind::Empty, 0),
            ("1+", ParseErrorKind::UnexpectedEnd, 2),
            ("(1+2", ParseErrorKind::UnexpectedEnd, 4),
            ("1+2)", ParseErrorKind::TrailingInput, 3),
            ("1 2", ParseErrorKind::TrailingInput, 2),
            ("SUM(1,", ParseErrorKind::UnexpectedEnd, 6),
            ("A1:", ParseErrorKind::UnexpectedEnd, 3),
            (
                "A1:5",
                ParseErrorKind::UnexpectedToken("number 5".to_owned()),
                3,
            ),
            ("*1", ParseErrorKind::UnexpectedToken("'*'".to_owned()), 0),
            (
                "SUM(1 2)",
                ParseErrorKind::UnexpectedToken("number 2".to_owned()),
                6,
            ),
        ];
        for (input, kind, position) in cases {
            let err = parse(input).unwrap_err();
            assert_eq!((input, err.kind), (input, kind));
            assert_eq!((input, err.position), (input, position));
        }
    }

    #[test]
    fn rejects_excessive_nesting_instead_of_overflowing() {
        let deep = format!("{}1{}", "(".repeat(10_000), ")".repeat(10_000));
        assert_eq!(
            parse(&deep).unwrap_err().kind,
            ParseErrorKind::TooDeeplyNested
        );
        let minuses = "-".repeat(10_000) + "1";
        assert_eq!(
            parse(&minuses).unwrap_err().kind,
            ParseErrorKind::TooDeeplyNested
        );
        let calls = format!("{}1{}", "SUM(".repeat(10_000), ")".repeat(10_000));
        assert_eq!(
            parse(&calls).unwrap_err().kind,
            ParseErrorKind::TooDeeplyNested
        );
        let ok = format!("{}1{}", "(".repeat(200), ")".repeat(200));
        assert!(parse(&ok).is_ok());
    }
}
