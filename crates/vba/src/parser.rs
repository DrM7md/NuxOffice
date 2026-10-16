// SPDX-License-Identifier: MPL-2.0
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Recursive-descent parser for VBA standard modules.

use crate::ast::*;
use crate::error::ParseError;
use crate::lexer::{Sym, Tok, Token, tokenize};
use crate::value::Variant;

/// Maximum nesting of blocks and expressions; deeper input is rejected.
const MAX_DEPTH: usize = 200;

/// Words that can never be used as plain identifiers in expressions.
const RESERVED: &[&str] = &[
    "and", "as", "byref", "byval", "call", "case", "const", "dim", "do", "each", "else", "elseif", "end", "eqv", "exit", "for", "function", "goto",
    "if", "imp", "in", "is", "like", "loop", "mod", "new", "next", "not", "on", "or", "redim", "resume", "select", "set", "step", "sub", "then",
    "to", "until", "wend", "while", "with", "xor",
];

/// Parses one standard module.
pub fn parse_module(name: &str, src: &str) -> Result<Module, ParseError> {
    let tokens = tokenize(src)?;
    let mut p = Parser { tokens, pos: 0, depth: 0 };
    let mut m = Module { name: name.to_string(), ..Module::default() };
    p.module(&mut m)?;
    Ok(m)
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    depth: usize,
}

impl Parser {
    // ---- token helpers -------------------------------------------------

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }
    fn peek_at(&self, n: usize) -> Option<&Token> {
        self.tokens.get(self.pos + n)
    }
    fn line(&self) -> u32 {
        self.peek().or_else(|| self.tokens.last()).map_or(1, |t| t.line)
    }
    fn err(&self, msg: impl Into<String>) -> ParseError {
        ParseError::new(self.line(), msg)
    }
    fn eof(&self) -> bool {
        self.pos >= self.tokens.len()
    }
    fn is_kw(&self, kw: &str) -> bool {
        self.peek().is_some_and(|t| t.is_kw(kw))
    }
    fn is_kw_at(&self, n: usize, kw: &str) -> bool {
        self.peek_at(n).is_some_and(|t| t.is_kw(kw))
    }
    fn is_sym(&self, s: Sym) -> bool {
        matches!(self.peek(), Some(Token { tok: Tok::Sym(x), .. }) if *x == s)
    }
    fn eat_kw(&mut self, kw: &str) -> bool {
        if self.is_kw(kw) {
            self.pos += 1;
            true
        } else {
            false
        }
    }
    fn eat_sym(&mut self, s: Sym) -> bool {
        if self.is_sym(s) {
            self.pos += 1;
            true
        } else {
            false
        }
    }
    fn expect_kw(&mut self, kw: &str) -> Result<(), ParseError> {
        if self.eat_kw(kw) { Ok(()) } else { Err(self.err(format!("expected {kw}"))) }
    }
    fn expect_sym(&mut self, s: Sym, what: &str) -> Result<(), ParseError> {
        if self.eat_sym(s) { Ok(()) } else { Err(self.err(format!("expected {what}"))) }
    }
    /// End of a statement: newline, `:` or end of input.
    fn at_stmt_end(&self) -> bool {
        self.eof() || matches!(self.peek(), Some(Token { tok: Tok::Sep | Tok::Sym(Sym::Colon), .. }))
    }
    fn at_newline(&self) -> bool {
        self.eof() || matches!(self.peek(), Some(Token { tok: Tok::Sep, .. }))
    }
    fn skip_seps(&mut self) {
        while matches!(self.peek(), Some(Token { tok: Tok::Sep | Tok::Sym(Sym::Colon), .. })) {
            self.pos += 1;
        }
    }
    fn skip_line(&mut self) {
        while !self.at_newline() {
            self.pos += 1;
        }
    }
    fn end_stmt(&mut self) -> Result<(), ParseError> {
        if self.at_stmt_end() || self.is_kw("else") {
            Ok(())
        } else {
            let found = self.peek().map(|t| format!("{:?}", t.tok)).unwrap_or_default();
            Err(self.err(format!("expected end of statement, found {found}")))
        }
    }
    fn word(&mut self) -> Result<String, ParseError> {
        match self.peek() {
            Some(Token { tok: Tok::Word(w), .. }) => {
                let w = w.clone();
                self.pos += 1;
                Ok(w)
            }
            _ => Err(self.err("expected a name")),
        }
    }
    fn enter(&mut self) -> Result<(), ParseError> {
        self.depth += 1;
        if self.depth > MAX_DEPTH { Err(self.err("code is nested too deeply")) } else { Ok(()) }
    }
    fn leave(&mut self) {
        self.depth = self.depth.saturating_sub(1);
    }

    // ---- module level --------------------------------------------------

    fn module(&mut self, m: &mut Module) -> Result<(), ParseError> {
        loop {
            self.skip_seps();
            if self.eof() {
                return Ok(());
            }
            let line = self.line();
            if self.eat_kw("attribute") {
                self.skip_line();
                continue;
            }
            if self.eat_kw("option") {
                if self.eat_kw("explicit") {
                    m.option_explicit = true;
                } else if self.eat_kw("base") {
                    m.option_base = match self.peek().map(|t| &t.tok) {
                        Some(Tok::Int(1)) => 1,
                        Some(Tok::Int(0)) => 0,
                        _ => return Err(self.err("Option Base must be 0 or 1")),
                    };
                    self.pos += 1;
                } else {
                    // Option Compare, Option Private Module: accepted, no effect yet.
                    self.skip_line();
                }
                continue;
            }
            let mut public = true;
            let mut scoped = false;
            for kw in ["public", "global", "friend"] {
                if self.eat_kw(kw) {
                    scoped = true;
                }
            }
            if self.eat_kw("private") {
                public = false;
                scoped = true;
            }
            self.eat_kw("static");
            if self.is_kw("sub") || self.is_kw("function") {
                let p = self.procedure(public)?;
                m.procedures.push(p);
                continue;
            }
            if self.eat_kw("property") {
                return Err(ParseError::new(line, "Property procedures (class modules) are not supported yet"));
            }
            if self.eat_kw("declare") {
                self.eat_kw("ptrsafe");
                let _ = self.eat_kw("sub") || self.eat_kw("function");
                let name = self.word()?;
                m.external_declares.push(name);
                self.skip_line();
                continue;
            }
            if self.eat_kw("const") {
                let consts = self.const_list()?;
                m.constants.extend(consts);
                self.end_stmt()?;
                continue;
            }
            if self.eat_kw("enum") {
                self.enum_block(m)?;
                continue;
            }
            if self.is_kw("type") {
                return Err(ParseError::new(line, "user-defined types (Type ... End Type) are not supported yet"));
            }
            if self.eat_kw("dim") || scoped {
                let vars = self.dim_list()?;
                m.variables.extend(vars);
                self.end_stmt()?;
                continue;
            }
            return Err(self.err("expected Sub, Function or a declaration at module level"));
        }
    }

    fn enum_block(&mut self, m: &mut Module) -> Result<(), ParseError> {
        self.word()?;
        let mut next: i64 = 0;
        loop {
            self.skip_seps();
            if self.eof() {
                return Err(self.err("missing End Enum"));
            }
            if self.is_kw("end") && self.is_kw_at(1, "enum") {
                self.pos += 2;
                return Ok(());
            }
            let name = self.word()?;
            let value = if self.eat_sym(Sym::Eq) {
                let e = self.expr()?;
                match &e {
                    Expr::Literal(v) => v.to_i64().map_err(|_| self.err("enum values must be whole numbers"))?,
                    Expr::Unary(UnOp::Neg, inner) => match inner.as_ref() {
                        Expr::Literal(v) => -v.to_i64().map_err(|_| self.err("enum values must be whole numbers"))?,
                        _ => return Err(self.err("enum values must be constants")),
                    },
                    _ => return Err(self.err("enum values must be constants")),
                }
            } else {
                next
            };
            next = value.saturating_add(1);
            m.constants.push((name, Expr::Literal(Variant::Long(i32::try_from(value).unwrap_or(i32::MAX)))));
        }
    }

    fn procedure(&mut self, public: bool) -> Result<Procedure, ParseError> {
        let line = self.line();
        let kind = if self.eat_kw("sub") {
            ProcKind::Sub
        } else {
            self.expect_kw("function")?;
            ProcKind::Function
        };
        let name = self.word()?;
        let mut params = Vec::new();
        if self.eat_sym(Sym::LParen) && !self.eat_sym(Sym::RParen) {
            loop {
                params.push(self.param()?);
                if self.eat_sym(Sym::RParen) {
                    break;
                }
                self.expect_sym(Sym::Comma, "',' or ')'")?;
            }
        }
        let returns = if self.eat_kw("as") { Some(self.type_name()?) } else { None };
        self.end_stmt()?;
        let end_kw = if kind == ProcKind::Sub { "sub" } else { "function" };
        let body = self.block(&[&["end", end_kw]])?;
        if !(self.eat_kw("end") && self.eat_kw(end_kw)) {
            return Err(ParseError::new(line, format!("missing End {}", if kind == ProcKind::Sub { "Sub" } else { "Function" })));
        }
        Ok(Procedure { name, kind, public, params, returns, body, line })
    }

    fn param(&mut self) -> Result<Param, ParseError> {
        let optional = self.eat_kw("optional");
        let mut by_ref = true;
        if self.eat_kw("byval") {
            by_ref = false;
        } else {
            self.eat_kw("byref");
        }
        let param_array = self.eat_kw("paramarray");
        let name = self.word()?;
        let is_array = if self.eat_sym(Sym::LParen) {
            self.expect_sym(Sym::RParen, "')'")?;
            true
        } else {
            false
        };
        let ty = if self.eat_kw("as") { Some(self.type_name()?) } else { None };
        let default = if self.eat_sym(Sym::Eq) { Some(self.expr()?) } else { None };
        Ok(Param { name, by_ref, optional, param_array, is_array, ty, default })
    }

    fn type_name(&mut self) -> Result<TypeName, ParseError> {
        self.eat_kw("new");
        let mut w = self.word()?;
        // Qualified names such as Excel.Range.
        while self.eat_sym(Sym::Dot) {
            w = self.word()?;
        }
        // `String * 10` fixed-length strings: the length is ignored.
        if w.eq_ignore_ascii_case("string") && self.eat_sym(Sym::Star) {
            self.pos += 1;
        }
        Ok(TypeName::parse(&w))
    }

    fn const_list(&mut self) -> Result<Vec<(String, Expr)>, ParseError> {
        let mut out = Vec::new();
        loop {
            let name = self.word()?;
            if self.eat_kw("as") {
                self.type_name()?;
            }
            self.expect_sym(Sym::Eq, "'='")?;
            out.push((name, self.expr()?));
            if !self.eat_sym(Sym::Comma) {
                return Ok(out);
            }
        }
    }

    fn dim_list(&mut self) -> Result<Vec<DimVar>, ParseError> {
        let mut out = Vec::new();
        loop {
            self.eat_kw("withevents");
            let line = self.line();
            let name = self.word()?;
            let bounds = if self.eat_sym(Sym::LParen) {
                let mut b = Vec::new();
                if !self.eat_sym(Sym::RParen) {
                    loop {
                        let first = self.expr()?;
                        if self.eat_kw("to") {
                            b.push((Some(first), self.expr()?));
                        } else {
                            b.push((None, first));
                        }
                        if self.eat_sym(Sym::RParen) {
                            break;
                        }
                        self.expect_sym(Sym::Comma, "',' or ')'")?;
                    }
                }
                Some(b)
            } else {
                None
            };
            let ty = if self.eat_kw("as") { Some(self.type_name()?) } else { None };
            out.push(DimVar { name, ty, bounds, line });
            if !self.eat_sym(Sym::Comma) {
                return Ok(out);
            }
        }
    }

    // ---- blocks and statements ------------------------------------------

    fn at_terminator(&self, terms: &[&[&str]]) -> bool {
        terms.iter().any(|seq| seq.iter().enumerate().all(|(i, kw)| self.is_kw_at(i, kw)))
    }

    /// Parses statements until one of `terms` starts a statement (not consumed).
    fn block(&mut self, terms: &[&[&str]]) -> Result<Vec<Stmt>, ParseError> {
        self.enter()?;
        let mut out = Vec::new();
        loop {
            self.skip_seps();
            if self.eof() {
                self.leave();
                let want: Vec<String> = terms.iter().map(|t| t.join(" ")).collect();
                return Err(self.err(format!("unexpected end of module; expected {}", want.join(" or "))));
            }
            if self.at_terminator(terms) {
                self.leave();
                return Ok(out);
            }
            out.push(self.statement()?);
            self.end_stmt()?;
        }
    }

    fn statement(&mut self) -> Result<Stmt, ParseError> {
        let line = self.line();
        let kind = self.statement_kind()?;
        Ok(Stmt { line, kind })
    }

    fn statement_kind(&mut self) -> Result<StmtKind, ParseError> {
        // Label: `name:` at the start of a statement.
        if let (Some(Token { tok: Tok::Word(w), .. }), Some(Token { tok: Tok::Sym(Sym::Colon), .. })) = (self.peek(), self.peek_at(1))
            && !RESERVED.contains(&w.to_ascii_lowercase().as_str())
        {
            let w = w.clone();
            self.pos += 2;
            return Ok(StmtKind::Label(w));
        }
        if self.eat_kw("dim") || self.eat_kw("static") {
            return Ok(StmtKind::Dim(self.dim_list()?));
        }
        if self.eat_kw("redim") {
            let preserve = self.eat_kw("preserve");
            return Ok(StmtKind::ReDim { preserve, vars: self.dim_list()? });
        }
        if self.eat_kw("const") {
            return Ok(StmtKind::Const(self.const_list()?));
        }
        if self.eat_kw("set") {
            let target = self.postfix(false)?;
            self.expect_sym(Sym::Eq, "'='")?;
            return Ok(StmtKind::Assign { target, value: self.expr()?, set: true });
        }
        if self.eat_kw("let") {
            let target = self.postfix(false)?;
            self.expect_sym(Sym::Eq, "'='")?;
            return Ok(StmtKind::Assign { target, value: self.expr()?, set: false });
        }
        if self.eat_kw("call") {
            return Ok(StmtKind::Call(self.postfix(false)?));
        }
        if self.eat_kw("if") {
            return self.if_stmt();
        }
        if self.eat_kw("for") {
            return self.for_stmt();
        }
        if self.eat_kw("do") {
            return self.do_stmt();
        }
        if self.eat_kw("while") {
            let cond = self.expr()?;
            let body = self.block(&[&["wend"]])?;
            self.expect_kw("wend")?;
            return Ok(StmtKind::While { cond, body });
        }
        if self.eat_kw("select") {
            return self.select_stmt();
        }
        if self.eat_kw("with") {
            let object = self.expr()?;
            let body = self.block(&[&["end", "with"]])?;
            self.pos += 2;
            return Ok(StmtKind::With { object, body });
        }
        if self.eat_kw("exit") {
            let k = if self.eat_kw("sub") {
                ExitKind::Sub
            } else if self.eat_kw("function") {
                ExitKind::Function
            } else if self.eat_kw("for") {
                ExitKind::For
            } else if self.eat_kw("do") {
                ExitKind::Do
            } else {
                return Err(self.err("expected Sub, Function, For or Do after Exit"));
            };
            return Ok(StmtKind::Exit(k));
        }
        if self.eat_kw("on") {
            if self.eat_kw("error") {
                if self.eat_kw("resume") {
                    self.expect_kw("next")?;
                    return Ok(StmtKind::OnError(OnError::ResumeNext));
                }
                self.expect_kw("goto")?;
                if matches!(self.peek().map(|t| &t.tok), Some(Tok::Int(0))) {
                    self.pos += 1;
                    return Ok(StmtKind::OnError(OnError::GoToZero));
                }
                if self.eat_sym(Sym::Minus) {
                    self.pos += 1;
                    return Ok(StmtKind::OnError(OnError::GoToZero));
                }
                return Ok(StmtKind::OnError(OnError::GoTo(self.word()?)));
            }
            self.skip_line();
            return Ok(StmtKind::Unsupported("On ... GoSub / On ... GoTo".into()));
        }
        if self.eat_kw("goto") {
            return Ok(StmtKind::GoTo(self.word()?));
        }
        if self.eat_kw("resume") {
            if self.eat_kw("next") {
                return Ok(StmtKind::ResumeNext);
            }
            while !self.at_stmt_end() {
                self.pos += 1;
            }
            return Ok(StmtKind::Unsupported("Resume or Resume <label>".into()));
        }
        if self.eat_kw("erase") {
            let mut names = vec![self.word()?];
            while self.eat_sym(Sym::Comma) {
                names.push(self.word()?);
            }
            return Ok(StmtKind::Erase(names));
        }
        if self.is_kw("end") {
            self.pos += 1;
            return Ok(StmtKind::End);
        }
        if self.is_kw("gosub") || self.is_kw("open") && self.is_kw_at(2, "for") {
            while !self.at_stmt_end() {
                self.pos += 1;
            }
            return Ok(StmtKind::Unsupported("GoSub and file I/O statements".into()));
        }
        self.expr_statement()
    }

    fn expr_statement(&mut self) -> Result<StmtKind, ParseError> {
        let head = self.postfix(true)?;
        if self.eat_sym(Sym::Eq) {
            return Ok(StmtKind::Assign { target: head, value: self.expr()?, set: false });
        }
        // Debug.Print a; b, c
        if let Expr::Member { object: Some(obj), name } = &head
            && name.eq_ignore_ascii_case("print")
            && matches!(obj.as_ref(), Expr::Ident(d) if d.eq_ignore_ascii_case("debug"))
        {
            let mut items = Vec::new();
            while !self.at_stmt_end() && !self.is_kw("else") {
                let e = self.expr()?;
                let sep = if self.eat_sym(Sym::Semi) {
                    ';'
                } else if self.eat_sym(Sym::Comma) {
                    ','
                } else {
                    '\n'
                };
                items.push((e, sep));
            }
            return Ok(StmtKind::DebugPrint(items));
        }
        if self.at_stmt_end() || self.is_kw("else") {
            return Ok(StmtKind::Call(head));
        }
        // Call statement without parentheses: `MsgBox "Hi", vbOKOnly`.
        let args = self.arg_list_until_stmt_end()?;
        Ok(StmtKind::Call(Expr::Call { callee: Box::new(head), args }))
    }

    fn if_stmt(&mut self) -> Result<StmtKind, ParseError> {
        let cond = self.expr()?;
        self.expect_kw("then")?;
        if !self.at_newline() {
            // Single-line If: statements separated by ':' with an optional Else.
            let mut then_body = Vec::new();
            loop {
                if self.eat_sym(Sym::Colon) {
                    continue;
                }
                if self.at_newline() || self.is_kw("else") {
                    break;
                }
                then_body.push(self.statement()?);
                self.end_stmt()?;
            }
            let mut otherwise = None;
            if self.eat_kw("else") {
                let mut body = Vec::new();
                loop {
                    if self.eat_sym(Sym::Colon) {
                        continue;
                    }
                    if self.at_newline() {
                        break;
                    }
                    body.push(self.statement()?);
                    self.end_stmt()?;
                }
                otherwise = Some(body);
            }
            return Ok(StmtKind::If { arms: vec![(cond, then_body)], otherwise });
        }
        let terms: &[&[&str]] = &[&["elseif"], &["else"], &["end", "if"]];
        let mut arms = vec![(cond, self.block(terms)?)];
        let mut otherwise = None;
        loop {
            if self.eat_kw("elseif") {
                let c = self.expr()?;
                self.expect_kw("then")?;
                arms.push((c, self.block(terms)?));
            } else if self.eat_kw("else") {
                otherwise = Some(self.block(&[&["end", "if"]])?);
            } else {
                self.expect_kw("end")?;
                self.expect_kw("if")?;
                return Ok(StmtKind::If { arms, otherwise });
            }
        }
    }

    fn for_stmt(&mut self) -> Result<StmtKind, ParseError> {
        if self.eat_kw("each") {
            let var = self.word()?;
            self.expect_kw("in")?;
            let collection = self.expr()?;
            let body = self.block(&[&["next"]])?;
            self.next_clause()?;
            return Ok(StmtKind::ForEach { var, collection, body });
        }
        let var = self.word()?;
        self.expect_sym(Sym::Eq, "'='")?;
        let from = self.expr()?;
        self.expect_kw("to")?;
        let to = self.expr()?;
        let step = if self.eat_kw("step") { Some(self.expr()?) } else { None };
        let body = self.block(&[&["next"]])?;
        self.next_clause()?;
        Ok(StmtKind::For { var, from, to, step, body })
    }

    /// `Next`, `Next i` or `Next i, j` (the latter closes only the inner loop here).
    fn next_clause(&mut self) -> Result<(), ParseError> {
        self.expect_kw("next")?;
        if let Some(Token { tok: Tok::Word(_), .. }) = self.peek()
            && !self.is_kw("else")
        {
            self.pos += 1;
            if self.is_sym(Sym::Comma) {
                return Err(self.err("Next with several loop variables is not supported; close each loop with its own Next"));
            }
        }
        Ok(())
    }

    fn do_stmt(&mut self) -> Result<StmtKind, ParseError> {
        let mut cond = None;
        if self.is_kw("while") || self.is_kw("until") {
            let is_while = self.eat_kw("while");
            if !is_while {
                self.expect_kw("until")?;
            }
            cond = Some(LoopCond { is_while, pre: true, expr: self.expr()? });
        }
        let body = self.block(&[&["loop"]])?;
        self.expect_kw("loop")?;
        if self.is_kw("while") || self.is_kw("until") {
            if cond.is_some() {
                return Err(self.err("a Do loop cannot have conditions at both ends"));
            }
            let is_while = self.eat_kw("while");
            if !is_while {
                self.expect_kw("until")?;
            }
            cond = Some(LoopCond { is_while, pre: false, expr: self.expr()? });
        }
        Ok(StmtKind::Do { cond, body })
    }

    fn select_stmt(&mut self) -> Result<StmtKind, ParseError> {
        self.expect_kw("case")?;
        let subject = self.expr()?;
        let terms: &[&[&str]] = &[&["case"], &["end", "select"]];
        let mut cases = Vec::new();
        let mut otherwise = None;
        loop {
            self.skip_seps();
            if self.eat_kw("end") {
                self.expect_kw("select")?;
                return Ok(StmtKind::Select { subject, cases, otherwise });
            }
            self.expect_kw("case")?;
            if self.eat_kw("else") {
                otherwise = Some(self.block(&[&["end", "select"]])?);
                continue;
            }
            let mut tests = Vec::new();
            loop {
                if self.eat_kw("is") {
                    let op = self.cmp_op().ok_or_else(|| self.err("expected a comparison after Case Is"))?;
                    tests.push(CaseTest::Is(op, self.concat()?));
                } else {
                    let a = self.expr()?;
                    if self.eat_kw("to") {
                        tests.push(CaseTest::Range(a, self.expr()?));
                    } else {
                        tests.push(CaseTest::Value(a));
                    }
                }
                if !self.eat_sym(Sym::Comma) {
                    break;
                }
            }
            cases.push((tests, self.block(terms)?));
        }
    }

    // ---- arguments -------------------------------------------------------

    fn arg(&mut self) -> Result<Arg, ParseError> {
        if let (Some(Token { tok: Tok::Word(w), .. }), Some(Token { tok: Tok::Sym(Sym::ColonEq), .. })) = (self.peek(), self.peek_at(1)) {
            let name = w.clone();
            self.pos += 2;
            return Ok(Arg { name: Some(name), value: Some(self.expr()?) });
        }
        Ok(Arg { name: None, value: Some(self.expr()?) })
    }

    /// Arguments inside parentheses; the `(` has been consumed.
    fn paren_args(&mut self) -> Result<Vec<Arg>, ParseError> {
        let mut args = Vec::new();
        if self.eat_sym(Sym::RParen) {
            return Ok(args);
        }
        loop {
            if self.is_sym(Sym::Comma) || self.is_sym(Sym::RParen) {
                args.push(Arg { name: None, value: None });
            } else {
                args.push(self.arg()?);
            }
            if self.eat_sym(Sym::RParen) {
                return Ok(args);
            }
            self.expect_sym(Sym::Comma, "',' or ')'")?;
        }
    }

    fn arg_list_until_stmt_end(&mut self) -> Result<Vec<Arg>, ParseError> {
        let mut args = Vec::new();
        loop {
            if self.is_sym(Sym::Comma) {
                args.push(Arg { name: None, value: None });
            } else {
                args.push(self.arg()?);
            }
            if !self.eat_sym(Sym::Comma) {
                return Ok(args);
            }
            if self.at_stmt_end() {
                args.push(Arg { name: None, value: None });
                return Ok(args);
            }
        }
    }

    // ---- expressions -----------------------------------------------------

    pub(crate) fn expr(&mut self) -> Result<Expr, ParseError> {
        self.enter()?;
        let r = self.binary_level(0);
        self.leave();
        r
    }

    /// Logical operators from loosest to tightest: Imp, Eqv, Xor, Or, And.
    fn binary_level(&mut self, level: usize) -> Result<Expr, ParseError> {
        const LEVELS: [(&str, BinOp); 5] = [("imp", BinOp::Imp), ("eqv", BinOp::Eqv), ("xor", BinOp::Xor), ("or", BinOp::Or), ("and", BinOp::And)];
        let Some((kw, op)) = LEVELS.get(level) else { return self.not_expr() };
        let mut lhs = self.binary_level(level + 1)?;
        while self.eat_kw(kw) {
            let rhs = self.binary_level(level + 1)?;
            lhs = Expr::Binary(*op, Box::new(lhs), Box::new(rhs));
        }
        Ok(lhs)
    }

    fn not_expr(&mut self) -> Result<Expr, ParseError> {
        if self.eat_kw("not") {
            self.enter()?;
            let inner = self.not_expr();
            self.leave();
            return Ok(Expr::Unary(UnOp::Not, Box::new(inner?)));
        }
        self.comparison()
    }

    fn cmp_op(&mut self) -> Option<BinOp> {
        let op = match self.peek().map(|t| &t.tok) {
            Some(Tok::Sym(Sym::Eq)) => BinOp::Eq,
            Some(Tok::Sym(Sym::Ne)) => BinOp::Ne,
            Some(Tok::Sym(Sym::Lt)) => BinOp::Lt,
            Some(Tok::Sym(Sym::Gt)) => BinOp::Gt,
            Some(Tok::Sym(Sym::Le)) => BinOp::Le,
            Some(Tok::Sym(Sym::Ge)) => BinOp::Ge,
            _ if self.is_kw("is") => BinOp::Is,
            _ if self.is_kw("like") => BinOp::Like,
            _ => return None,
        };
        self.pos += 1;
        Some(op)
    }

    fn comparison(&mut self) -> Result<Expr, ParseError> {
        let mut lhs = self.concat()?;
        while let Some(op) = self.cmp_op() {
            let rhs = self.concat()?;
            lhs = Expr::Binary(op, Box::new(lhs), Box::new(rhs));
        }
        Ok(lhs)
    }

    fn concat(&mut self) -> Result<Expr, ParseError> {
        let mut lhs = self.additive()?;
        while self.eat_sym(Sym::Amp) {
            let rhs = self.additive()?;
            lhs = Expr::Binary(BinOp::Concat, Box::new(lhs), Box::new(rhs));
        }
        Ok(lhs)
    }

    fn additive(&mut self) -> Result<Expr, ParseError> {
        let mut lhs = self.modulo()?;
        loop {
            let op = if self.eat_sym(Sym::Plus) {
                BinOp::Add
            } else if self.eat_sym(Sym::Minus) {
                BinOp::Sub
            } else {
                return Ok(lhs);
            };
            let rhs = self.modulo()?;
            lhs = Expr::Binary(op, Box::new(lhs), Box::new(rhs));
        }
    }

    fn modulo(&mut self) -> Result<Expr, ParseError> {
        let mut lhs = self.int_div()?;
        while self.eat_kw("mod") {
            let rhs = self.int_div()?;
            lhs = Expr::Binary(BinOp::Mod, Box::new(lhs), Box::new(rhs));
        }
        Ok(lhs)
    }

    fn int_div(&mut self) -> Result<Expr, ParseError> {
        let mut lhs = self.multiplicative()?;
        while self.eat_sym(Sym::Backslash) {
            let rhs = self.multiplicative()?;
            lhs = Expr::Binary(BinOp::IntDiv, Box::new(lhs), Box::new(rhs));
        }
        Ok(lhs)
    }

    fn multiplicative(&mut self) -> Result<Expr, ParseError> {
        let mut lhs = self.negation()?;
        loop {
            let op = if self.eat_sym(Sym::Star) {
                BinOp::Mul
            } else if self.eat_sym(Sym::Slash) {
                BinOp::Div
            } else {
                return Ok(lhs);
            };
            let rhs = self.negation()?;
            lhs = Expr::Binary(op, Box::new(lhs), Box::new(rhs));
        }
    }

    /// Unary minus binds looser than `^`: `-2 ^ 2` is `-4`.
    fn negation(&mut self) -> Result<Expr, ParseError> {
        if self.eat_sym(Sym::Minus) {
            self.enter()?;
            let inner = self.negation();
            self.leave();
            return Ok(Expr::Unary(UnOp::Neg, Box::new(inner?)));
        }
        if self.eat_sym(Sym::Plus) {
            self.enter()?;
            let inner = self.negation();
            self.leave();
            return inner;
        }
        self.power()
    }

    fn power(&mut self) -> Result<Expr, ParseError> {
        let mut lhs = self.postfix(false)?;
        while self.eat_sym(Sym::Caret) {
            let rhs = self.power_operand()?;
            lhs = Expr::Binary(BinOp::Pow, Box::new(lhs), Box::new(rhs));
        }
        Ok(lhs)
    }

    fn power_operand(&mut self) -> Result<Expr, ParseError> {
        if self.eat_sym(Sym::Minus) {
            self.enter()?;
            let inner = self.power_operand();
            self.leave();
            return Ok(Expr::Unary(UnOp::Neg, Box::new(inner?)));
        }
        self.postfix(false)
    }

    /// A primary followed by `.member` and `(args)` suffixes. In a statement
    /// head, a `(` preceded by a space starts the argument list of a call
    /// statement instead (`MsgBox ("x")`).
    fn postfix(&mut self, stmt_head: bool) -> Result<Expr, ParseError> {
        let mut e = self.primary()?;
        loop {
            // In a statement head, ` .Name` after a space is a With-block argument
            // (`Debug.Print .Value`), not member access.
            if stmt_head && self.is_sym(Sym::Dot) && self.peek().is_some_and(|t| t.space_before) {
                return Ok(e);
            }
            if self.eat_sym(Sym::Dot) {
                let name = self.word()?;
                e = Expr::Member { object: Some(Box::new(e)), name };
            } else if self.is_sym(Sym::LParen) && !(stmt_head && self.peek().is_some_and(|t| t.space_before)) {
                self.pos += 1;
                self.enter()?;
                let args = self.paren_args();
                self.leave();
                e = Expr::Call { callee: Box::new(e), args: args? };
            } else {
                return Ok(e);
            }
        }
    }

    fn primary(&mut self) -> Result<Expr, ParseError> {
        let Some(t) = self.peek().cloned() else { return Err(self.err("expected an expression")) };
        match t.tok {
            Tok::Int(n) => {
                self.pos += 1;
                Ok(Expr::Literal(Variant::from_int(n)))
            }
            Tok::Long(n) => {
                self.pos += 1;
                Ok(Expr::Literal(i32::try_from(n).map_or(Variant::LongLong(n), Variant::Long)))
            }
            Tok::Float(x) => {
                self.pos += 1;
                Ok(Expr::Literal(Variant::Double(x)))
            }
            Tok::Str(s) => {
                self.pos += 1;
                Ok(Expr::Literal(Variant::String(s)))
            }
            Tok::Sym(Sym::LParen) => {
                self.pos += 1;
                let e = self.expr()?;
                self.expect_sym(Sym::RParen, "')'")?;
                Ok(e)
            }
            Tok::Sym(Sym::Dot) => {
                // `.Member` inside a With block.
                self.pos += 1;
                let name = self.word()?;
                Ok(Expr::Member { object: None, name })
            }
            Tok::Word(w) => {
                let lower = w.to_ascii_lowercase();
                self.pos += 1;
                match lower.as_str() {
                    "true" => Ok(Expr::Literal(Variant::Boolean(true))),
                    "false" => Ok(Expr::Literal(Variant::Boolean(false))),
                    "empty" => Ok(Expr::Literal(Variant::Empty)),
                    "null" => Ok(Expr::Literal(Variant::Null)),
                    "nothing" => Ok(Expr::Nothing),
                    "new" => Ok(Expr::New(self.word()?)),
                    _ if RESERVED.contains(&lower.as_str()) => {
                        self.pos -= 1;
                        Err(self.err(format!("expected an expression, found keyword {w}")))
                    }
                    _ => Ok(Expr::Ident(w)),
                }
            }
            other => Err(self.err(format!("expected an expression, found {other:?}"))),
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    fn body(src: &str) -> Vec<Stmt> {
        let m = parse_module("M", &format!("Sub T()\n{src}\nEnd Sub")).unwrap();
        m.procedures.into_iter().next().unwrap().body
    }

    #[test]
    fn blueprint_fixture_parses() {
        let m = parse_module(
            "Module1",
            "Attribute VB_Name = \"Module1\"\nOption Explicit\n\nSub CalculateTotal()\n    Range(\"A1\").Value = 100\n    Range(\"A2\").Value = 200\n    Range(\"A3\").Formula = \"=SUM(A1:A2)\"\nEnd Sub\n",
        )
        .unwrap();
        assert!(m.option_explicit);
        assert_eq!(m.procedures.len(), 1);
        assert_eq!(m.procedures[0].body.len(), 3);
        assert!(matches!(&m.procedures[0].body[0].kind, StmtKind::Assign { set: false, .. }));
    }

    #[test]
    fn precedence() {
        let b = body("x = -2 ^ 2 + 3 * 4 & \"a\" = \"b\" And Not y");
        let StmtKind::Assign { value, .. } = &b[0].kind else { panic!() };
        // And is the outermost operator; Not binds looser than comparison.
        let Expr::Binary(BinOp::And, lhs, rhs) = value else { panic!("{value:?}") };
        assert!(matches!(rhs.as_ref(), Expr::Unary(UnOp::Not, _)));
        let Expr::Binary(BinOp::Eq, cat, _) = lhs.as_ref() else { panic!() };
        let Expr::Binary(BinOp::Concat, sum, _) = cat.as_ref() else { panic!() };
        let Expr::Binary(BinOp::Add, neg, _) = sum.as_ref() else { panic!() };
        assert!(matches!(neg.as_ref(), Expr::Unary(UnOp::Neg, inner) if matches!(inner.as_ref(), Expr::Binary(BinOp::Pow, _, _))));
    }

    #[test]
    fn call_statements_with_and_without_parentheses() {
        let b = body("MsgBox \"Hi\", 1\nMsgBox (\"Hi\")\nFoo\nCall Bar(1, 2)\nx.Method 1, , 3\nResult = f(1)(2)");
        assert!(matches!(&b[0].kind, StmtKind::Call(Expr::Call { args, .. }) if args.len() == 2));
        assert!(matches!(&b[1].kind, StmtKind::Call(Expr::Call { args, .. }) if args.len() == 1));
        assert!(matches!(&b[2].kind, StmtKind::Call(Expr::Ident(_))));
        assert!(matches!(&b[3].kind, StmtKind::Call(Expr::Call { args, .. }) if args.len() == 2));
        assert!(matches!(&b[4].kind, StmtKind::Call(Expr::Call { args, .. }) if args.len() == 3 && args[1].value.is_none()));
        assert!(matches!(&b[5].kind, StmtKind::Assign { .. }));
    }

    #[test]
    fn control_flow() {
        let b = body(
            "If a Then b = 1 Else b = 2\nIf a Then\n x = 1\nElseIf b Then\n x = 2\nElse\n x = 3\nEnd If\nFor i = 1 To 10 Step 2: s = s + i: Next i\nFor Each c In Range(\"A1:A3\")\nNext\nDo While i < 3\n i = i + 1\nLoop\nDo\nLoop Until True\nWhile False\nWend\nSelect Case x\nCase 1, 2\nCase 3 To 5\nCase Is > 10\nCase Else\nEnd Select\nWith Range(\"A1\")\n .Value = 1\nEnd With\nOn Error Resume Next\nOn Error GoTo Handler\nHandler:\nExit Sub",
        );
        let kinds: Vec<&str> = b
            .iter()
            .map(|s| match &s.kind {
                StmtKind::If { .. } => "if",
                StmtKind::For { .. } => "for",
                StmtKind::ForEach { .. } => "each",
                StmtKind::Do { .. } => "do",
                StmtKind::While { .. } => "while",
                StmtKind::Select { .. } => "select",
                StmtKind::With { .. } => "with",
                StmtKind::OnError(_) => "onerror",
                StmtKind::Label(_) => "label",
                StmtKind::Exit(_) => "exit",
                _ => "other",
            })
            .collect();
        assert_eq!(kinds, ["if", "if", "for", "each", "do", "do", "while", "select", "with", "onerror", "onerror", "label", "exit"]);
    }

    #[test]
    fn declarations_and_signatures() {
        let m = parse_module(
            "M",
            "Option Base 1\nPrivate total As Long\nConst Rate As Double = 0.5\nEnum Color\n Red\n Green = 5\n Blue\nEnd Enum\nDeclare PtrSafe Function GetTickCount Lib \"kernel32\" () As Long\nPublic Function Add(ByVal a As Long, Optional b As Long = 1) As Long\n Dim arr(1 To 3, 2) As Integer, s As String\n Add = a + b\nEnd Function",
        )
        .unwrap();
        assert_eq!(m.option_base, 1);
        assert_eq!(m.variables.len(), 1);
        assert_eq!(m.constants.len(), 4);
        assert_eq!(m.external_declares, ["GetTickCount"]);
        let f = &m.procedures[0];
        assert_eq!(f.kind, ProcKind::Function);
        assert!(!f.params[0].by_ref && f.params[1].optional);
        assert_eq!(f.returns, Some(TypeName::Long));
    }

    #[test]
    fn syntax_errors_report_lines() {
        assert_eq!(parse_module("M", "Sub T()\nIf x Then\nEnd Sub").unwrap_err().line, 3);
        assert!(parse_module("M", "Sub T()\nx = \nEnd Sub").is_err());
        assert!(parse_module("M", "Sub T()").is_err());
        let deep = format!("Sub T()\nx = {}1{}\nEnd Sub", "(".repeat(500), ")".repeat(500));
        assert!(parse_module("M", &deep).is_err());
    }
}
