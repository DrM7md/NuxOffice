// SPDX-License-Identifier: MPL-2.0
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! The tree-walking VBA interpreter and the Excel-compatible object model.

use std::cmp::Ordering;
use std::collections::HashMap;

use gridcraft_core::{CellRef, MAX_COLS, MAX_ROWS, RangeRef, Value, col_to_letters};

use crate::ast::*;
use crate::builtins;
use crate::error::{ParseError, RuntimeError};
use crate::host::{Host, Input};
use crate::parser::parse_module;
use crate::value::{Object, Variant, VbArray, default_for, parse_numeric};

/// Resource limits for one macro run. A macro that exceeds a limit stops with
/// a non-trappable error; `On Error Resume Next` cannot hide it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Limits {
    /// Statements and loop iterations executed.
    pub max_steps: u64,
    /// Nested procedure calls.
    pub max_call_depth: usize,
    /// Elements in one array.
    pub max_array_elements: usize,
    /// Bytes in one string.
    pub max_string_len: usize,
    /// Cells touched by one range read or write.
    pub max_range_cells: u64,
}

impl Default for Limits {
    fn default() -> Self {
        Limits {
            max_steps: 10_000_000,
            max_call_depth: 256,
            max_array_elements: 10_000_000,
            max_string_len: 64 * 1024 * 1024,
            max_range_cells: 1_000_000,
        }
    }
}

/// A set of parsed VBA standard modules.
#[derive(Debug, Clone, Default)]
pub struct Program {
    modules: Vec<Module>,
}

impl Program {
    pub fn new() -> Program {
        Program::default()
    }

    /// Parses and adds a standard module.
    pub fn add_module(&mut self, name: &str, source: &str) -> Result<(), ParseError> {
        self.modules.push(parse_module(name, source)?);
        Ok(())
    }

    /// Parses a single module.
    pub fn from_source(source: &str) -> Result<Program, ParseError> {
        let mut p = Program::new();
        p.add_module("Module1", source)?;
        Ok(p)
    }

    /// Macros a user can run: public `Sub`s without required parameters.
    pub fn macros(&self) -> Vec<String> {
        self.modules
            .iter()
            .flat_map(|m| m.procedures.iter())
            .filter(|p| p.kind == ProcKind::Sub && p.public && p.params.iter().all(|a| a.optional || a.param_array))
            .map(|p| p.name.clone())
            .collect()
    }

    fn find(&self, name: &str, from_module: usize) -> Option<(usize, &Procedure)> {
        let own = self.modules.get(from_module).and_then(|m| m.procedures.iter().find(|p| p.name.eq_ignore_ascii_case(name)));
        if let Some(p) = own {
            return Some((from_module, p));
        }
        self.modules.iter().enumerate().find_map(|(i, m)| m.procedures.iter().find(|p| p.public && p.name.eq_ignore_ascii_case(name)).map(|p| (i, p)))
    }
}

/// Runs the macro `name` (a `Sub` or `Function` with no required arguments)
/// against `host`, returning a `Function`'s result or `Empty`.
pub fn run_macro(program: &Program, name: &str, host: &mut dyn Host, limits: Limits) -> Result<Variant, RuntimeError> {
    let mut it = Interp { prog: program, host, limits, steps: 0, depth: 0, globals: Vec::new(), err: (0, String::new()) };
    it.init_globals()?;
    let (module, proc_) = program.find(name, 0).ok_or_else(|| RuntimeError::not_defined(name))?;
    match it.call_proc(module, proc_, &[], None) {
        Ok(v) => Ok(v),
        Err(e) if e.number == END_SIGNAL => Ok(Variant::Empty),
        Err(e) => Err(e),
    }
}

/// Internal error number used to unwind on the `End` statement.
const END_SIGNAL: i32 = -2;

#[derive(Debug, Clone)]
struct Var {
    value: Variant,
    ty: Option<TypeName>,
    constant: bool,
}

struct Frame {
    vars: HashMap<String, Var>,
    module: usize,
    proc_name: String,
    on_error: OnError,
    in_handler: bool,
    with_stack: Vec<Variant>,
}

enum Flow {
    Normal,
    ExitProc,
    ExitFor,
    ExitDo,
    GoTo(String),
}

struct Interp<'p, 'h> {
    prog: &'p Program,
    host: &'h mut dyn Host,
    limits: Limits,
    steps: u64,
    depth: usize,
    globals: Vec<HashMap<String, Var>>,
    /// `Err.Number` and `Err.Description`.
    err: (i32, String),
}

fn key(name: &str) -> String {
    name.to_ascii_lowercase()
}

/// Finds a variable in the frame, then in the frame's module.
fn lookup_mut<'a>(globals: &'a mut [HashMap<String, Var>], f: &'a mut Frame, k: &str) -> Option<&'a mut Var> {
    if f.vars.contains_key(k) {
        return f.vars.get_mut(k);
    }
    globals.get_mut(f.module).and_then(|g| g.get_mut(k))
}

fn cell_to_variant(v: &Value) -> Variant {
    match v {
        Value::Empty => Variant::Empty,
        Value::Number(n) => Variant::Double(*n),
        Value::Text(t) => Variant::String(t.to_string()),
        Value::Bool(b) => Variant::Boolean(*b),
        Value::Error(e) => Variant::CellError(e.as_str().to_string()),
        Value::Array(a) => a.get(0, 0).map_or(Variant::Empty, cell_to_variant),
    }
}

fn address(r: &RangeRef) -> String {
    let one = |c: CellRef| format!("${}${}", col_to_letters(c.col), c.row + 1);
    if r.is_single() { one(r.start) } else { format!("{}:{}", one(r.start), one(r.end)) }
}

const CONSTANTS: &[(&str, i64)] = &[
    ("vbokonly", 0),
    ("vbokcancel", 1),
    ("vbabortretryignore", 2),
    ("vbyesnocancel", 3),
    ("vbyesno", 4),
    ("vbretrycancel", 5),
    ("vbcritical", 16),
    ("vbquestion", 32),
    ("vbexclamation", 48),
    ("vbinformation", 64),
    ("vbok", 1),
    ("vbcancel", 2),
    ("vbabort", 3),
    ("vbretry", 4),
    ("vbignore", 5),
    ("vbyes", 6),
    ("vbno", 7),
    ("vbtrue", -1),
    ("vbfalse", 0),
    ("vbbinarycompare", 0),
    ("vbtextcompare", 1),
    ("xlup", -4162),
    ("xldown", -4121),
    ("xltoleft", -4159),
    ("xltoright", -4161),
    ("xlcalculationautomatic", -4105),
    ("xlcalculationmanual", -4135),
];

const STRING_CONSTANTS: &[(&str, &str)] =
    &[("vbcrlf", "\r\n"), ("vbnewline", "\r\n"), ("vbcr", "\r"), ("vblf", "\n"), ("vbtab", "\t"), ("vbnullstring", ""), ("vbnullchar", "\0")];

impl<'p> Interp<'p, '_> {
    // ---- setup -----------------------------------------------------------

    fn init_globals(&mut self) -> Result<(), RuntimeError> {
        let prog: &'p Program = self.prog;
        self.globals = vec![HashMap::new(); prog.modules.len()];
        for (mi, m) in prog.modules.iter().enumerate() {
            let mut f = self.frame(mi, &m.name);
            for (name, e) in &m.constants {
                let v = self.eval_value(e, &mut f)?;
                if let Some(g) = self.globals.get_mut(mi) {
                    g.insert(key(name), Var { value: v, ty: None, constant: true });
                }
            }
            for d in &m.variables {
                let var = self.new_var(d, &mut f)?;
                if let Some(g) = self.globals.get_mut(mi) {
                    g.insert(key(&d.name), var);
                }
            }
        }
        Ok(())
    }

    fn frame(&self, module: usize, proc_name: &str) -> Frame {
        Frame {
            vars: HashMap::new(),
            module,
            proc_name: proc_name.to_string(),
            on_error: OnError::GoToZero,
            in_handler: false,
            with_stack: Vec::new(),
        }
    }

    fn module(&self, f: &Frame) -> Option<&'p Module> {
        let prog: &'p Program = self.prog;
        prog.modules.get(f.module)
    }

    fn option_base(&self, f: &Frame) -> i64 {
        self.module(f).map_or(0, |m| m.option_base)
    }

    fn tick(&mut self) -> Result<(), RuntimeError> {
        self.steps += 1;
        if self.steps > self.limits.max_steps {
            return Err(RuntimeError::fatal(18, "Execution stopped: the macro exceeded its step limit"));
        }
        Ok(())
    }

    fn new_var(&mut self, d: &'p DimVar, f: &mut Frame) -> Result<Var, RuntimeError> {
        let value = match &d.bounds {
            None => match &d.ty {
                Some(TypeName::Class(c)) if !is_known_class(c) => {
                    return Err(RuntimeError::not_supported(&format!("user-defined type or class {c}")));
                }
                ty => default_for(ty.as_ref()),
            },
            Some(b) if b.is_empty() => Variant::Array(Box::new(VbArray { bounds: Vec::new(), data: Vec::new(), elem: d.ty.clone() })),
            Some(b) => {
                let bounds = self.eval_bounds(b, f)?;
                Variant::Array(Box::new(VbArray::new(bounds, d.ty.clone(), self.limits.max_array_elements)?))
            }
        };
        Ok(Var { value, ty: if d.bounds.is_some() { None } else { d.ty.clone() }, constant: false })
    }

    fn eval_bounds(&mut self, b: &'p [(Option<Expr>, Expr)], f: &mut Frame) -> Result<Vec<(i64, i64)>, RuntimeError> {
        let base = self.option_base(f);
        let mut out = Vec::new();
        for (lo, hi) in b {
            let lo = match lo {
                Some(e) => self.eval_value(e, f)?.to_i64()?,
                None => base,
            };
            out.push((lo, self.eval_value(hi, f)?.to_i64()?));
        }
        Ok(out)
    }

    // ---- procedures --------------------------------------------------------

    fn call_proc(&mut self, module: usize, p: &'p Procedure, args: &'p [Arg], caller: Option<&mut Frame>) -> Result<Variant, RuntimeError> {
        if self.depth >= self.limits.max_call_depth {
            return Err(RuntimeError::fatal(28, "Out of stack space: too many nested procedure calls"));
        }
        let mut f = self.frame(module, &p.name);
        // Bind arguments: positional first, then named.
        let mut byref_back: Vec<(String, String)> = Vec::new();
        let mut bound: Vec<Option<(Variant, Option<&Expr>)>> = vec![None; p.params.len()];
        let mut rest: Vec<Variant> = Vec::new();
        let mut caller = caller;
        for (i, a) in args.iter().enumerate() {
            let slot = match &a.name {
                Some(n) => p.params.iter().position(|q| q.name.eq_ignore_ascii_case(n)).ok_or_else(RuntimeError::wrong_arg_count)?,
                None => i,
            };
            let Some(e) = &a.value else { continue };
            let v = match caller.as_deref_mut() {
                Some(cf) => self.eval(e, cf)?,
                None => return Err(RuntimeError::wrong_arg_count()),
            };
            match p.params.get(slot) {
                Some(q) if q.param_array => rest.push(v),
                Some(_) => {
                    if let Some(b) = bound.get_mut(slot) {
                        *b = Some((v, Some(e)));
                    }
                }
                None if p.params.last().is_some_and(|q| q.param_array) => rest.push(v),
                None => return Err(RuntimeError::wrong_arg_count()),
            }
        }
        for (q, b) in p.params.iter().zip(bound) {
            let value = if q.param_array {
                Variant::Array(Box::new(VbArray::from_vec(std::mem::take(&mut rest))))
            } else {
                match b {
                    Some((v, src)) => {
                        if q.by_ref
                            && let Some(Expr::Ident(name)) = src
                        {
                            byref_back.push((key(&q.name), key(name)));
                        }
                        let v = if q.ty.as_ref().is_some_and(TypeName::is_object) { v } else { self.deref(v)? };
                        v.coerce(if q.is_array { None } else { q.ty.as_ref() })?
                    }
                    None if q.optional => match &q.default {
                        Some(d) => self.eval_value(d, &mut f)?.coerce(q.ty.as_ref())?,
                        None => default_for(q.ty.as_ref()),
                    },
                    None => return Err(RuntimeError::arg_not_optional()),
                }
            };
            f.vars.insert(key(&q.name), Var { value, ty: if q.is_array { None } else { q.ty.clone() }, constant: false });
        }
        if p.kind == ProcKind::Function {
            f.vars.insert(key(&p.name), Var { value: default_for(p.returns.as_ref()), ty: p.returns.clone(), constant: false });
        }
        self.depth += 1;
        let r = self.run_body(&p.body, &mut f);
        self.depth -= 1;
        r?;
        // Copy ByRef arguments back to the caller's variables.
        if let Some(cf) = caller {
            for (param, target) in byref_back {
                if let Some(v) = f.vars.get(&param).map(|v| v.value.clone())
                    && let Some(var) = lookup_mut(&mut self.globals, cf, &target)
                    && !var.constant
                {
                    var.value = v.coerce(var.ty.as_ref())?;
                }
            }
        }
        Ok(if p.kind == ProcKind::Function { f.vars.remove(&key(&p.name)).map(|v| v.value).unwrap_or_default() } else { Variant::Empty })
    }

    /// Runs a procedure body, resolving `GoTo` to top-level labels.
    fn run_body(&mut self, body: &'p [Stmt], f: &mut Frame) -> Result<(), RuntimeError> {
        let mut start = 0;
        loop {
            match self.exec_from(body, start, f)? {
                Flow::GoTo(label) => {
                    start = body
                        .iter()
                        .position(|s| matches!(&s.kind, StmtKind::Label(l) if l.eq_ignore_ascii_case(&label)))
                        .ok_or_else(|| RuntimeError::new(1003, format!("Label not defined: {label}")))?;
                }
                _ => return Ok(()),
            }
        }
    }

    fn exec_block(&mut self, body: &'p [Stmt], f: &mut Frame) -> Result<Flow, RuntimeError> {
        self.exec_from(body, 0, f)
    }

    fn exec_from(&mut self, body: &'p [Stmt], start: usize, f: &mut Frame) -> Result<Flow, RuntimeError> {
        for s in body.iter().skip(start) {
            match self.exec(s, f) {
                Ok(Flow::Normal) => {}
                Ok(flow) => return Ok(flow),
                Err(mut e) => {
                    if e.line.is_none() && e.number != END_SIGNAL {
                        e.line = Some(s.line);
                        e.procedure = Some(f.proc_name.clone());
                    }
                    if !e.trappable || e.number == END_SIGNAL {
                        return Err(e);
                    }
                    match &f.on_error {
                        OnError::ResumeNext => {
                            self.err = (e.number, e.description);
                        }
                        OnError::GoTo(label) if !f.in_handler => {
                            self.err = (e.number, e.description);
                            f.in_handler = true;
                            return Ok(Flow::GoTo(label.clone()));
                        }
                        _ => return Err(e),
                    }
                }
            }
        }
        Ok(Flow::Normal)
    }

    fn exec(&mut self, s: &'p Stmt, f: &mut Frame) -> Result<Flow, RuntimeError> {
        self.tick()?;
        match &s.kind {
            StmtKind::Dim(vars) => {
                for d in vars {
                    let k = key(&d.name);
                    if !f.vars.contains_key(&k) {
                        let v = self.new_var(d, f)?;
                        f.vars.insert(k, v);
                    }
                }
            }
            StmtKind::ReDim { preserve, vars } => {
                for d in vars {
                    self.redim(d, *preserve, f)?;
                }
            }
            StmtKind::Const(list) => {
                for (name, e) in list {
                    let v = self.eval_value(e, f)?;
                    f.vars.insert(key(name), Var { value: v, ty: None, constant: true });
                }
            }
            StmtKind::Assign { target, value, set } => self.assign(target, value, *set, f)?,
            StmtKind::Call(e) => {
                self.call_stmt(e, f)?;
            }
            StmtKind::If { arms, otherwise } => {
                for (cond, body) in arms {
                    if self.condition(cond, f)? {
                        return self.exec_block(body, f);
                    }
                }
                if let Some(body) = otherwise {
                    return self.exec_block(body, f);
                }
            }
            StmtKind::For { var, from, to, step, body } => return self.for_loop(var, from, to, step.as_ref(), body, f),
            StmtKind::ForEach { var, collection, body } => return self.for_each(var, collection, body, f),
            StmtKind::Do { cond, body } => loop {
                self.tick()?;
                if let Some(c) = cond
                    && c.pre
                    && self.condition(&c.expr, f)? != c.is_while
                {
                    return Ok(Flow::Normal);
                }
                match self.exec_block(body, f)? {
                    Flow::Normal => {}
                    Flow::ExitDo => return Ok(Flow::Normal),
                    other => return Ok(other),
                }
                if let Some(c) = cond
                    && !c.pre
                    && self.condition(&c.expr, f)? != c.is_while
                {
                    return Ok(Flow::Normal);
                }
            },
            StmtKind::While { cond, body } => {
                while self.condition(cond, f)? {
                    self.tick()?;
                    match self.exec_block(body, f)? {
                        Flow::Normal => {}
                        other => return Ok(other),
                    }
                }
            }
            StmtKind::Select { subject, cases, otherwise } => {
                let v = self.eval_value(subject, f)?;
                for (tests, body) in cases {
                    for t in tests {
                        let hit = match t {
                            CaseTest::Value(e) => {
                                let x = self.eval_value(e, f)?;
                                compare(&v, &x)? == Ordering::Equal
                            }
                            CaseTest::Range(a, b) => {
                                let lo = self.eval_value(a, f)?;
                                let hi = self.eval_value(b, f)?;
                                compare(&v, &lo)? != Ordering::Less && compare(&v, &hi)? != Ordering::Greater
                            }
                            CaseTest::Is(op, e) => {
                                let x = self.eval_value(e, f)?;
                                self.binop(*op, v.clone(), x)?.to_bool()?
                            }
                        };
                        if hit {
                            return self.exec_block(body, f);
                        }
                    }
                }
                if let Some(body) = otherwise {
                    return self.exec_block(body, f);
                }
            }
            StmtKind::With { object, body } => {
                let obj = self.eval(object, f)?;
                f.with_stack.push(obj);
                let r = self.exec_block(body, f);
                f.with_stack.pop();
                return r;
            }
            StmtKind::Exit(k) => {
                return Ok(match k {
                    ExitKind::Sub | ExitKind::Function => Flow::ExitProc,
                    ExitKind::For => Flow::ExitFor,
                    ExitKind::Do => Flow::ExitDo,
                });
            }
            StmtKind::OnError(mode) => {
                f.on_error = mode.clone();
                if *mode == OnError::GoToZero {
                    f.in_handler = false;
                }
                self.err = (0, String::new());
            }
            StmtKind::Label(_) => {}
            StmtKind::GoTo(l) => return Ok(Flow::GoTo(l.clone())),
            StmtKind::ResumeNext => return Err(RuntimeError::not_supported("Resume Next inside an On Error GoTo handler")),
            StmtKind::End => return Err(RuntimeError::fatal(END_SIGNAL, "End")),
            StmtKind::DebugPrint(items) => {
                let mut line = String::new();
                for (e, sep) in items {
                    let v = self.eval_value(e, f)?;
                    line.push_str(&builtins::print_text(&v));
                    if *sep == ',' {
                        line.push('\t');
                    }
                }
                self.host.debug_print(&line);
            }
            StmtKind::Erase(names) => {
                for n in names {
                    let k = key(n);
                    let var = lookup_mut(&mut self.globals, f, &k).ok_or_else(|| RuntimeError::variable_not_defined(n))?;
                    match &mut var.value {
                        Variant::Array(a) => {
                            let fill = default_for(a.elem.as_ref());
                            a.data.iter_mut().for_each(|x| *x = fill.clone());
                        }
                        _ => return Err(RuntimeError::type_mismatch()),
                    }
                }
            }
            StmtKind::Unsupported(what) => return Err(RuntimeError::not_supported(what)),
        }
        Ok(Flow::Normal)
    }

    fn for_loop(
        &mut self,
        var: &str,
        from: &'p Expr,
        to: &'p Expr,
        step: Option<&'p Expr>,
        body: &'p [Stmt],
        f: &mut Frame,
    ) -> Result<Flow, RuntimeError> {
        let start = self.eval_value(from, f)?;
        let end = self.eval_value(to, f)?.to_f64()?;
        let step = match step {
            Some(e) => self.eval_value(e, f)?,
            None => Variant::Integer(1),
        };
        let up = step.to_f64()? >= 0.0;
        self.set_var(var, start, f)?;
        loop {
            self.tick()?;
            let cur = self.get_var(var, f)?.to_f64()?;
            if (up && cur > end) || (!up && cur < end) {
                return Ok(Flow::Normal);
            }
            match self.exec_block(body, f)? {
                Flow::Normal => {}
                Flow::ExitFor => return Ok(Flow::Normal),
                other => return Ok(other),
            }
            let cur = self.get_var(var, f)?;
            let next = self.binop(BinOp::Add, cur, step.clone())?;
            self.set_var(var, next, f)?;
        }
    }

    fn for_each(&mut self, var: &str, collection: &'p Expr, body: &'p [Stmt], f: &mut Frame) -> Result<Flow, RuntimeError> {
        let coll = self.eval(collection, f)?;
        let items: Vec<Variant> = match coll {
            Variant::Array(a) => a.data,
            Variant::Object(Object::Range { sheet, range }) => {
                self.check_cells(range.count())?;
                range.iter().map(|c| Variant::Object(Object::Range { sheet, range: RangeRef::cell(c) })).collect()
            }
            Variant::Object(Object::Worksheets) => (0..self.host.sheet_count()).map(|i| Variant::Object(Object::Worksheet(i))).collect(),
            _ => return Err(RuntimeError::new(451, "For Each may only iterate over a collection or an array")),
        };
        for item in items {
            self.tick()?;
            self.set_var_raw(var, item, f)?;
            match self.exec_block(body, f)? {
                Flow::Normal => {}
                Flow::ExitFor => return Ok(Flow::Normal),
                other => return Ok(other),
            }
        }
        Ok(Flow::Normal)
    }

    fn redim(&mut self, d: &'p DimVar, preserve: bool, f: &mut Frame) -> Result<(), RuntimeError> {
        let b = d.bounds.as_deref().unwrap_or_default();
        let bounds = self.eval_bounds(b, f)?;
        let k = key(&d.name);
        let elem = match lookup_mut(&mut self.globals, f, &k) {
            Some(Var { value: Variant::Array(a), .. }) => a.elem.clone().or_else(|| d.ty.clone()),
            Some(_) => return Err(RuntimeError::type_mismatch()),
            None => d.ty.clone(),
        };
        let mut fresh = VbArray::new(bounds, elem, self.limits.max_array_elements)?;
        if preserve && let Some(Var { value: Variant::Array(old), .. }) = lookup_mut(&mut self.globals, f, &k) {
            if old.bounds.len() == fresh.bounds.len() && old.bounds.len() == 1 {
                for (i, v) in old.data.iter().enumerate() {
                    let idx = old.bounds.first().map_or(0, |b| b.0) + i as i64;
                    if let Ok(o) = fresh.offset(&[idx])
                        && let Some(slot) = fresh.data.get_mut(o)
                    {
                        *slot = v.clone();
                    }
                }
            } else if !old.bounds.is_empty() {
                return Err(RuntimeError::not_supported("ReDim Preserve on multi-dimensional arrays"));
            }
        }
        match lookup_mut(&mut self.globals, f, &k) {
            Some(var) => var.value = Variant::Array(Box::new(fresh)),
            None => {
                f.vars.insert(k, Var { value: Variant::Array(Box::new(fresh)), ty: None, constant: false });
            }
        }
        Ok(())
    }

    // ---- variables -----------------------------------------------------------

    fn get_var(&mut self, name: &str, f: &mut Frame) -> Result<Variant, RuntimeError> {
        let k = key(name);
        match lookup_mut(&mut self.globals, f, &k) {
            Some(v) => Ok(v.value.clone()),
            None => Err(RuntimeError::variable_not_defined(name)),
        }
    }

    /// Assigns with the variable's declared type; creates an implicit
    /// Variant when `Option Explicit` is off.
    fn set_var(&mut self, name: &str, value: Variant, f: &mut Frame) -> Result<(), RuntimeError> {
        let k = key(name);
        let explicit = self.module(f).is_some_and(|m| m.option_explicit);
        match lookup_mut(&mut self.globals, f, &k) {
            Some(var) if var.constant => Err(RuntimeError::new(1005, format!("Assignment to constant not permitted: {name}"))),
            Some(var) => {
                if var.ty.as_ref().is_some_and(TypeName::is_object) && !matches!(value, Variant::Object(_) | Variant::Nothing) {
                    return Err(RuntimeError::object_not_set());
                }
                var.value = value.coerce(var.ty.as_ref())?;
                Ok(())
            }
            None if explicit => Err(RuntimeError::variable_not_defined(name)),
            None => {
                f.vars.insert(k, Var { value, ty: None, constant: false });
                Ok(())
            }
        }
    }

    /// Assignment that keeps objects as objects (`Set`, `For Each`).
    fn set_var_raw(&mut self, name: &str, value: Variant, f: &mut Frame) -> Result<(), RuntimeError> {
        let k = key(name);
        match lookup_mut(&mut self.globals, f, &k) {
            Some(var) if matches!(value, Variant::Object(_) | Variant::Nothing) => {
                if var.ty.as_ref().is_some_and(|t| !t.is_object() && *t != TypeName::Variant) {
                    return Err(RuntimeError::type_mismatch());
                }
                var.value = value;
                Ok(())
            }
            Some(_) => self.set_var(name, value, f),
            None => self.set_var(name, value, f),
        }
    }

    fn assign(&mut self, target: &'p Expr, value: &'p Expr, set: bool, f: &mut Frame) -> Result<(), RuntimeError> {
        let v = if set {
            match self.eval(value, f)? {
                v @ (Variant::Object(_) | Variant::Nothing) => v,
                _ => return Err(RuntimeError::object_required()),
            }
        } else {
            let raw = self.eval(value, f)?;
            self.deref(raw)?
        };
        match target {
            Expr::Ident(name) => {
                if set {
                    self.set_var_raw(name, v, f)
                } else {
                    // Assigning a value to an object variable sets its default property.
                    if let Ok(Variant::Object(o)) = self.get_var(name, f) {
                        return self.set_member(o, "value", v);
                    }
                    self.set_var(name, v, f)
                }
            }
            Expr::Member { object, name } => {
                let obj = self.object_of(object.as_deref(), f)?;
                self.set_member(obj, name, v)
            }
            Expr::Call { callee, args } => {
                // Array element?
                if let Expr::Ident(name) = callee.as_ref() {
                    let k = key(name);
                    let is_array = matches!(lookup_mut(&mut self.globals, f, &k), Some(Var { value: Variant::Array(_), .. }));
                    if is_array {
                        let idx = self.indices(args, f)?;
                        let var = lookup_mut(&mut self.globals, f, &k).ok_or_else(RuntimeError::subscript)?;
                        let Variant::Array(a) = &mut var.value else { return Err(RuntimeError::type_mismatch()) };
                        let off = a.offset(&idx)?;
                        let coerced = v.coerce(a.elem.as_ref())?;
                        if let Some(slot) = a.data.get_mut(off) {
                            *slot = coerced;
                        }
                        return Ok(());
                    }
                }
                // Otherwise an object expression: set its default property.
                match self.eval(target, f)? {
                    Variant::Object(o) => self.set_member(o, "value", v),
                    _ => Err(RuntimeError::object_required()),
                }
            }
            _ => Err(RuntimeError::new(1006, "invalid assignment target")),
        }
    }

    fn indices(&mut self, args: &'p [Arg], f: &mut Frame) -> Result<Vec<i64>, RuntimeError> {
        let mut out = Vec::new();
        for a in args {
            let e = a.value.as_ref().ok_or_else(RuntimeError::subscript)?;
            out.push(self.eval_value(e, f)?.to_i64()?);
        }
        Ok(out)
    }

    // ---- expressions -----------------------------------------------------------

    fn condition(&mut self, e: &'p Expr, f: &mut Frame) -> Result<bool, RuntimeError> {
        match self.eval_value(e, f)? {
            Variant::Null => Err(RuntimeError::invalid_null()),
            v => v.to_bool(),
        }
    }

    /// Evaluates to a plain value: objects are replaced by their default property.
    fn eval_value(&mut self, e: &'p Expr, f: &mut Frame) -> Result<Variant, RuntimeError> {
        let v = self.eval(e, f)?;
        self.deref(v)
    }

    fn deref(&mut self, v: Variant) -> Result<Variant, RuntimeError> {
        match v {
            Variant::Object(o) => self.member(o, "value", Vec::new()),
            Variant::Nothing => Err(RuntimeError::object_not_set()),
            other => Ok(other),
        }
    }

    fn object_of(&mut self, object: Option<&'p Expr>, f: &mut Frame) -> Result<Object, RuntimeError> {
        let v = match object {
            Some(e) => self.eval(e, f)?,
            None => f.with_stack.last().cloned().ok_or_else(|| RuntimeError::new(1007, "'.' used outside a With block"))?,
        };
        match v {
            Variant::Object(o) => Ok(o),
            Variant::Nothing => Err(RuntimeError::object_not_set()),
            _ => Err(RuntimeError::object_required()),
        }
    }

    fn eval(&mut self, e: &'p Expr, f: &mut Frame) -> Result<Variant, RuntimeError> {
        match e {
            Expr::Literal(v) => Ok(v.clone()),
            Expr::Nothing => Ok(Variant::Nothing),
            Expr::New(c) => Err(RuntimeError::not_supported(&format!("New {c}"))),
            Expr::Ident(name) => self.ident(name, f),
            Expr::Member { object, name } => {
                let obj = self.object_of(object.as_deref(), f)?;
                self.member(obj, name, Vec::new())
            }
            Expr::Call { callee, args } => self.call_expr(callee, args, f),
            Expr::Unary(op, inner) => {
                let v = self.eval_value(inner, f)?;
                unop(*op, v)
            }
            Expr::Binary(BinOp::Is, a, b) => {
                let x = self.eval(a, f)?;
                let y = self.eval(b, f)?;
                match (&x, &y) {
                    (Variant::Object(_) | Variant::Nothing, Variant::Object(_) | Variant::Nothing) => Ok(Variant::Boolean(x == y)),
                    _ => Err(RuntimeError::object_required()),
                }
            }
            Expr::Binary(op, a, b) => {
                let x = self.eval_value(a, f)?;
                let y = self.eval_value(b, f)?;
                self.binop(*op, x, y)
            }
        }
    }

    fn ident(&mut self, name: &'p str, f: &mut Frame) -> Result<Variant, RuntimeError> {
        let k = key(name);
        if let Some(v) = lookup_mut(&mut self.globals, f, &k) {
            return Ok(v.value.clone());
        }
        if let Some(v) = self.global_object(&k) {
            return Ok(v);
        }
        if let Some((_, n)) = CONSTANTS.iter().find(|(c, _)| *c == k) {
            return Ok(Variant::from_int(*n));
        }
        if let Some((_, s)) = STRING_CONSTANTS.iter().find(|(c, _)| *c == k) {
            return Ok(Variant::String((*s).to_string()));
        }
        let prog: &'p Program = self.prog;
        if let Some((mi, p)) = prog.find(name, f.module) {
            return self.call_proc(mi, p, &[], Some(f));
        }
        if let Some(r) = self.call_named(&k, Vec::new(), f) {
            return r;
        }
        if self.module(f).is_some_and(|m| m.option_explicit) {
            return Err(RuntimeError::variable_not_defined(name));
        }
        Ok(Variant::Empty)
    }

    fn global_object(&self, k: &str) -> Option<Variant> {
        Some(Variant::Object(match k {
            "application" => Object::Application,
            "activeworkbook" | "thisworkbook" => Object::Workbook,
            "worksheets" | "sheets" => Object::Worksheets,
            "activesheet" => Object::Worksheet(self.host.active_sheet()),
            "debug" => Object::Debug,
            "err" => Object::Err,
            "worksheetfunction" => Object::WorksheetFunction,
            _ => return None,
        }))
    }

    fn eval_args(&mut self, args: &'p [Arg], f: &mut Frame) -> Result<Vec<Option<Variant>>, RuntimeError> {
        let mut out = Vec::new();
        for a in args {
            if a.name.is_some() {
                return Err(RuntimeError::not_supported("named arguments to built-in functions"));
            }
            out.push(match &a.value {
                Some(e) => Some(self.eval(e, f)?),
                None => None,
            });
        }
        Ok(out)
    }

    fn call_expr(&mut self, callee: &'p Expr, args: &'p [Arg], f: &mut Frame) -> Result<Variant, RuntimeError> {
        match callee {
            Expr::Ident(name) => {
                let k = key(name);
                if let Some(var) = lookup_mut(&mut self.globals, f, &k) {
                    match &var.value {
                        Variant::Array(_) => {
                            let idx = self.indices(args, f)?;
                            let var = lookup_mut(&mut self.globals, f, &k).ok_or_else(RuntimeError::subscript)?;
                            let Variant::Array(a) = &var.value else { return Err(RuntimeError::type_mismatch()) };
                            let off = a.offset(&idx)?;
                            return a.data.get(off).cloned().ok_or_else(RuntimeError::subscript);
                        }
                        Variant::Object(o) => {
                            let o = o.clone();
                            let vals = self.eval_args(args, f)?;
                            return self.member(o, "_default", vals);
                        }
                        _ => {}
                    }
                }
                let prog: &'p Program = self.prog;
                if let Some((mi, p)) = prog.find(name, f.module) {
                    return self.call_proc(mi, p, args, Some(f));
                }
                if self.module(f).is_some_and(|m| m.external_declares.iter().any(|d| d.eq_ignore_ascii_case(name))) {
                    return Err(RuntimeError::permission_denied(&format!("the Windows API function {name}")));
                }
                if let Some(Variant::Object(o)) = self.global_object(&k) {
                    let vals = self.eval_args(args, f)?;
                    return self.member(o, "_default", vals);
                }
                let vals = self.eval_args(args, f)?;
                match self.call_named(&k, vals, f) {
                    Some(r) => r,
                    None => Err(RuntimeError::not_defined(name)),
                }
            }
            Expr::Member { object, name } => {
                let obj = self.object_of(object.as_deref(), f)?;
                let vals = self.eval_args(args, f)?;
                self.member(obj, name, vals)
            }
            other => {
                let v = self.eval(other, f)?;
                match v {
                    Variant::Array(a) => {
                        let idx = self.indices(args, f)?;
                        let off = a.offset(&idx)?;
                        a.data.get(off).cloned().ok_or_else(RuntimeError::subscript)
                    }
                    Variant::Object(o) => {
                        let vals = self.eval_args(args, f)?;
                        self.member(o, "_default", vals)
                    }
                    _ => Err(RuntimeError::type_mismatch()),
                }
            }
        }
    }

    fn call_stmt(&mut self, e: &'p Expr, f: &mut Frame) -> Result<(), RuntimeError> {
        // A bare object reference as a statement (e.g. `Range("A1").Select` parses as Member) is evaluated
        // for its side effects; values are discarded.
        self.eval(e, f).map(|_| ())
    }

    /// Global functions: `Range`, `Cells`, `MsgBox` and the pure built-ins.
    fn call_named(&mut self, k: &str, args: Vec<Option<Variant>>, f: &mut Frame) -> Option<Result<Variant, RuntimeError>> {
        let active = self.host.active_sheet();
        Some(match k {
            "range" => self.range_fn(active, &args),
            "cells" => self.cells_fn(active, None, &args),
            "msgbox" => (|| {
                let prompt = match args.first().cloned().flatten() {
                    Some(v) => self.deref(v)?.to_str()?,
                    None => return Err(RuntimeError::arg_not_optional()),
                };
                let buttons = match args.get(1).cloned().flatten() {
                    Some(v) => self.deref(v)?.to_i64()?,
                    None => 0,
                };
                let title = match args.get(2).cloned().flatten() {
                    Some(v) => self.deref(v)?.to_str()?,
                    None => String::new(),
                };
                Ok(Variant::from_int(self.host.msg_box(&prompt, buttons, &title)))
            })(),
            _ if builtins::DENIED.contains(&k) => Err(RuntimeError::permission_denied(k)),
            _ if builtins::NOT_YET.contains(&k) => Err(RuntimeError::not_supported(&format!("the {k} function"))),
            _ => {
                let raw_ok = matches!(k, "typename" | "isobject" | "ismissing");
                let mut vals = Vec::with_capacity(args.len());
                for a in args {
                    vals.push(match a {
                        Some(v) if !raw_ok => match self.deref(v) {
                            Ok(v) => Some(v),
                            Err(e) => return Some(Err(e)),
                        },
                        other => other,
                    });
                }
                let base = self.option_base(f);
                return builtins::call(k, &vals, base, self.limits.max_string_len);
            }
        })
    }

    // ---- object model ------------------------------------------------------------

    fn check_cells(&self, n: u64) -> Result<(), RuntimeError> {
        if n > self.limits.max_range_cells {
            Err(RuntimeError::fatal(7, format!("Out of memory: the macro touched {n} cells at once, above the limit")))
        } else {
            Ok(())
        }
    }

    fn sheet_by(&mut self, v: &Variant) -> Result<usize, RuntimeError> {
        match v {
            Variant::String(name) => self.host.sheet_index(name).ok_or_else(RuntimeError::subscript),
            other => {
                let i = other.to_i64()?;
                let idx = usize::try_from(i - 1).map_err(|_| RuntimeError::subscript())?;
                if idx < self.host.sheet_count() { Ok(idx) } else { Err(RuntimeError::subscript()) }
            }
        }
    }

    fn parse_range(&mut self, default_sheet: usize, text: &str) -> Result<(usize, RangeRef), RuntimeError> {
        let (sheet, addr) = match text.rsplit_once('!') {
            Some((s, a)) => {
                let name = s.trim_matches('\'');
                (self.host.sheet_index(name).ok_or_else(RuntimeError::subscript)?, a)
            }
            None => (default_sheet, text),
        };
        let clean: String = addr.chars().filter(|c| *c != '$').collect();
        let range = RangeRef::parse(&clean).ok_or_else(|| {
            RuntimeError::excel(format!("Method 'Range' failed: {text:?} is not a cell reference (named ranges are not supported yet)"))
        })?;
        Ok((sheet, range))
    }

    fn range_fn(&mut self, sheet: usize, args: &[Option<Variant>]) -> Result<Variant, RuntimeError> {
        let first = args.first().cloned().flatten().ok_or_else(RuntimeError::arg_not_optional)?;
        let (s1, r1) = match first {
            Variant::Object(Object::Range { sheet, range }) => (sheet, range),
            v => {
                let t = self.deref(v)?.to_str()?;
                self.parse_range(sheet, &t)?
            }
        };
        let range = match args.get(1).cloned().flatten() {
            None => r1,
            Some(Variant::Object(Object::Range { range, .. })) => r1.union(&range),
            Some(v) => {
                let t = self.deref(v)?.to_str()?;
                r1.union(&self.parse_range(s1, &t)?.1)
            }
        };
        Ok(Variant::Object(Object::Range { sheet: s1, range }))
    }

    /// `Cells(row, col)`; relative to `origin` when called on a range.
    fn cells_fn(&mut self, sheet: usize, origin: Option<CellRef>, args: &[Option<Variant>]) -> Result<Variant, RuntimeError> {
        let o = origin.unwrap_or(CellRef { row: 0, col: 0 });
        match (args.first().cloned().flatten(), args.get(1).cloned().flatten()) {
            (None, None) => {
                let range = match origin {
                    Some(_) => return Err(RuntimeError::not_supported("Range.Cells without arguments")),
                    None => RangeRef::all(),
                };
                Ok(Variant::Object(Object::Range { sheet, range }))
            }
            (Some(r), c) => {
                let row = self.deref(r)?.to_i64()?;
                let col = match c {
                    None => 1,
                    Some(v) => match self.deref(v)? {
                        Variant::String(letters) => gridcraft_core::letters_to_col(&letters.to_ascii_uppercase())
                            .map(|c| i64::from(c) + 1)
                            .ok_or_else(RuntimeError::excel_range)?,
                        v => v.to_i64()?,
                    },
                };
                let r = i64::from(o.row) + row - 1;
                let c = i64::from(o.col) + col - 1;
                if r < 0 || c < 0 || r >= i64::from(MAX_ROWS) || c >= i64::from(MAX_COLS) {
                    return Err(RuntimeError::excel_range());
                }
                let at = CellRef { row: u32::try_from(r).unwrap_or(0), col: u32::try_from(c).unwrap_or(0) };
                Ok(Variant::Object(Object::Range { sheet, range: RangeRef::cell(at) }))
            }
            (None, Some(_)) => Err(RuntimeError::arg_not_optional()),
        }
    }

    fn read_range(&mut self, sheet: usize, range: RangeRef, formulas: bool) -> Result<Variant, RuntimeError> {
        let read = |h: &dyn Host, c: CellRef| -> Variant {
            if formulas {
                match h.formula(sheet, c) {
                    Some(t) => Variant::String(t),
                    None => match cell_to_variant(&h.value(sheet, c)) {
                        Variant::Empty => Variant::String(String::new()),
                        v => Variant::String(v.to_str().unwrap_or_default()),
                    },
                }
            } else {
                cell_to_variant(&h.value(sheet, c))
            }
        };
        if range.is_single() {
            return Ok(read(&*self.host, range.start));
        }
        self.check_cells(range.count())?;
        let mut a = VbArray::new(vec![(1, i64::from(range.height())), (1, i64::from(range.width()))], None, self.limits.max_array_elements)?;
        for (i, c) in range.iter().enumerate() {
            if let Some(slot) = a.data.get_mut(i) {
                *slot = read(&*self.host, c);
            }
        }
        Ok(Variant::Array(Box::new(a)))
    }

    fn cell_input(&mut self, v: Variant) -> Result<Input, RuntimeError> {
        Ok(match self.deref(v)? {
            Variant::Empty | Variant::Null => Input::Empty,
            Variant::Boolean(b) => Input::Bool(b),
            Variant::String(s) => Input::Text(s),
            Variant::CellError(e) => Input::Text(e),
            v @ (Variant::Byte(_) | Variant::Integer(_) | Variant::Long(_) | Variant::LongLong(_) | Variant::Double(_)) => Input::Number(v.to_f64()?),
            Variant::Array(_) | Variant::Object(_) | Variant::Nothing => return Err(RuntimeError::type_mismatch()),
        })
    }

    fn write_range(&mut self, sheet: usize, range: RangeRef, v: Variant) -> Result<(), RuntimeError> {
        self.check_cells(range.count())?;
        match v {
            Variant::Array(a) => {
                let (rows, cols) = match a.bounds.as_slice() {
                    [(lo, hi)] => (vec![None], (*lo..=*hi).collect::<Vec<_>>()),
                    [(rl, rh), (cl, ch)] => ((*rl..=*rh).map(Some).collect(), (*cl..=*ch).collect()),
                    _ => return Err(RuntimeError::type_mismatch()),
                };
                for c in range.iter() {
                    let r = (c.row - range.start.row) as usize;
                    let k = (c.col - range.start.col) as usize;
                    let item = match (rows.get(r), cols.get(k)) {
                        (Some(Some(ri)), Some(ci)) => a.offset(&[*ri, *ci]).ok().and_then(|o| a.data.get(o).cloned()),
                        (Some(None), Some(ci)) => a.offset(&[*ci]).ok().and_then(|o| a.data.get(o).cloned()),
                        _ => None,
                    };
                    let input = match item {
                        Some(x) => self.cell_input(x)?,
                        None => Input::Text("#N/A".into()),
                    };
                    self.host.set_input(sheet, c, input).map_err(RuntimeError::excel)?;
                }
            }
            other => {
                let input = self.cell_input(other)?;
                for c in range.iter() {
                    self.tick()?;
                    self.host.set_input(sheet, c, input.clone()).map_err(RuntimeError::excel)?;
                }
            }
        }
        Ok(())
    }

    fn arg_value(&mut self, args: &[Option<Variant>], i: usize) -> Result<Variant, RuntimeError> {
        let v = args.get(i).cloned().flatten().ok_or_else(RuntimeError::arg_not_optional)?;
        self.deref(v)
    }

    /// Reads a property or calls a method on an object.
    fn member(&mut self, obj: Object, name: &str, args: Vec<Option<Variant>>) -> Result<Variant, RuntimeError> {
        let m = name.to_ascii_lowercase();
        let ok = |v: Variant| Ok(v);
        match obj {
            Object::Range { sheet, range } => match m.as_str() {
                "value" | "value2" | "_default" if args.is_empty() => self.read_range(sheet, range, false),
                "_default" | "item" | "cells" => self.cells_fn(sheet, Some(range.start), &args),
                "formula" => self.read_range(sheet, range, true),
                "text" => {
                    let v = self.read_range(sheet, range, false)?;
                    match v {
                        Variant::Array(_) => Err(RuntimeError::not_supported("Text on a multi-cell range")),
                        v => ok(Variant::String(builtins::print_text(&v))),
                    }
                }
                "address" => ok(Variant::String(address(&range))),
                "row" => ok(Variant::Long(i32::try_from(range.start.row + 1).unwrap_or(i32::MAX))),
                "column" => ok(Variant::Long(i32::try_from(range.start.col + 1).unwrap_or(i32::MAX))),
                "count" | "countlarge" => ok(Variant::Double(range.count() as f64)),
                "offset" | "resize" => {
                    let a = match args.first().cloned().flatten() {
                        Some(v) => self.deref(v)?.to_i64()?,
                        None => {
                            if m == "offset" {
                                0
                            } else {
                                i64::from(range.height())
                            }
                        }
                    };
                    let b = match args.get(1).cloned().flatten() {
                        Some(v) => self.deref(v)?.to_i64()?,
                        None => {
                            if m == "offset" {
                                0
                            } else {
                                i64::from(range.width())
                            }
                        }
                    };
                    let (r0, c0, h, w) = if m == "offset" {
                        (i64::from(range.start.row) + a, i64::from(range.start.col) + b, i64::from(range.height()), i64::from(range.width()))
                    } else {
                        (i64::from(range.start.row), i64::from(range.start.col), a, b)
                    };
                    if r0 < 0 || c0 < 0 || h < 1 || w < 1 || r0 + h > i64::from(MAX_ROWS) || c0 + w > i64::from(MAX_COLS) {
                        return Err(RuntimeError::excel_range());
                    }
                    let start = CellRef { row: r0 as u32, col: c0 as u32 };
                    let end = CellRef { row: (r0 + h - 1) as u32, col: (c0 + w - 1) as u32 };
                    ok(Variant::Object(Object::Range { sheet, range: RangeRef::new(start, end) }))
                }
                "clearcontents" | "clear" => {
                    self.write_range(sheet, range, Variant::Empty)?;
                    ok(Variant::Empty)
                }
                "worksheet" | "parent" => ok(Variant::Object(Object::Worksheet(sheet))),
                "select" | "activate" => {
                    if sheet != self.host.active_sheet() {
                        self.host.activate_sheet(sheet).map_err(RuntimeError::excel)?;
                    }
                    ok(Variant::Boolean(true))
                }
                "range" => {
                    let inner = self.range_fn(sheet, &args)?;
                    match inner {
                        Variant::Object(Object::Range { range: r, .. }) => {
                            let start = CellRef { row: range.start.row + r.start.row, col: range.start.col + r.start.col };
                            let end = CellRef { row: range.start.row + r.end.row, col: range.start.col + r.end.col };
                            ok(Variant::Object(Object::Range { sheet, range: RangeRef::new(start, end) }))
                        }
                        other => ok(other),
                    }
                }
                _ => Err(RuntimeError::unsupported_member(name)),
            },
            Object::Worksheet(i) => match m.as_str() {
                "name" => ok(Variant::String(self.host.sheet_name(i).unwrap_or_default())),
                "index" => ok(Variant::from_int(i as i64 + 1)),
                "range" => self.range_fn(i, &args),
                "cells" => self.cells_fn(i, None, &args),
                "activate" | "select" => {
                    self.host.activate_sheet(i).map_err(RuntimeError::excel)?;
                    ok(Variant::Empty)
                }
                "value" | "_default" if args.is_empty() => Err(RuntimeError::type_mismatch()),
                _ => Err(RuntimeError::unsupported_member(name)),
            },
            Object::Worksheets => match m.as_str() {
                "count" => ok(Variant::from_int(self.host.sheet_count() as i64)),
                "item" | "_default" if !args.is_empty() => {
                    let v = self.arg_value(&args, 0)?;
                    ok(Variant::Object(Object::Worksheet(self.sheet_by(&v)?)))
                }
                "value" | "_default" => Err(RuntimeError::type_mismatch()),
                _ => Err(RuntimeError::unsupported_member(name)),
            },
            Object::Workbook | Object::Application => match m.as_str() {
                "worksheets" | "sheets" if args.is_empty() => ok(Variant::Object(Object::Worksheets)),
                "worksheets" | "sheets" => self.member(Object::Worksheets, "item", args),
                "activesheet" => ok(Variant::Object(Object::Worksheet(self.host.active_sheet()))),
                "activeworkbook" | "thisworkbook" => ok(Variant::Object(Object::Workbook)),
                "range" => {
                    let active = self.host.active_sheet();
                    self.range_fn(active, &args)
                }
                "cells" => {
                    let active = self.host.active_sheet();
                    self.cells_fn(active, None, &args)
                }
                "worksheetfunction" => ok(Variant::Object(Object::WorksheetFunction)),
                "screenupdating" | "displayalerts" | "enableevents" => ok(Variant::Boolean(true)),
                "calculate" => ok(Variant::Empty),
                "name" | "value" | "_default" if obj_is_app(&obj) => ok(Variant::String("Cellnux".into())),
                _ => Err(RuntimeError::unsupported_member(name)),
            },
            Object::Debug => match m.as_str() {
                "print" => {
                    let mut line = String::new();
                    for a in args.into_iter().flatten() {
                        let v = self.deref(a)?;
                        line.push_str(&builtins::print_text(&v));
                    }
                    self.host.debug_print(&line);
                    ok(Variant::Empty)
                }
                _ => Err(RuntimeError::unsupported_member(name)),
            },
            Object::Err => match m.as_str() {
                "number" | "value" | "_default" => ok(Variant::Long(self.err.0)),
                "description" => ok(Variant::String(self.err.1.clone())),
                "source" => ok(Variant::String("Cellnux".into())),
                "clear" => {
                    self.err = (0, String::new());
                    ok(Variant::Empty)
                }
                "raise" => {
                    let n = self.arg_value(&args, 0)?.to_i32()?;
                    let desc = match args.get(2).cloned().flatten() {
                        Some(v) => self.deref(v)?.to_str()?,
                        None => format!("Application-defined or object-defined error {n}"),
                    };
                    Err(RuntimeError::new(n, desc))
                }
                _ => Err(RuntimeError::unsupported_member(name)),
            },
            Object::WorksheetFunction => self.worksheet_function(&m, args),
        }
    }

    fn set_member(&mut self, obj: Object, name: &str, v: Variant) -> Result<(), RuntimeError> {
        let m = name.to_ascii_lowercase();
        match (&obj, m.as_str()) {
            (Object::Range { sheet, range }, "value" | "value2") => self.write_range(*sheet, *range, v),
            (Object::Range { sheet, range }, "formula") => {
                let text = self.deref(v)?.to_str()?;
                self.write_range(*sheet, *range, Variant::String(text))
            }
            (Object::Application, "screenupdating" | "displayalerts" | "enableevents" | "statusbar" | "calculation" | "cutcopymode") => Ok(()),
            (Object::Err, "number") => {
                self.err.0 = self.deref(v)?.to_i32()?;
                Ok(())
            }
            (Object::Err, "description") => {
                self.err.1 = self.deref(v)?.to_str()?;
                Ok(())
            }
            _ => Err(RuntimeError::unsupported_member(name)),
        }
    }

    /// `Application.WorksheetFunction.Sum` and friends, computed natively.
    fn worksheet_function(&mut self, m: &str, args: Vec<Option<Variant>>) -> Result<Variant, RuntimeError> {
        let mut numbers = Vec::new();
        let mut non_empty = 0usize;
        for a in args.into_iter().flatten() {
            match a {
                Variant::Object(Object::Range { sheet, range }) => {
                    self.check_cells(range.count())?;
                    for c in range.iter() {
                        match self.host.value(sheet, c) {
                            Value::Number(n) => {
                                numbers.push(n);
                                non_empty += 1;
                            }
                            Value::Empty => {}
                            Value::Error(e) if matches!(m, "sum" | "average" | "min" | "max") => {
                                return Err(RuntimeError::excel(format!("Unable to get the {m} property: the range contains {}", e.as_str())));
                            }
                            _ => non_empty += 1,
                        }
                    }
                }
                Variant::Array(arr) => {
                    for x in arr.data {
                        if let Ok(n) = x.to_f64()
                            && x.is_numeric_type()
                            && !matches!(x, Variant::Empty)
                        {
                            numbers.push(n);
                        }
                        if !matches!(x, Variant::Empty) {
                            non_empty += 1;
                        }
                    }
                }
                other => {
                    let v = self.deref(other)?;
                    numbers.push(v.to_f64()?);
                    non_empty += 1;
                }
            }
        }
        let r = match m {
            "sum" => numbers.iter().sum(),
            "average" => {
                if numbers.is_empty() {
                    return Err(RuntimeError::excel("Unable to get the Average property: division by zero"));
                }
                numbers.iter().sum::<f64>() / numbers.len() as f64
            }
            "min" => numbers.iter().copied().fold(f64::INFINITY, f64::min),
            "max" => numbers.iter().copied().fold(f64::NEG_INFINITY, f64::max),
            "count" => numbers.len() as f64,
            "counta" => non_empty as f64,
            _ => return Err(RuntimeError::not_supported(&format!("WorksheetFunction.{m}"))),
        };
        Ok(Variant::Double(if r.is_finite() { r } else { 0.0 }))
    }

    // ---- operators -------------------------------------------------------------------

    fn binop(&mut self, op: BinOp, a: Variant, b: Variant) -> Result<Variant, RuntimeError> {
        binop(op, a, b, self.limits.max_string_len)
    }
}

fn obj_is_app(o: &Object) -> bool {
    matches!(o, Object::Application)
}

fn is_known_class(c: &str) -> bool {
    matches!(c.to_ascii_lowercase().as_str(), "range" | "worksheet" | "workbook" | "application" | "sheets" | "worksheets" | "collection")
}

impl RuntimeError {
    fn excel_range() -> RuntimeError {
        RuntimeError::excel("Application-defined or object-defined error: the cell is outside the sheet")
    }
}

/// Numeric rank used to choose an arithmetic result type.
fn rank(v: &Variant) -> u8 {
    match v {
        Variant::Byte(_) => 0,
        Variant::Empty | Variant::Boolean(_) | Variant::Integer(_) => 1,
        Variant::Long(_) => 2,
        Variant::LongLong(_) => 3,
        _ => 4,
    }
}

fn int_result(n: i64, r: u8) -> Result<Variant, RuntimeError> {
    let fits = match r {
        0 => u8::try_from(n).map(Variant::Byte).ok(),
        1 => i16::try_from(n).map(Variant::Integer).ok(),
        2 => i32::try_from(n).map(Variant::Long).ok(),
        _ => Some(Variant::LongLong(n)),
    };
    fits.ok_or_else(RuntimeError::overflow)
}

fn numeric_operand(v: Variant) -> Result<Variant, RuntimeError> {
    match v {
        Variant::String(s) => parse_numeric(&s).map(Variant::Double).ok_or_else(RuntimeError::type_mismatch),
        Variant::Boolean(b) => Ok(Variant::Integer(if b { -1 } else { 0 })),
        Variant::Empty => Ok(Variant::Integer(0)),
        v @ (Variant::Byte(_) | Variant::Integer(_) | Variant::Long(_) | Variant::LongLong(_) | Variant::Double(_)) => Ok(v),
        Variant::Null => Err(RuntimeError::invalid_null()),
        _ => Err(RuntimeError::type_mismatch()),
    }
}

/// Compares two values as `=`, `<` and friends do.
fn compare(a: &Variant, b: &Variant) -> Result<Ordering, RuntimeError> {
    use Variant as V;
    let is_num = |v: &V| v.is_numeric_type();
    match (a, b) {
        (V::Null, _) | (_, V::Null) => Err(RuntimeError::invalid_null()),
        (V::String(x), V::String(y)) => Ok(x.cmp(y)),
        (V::String(x), V::Empty) => Ok(x.as_str().cmp("")),
        (V::Empty, V::String(y)) => Ok("".cmp(y.as_str())),
        (V::String(s), n) if is_num(n) => match parse_numeric(s) {
            Some(x) => Ok(x.partial_cmp(&n.to_f64()?).unwrap_or(Ordering::Equal)),
            None => Ok(Ordering::Greater),
        },
        (n, V::String(s)) if is_num(n) => match parse_numeric(s) {
            Some(y) => Ok(n.to_f64()?.partial_cmp(&y).unwrap_or(Ordering::Equal)),
            None => Ok(Ordering::Less),
        },
        (x, y) if is_num(x) && is_num(y) => Ok(x.to_f64()?.partial_cmp(&y.to_f64()?).unwrap_or(Ordering::Equal)),
        (V::CellError(x), V::CellError(y)) => Ok(x.cmp(y)),
        _ => Err(RuntimeError::type_mismatch()),
    }
}

fn unop(op: UnOp, v: Variant) -> Result<Variant, RuntimeError> {
    if v == Variant::Null {
        return Ok(Variant::Null);
    }
    match op {
        UnOp::Not => match v {
            Variant::Boolean(b) => Ok(Variant::Boolean(!b)),
            other => {
                let n = numeric_operand(other)?;
                let r = rank(&n).max(1);
                if r == 4 { int_result(!n.to_i64()?, 2) } else { int_result(!n.to_i64()?, r) }
            }
        },
        UnOp::Neg => {
            let n = numeric_operand(v)?;
            match n {
                Variant::Double(x) => Ok(Variant::Double(-x)),
                other => {
                    let r = rank(&other).max(1);
                    int_result(other.to_i64()?.checked_neg().ok_or_else(RuntimeError::overflow)?, r)
                }
            }
        }
    }
}

fn binop(op: BinOp, a: Variant, b: Variant, max_string: usize) -> Result<Variant, RuntimeError> {
    use Variant as V;
    match op {
        BinOp::Concat => {
            let x = if a == V::Null { String::new() } else { a.to_str()? };
            let y = if b == V::Null { String::new() } else { b.to_str()? };
            if x.len() + y.len() > max_string {
                return Err(RuntimeError::fatal(7, "Out of string space: the macro string limit was exceeded"));
            }
            return Ok(V::String(x + &y));
        }
        BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Gt | BinOp::Le | BinOp::Ge => {
            if a == V::Null || b == V::Null {
                return Ok(V::Null);
            }
            let o = compare(&a, &b)?;
            return Ok(V::Boolean(match op {
                BinOp::Eq => o == Ordering::Equal,
                BinOp::Ne => o != Ordering::Equal,
                BinOp::Lt => o == Ordering::Less,
                BinOp::Gt => o == Ordering::Greater,
                BinOp::Le => o != Ordering::Greater,
                _ => o != Ordering::Less,
            }));
        }
        BinOp::Like => return Ok(V::Boolean(builtins::like(&a.to_str()?, &b.to_str()?))),
        BinOp::Is => return Ok(V::Boolean(a == b)),
        _ => {}
    }
    if a == V::Null || b == V::Null {
        return Ok(V::Null);
    }
    if let (BinOp::And | BinOp::Or | BinOp::Xor | BinOp::Eqv | BinOp::Imp, V::Boolean(x), V::Boolean(y)) = (op, &a, &b) {
        let (x, y) = (*x, *y);
        return Ok(V::Boolean(match op {
            BinOp::And => x && y,
            BinOp::Or => x || y,
            BinOp::Xor => x != y,
            BinOp::Eqv => x == y,
            _ => !x || y,
        }));
    }
    if op == BinOp::Add && matches!((&a, &b), (V::String(_), V::String(_) | V::Empty) | (V::Empty, V::String(_))) {
        return binop(BinOp::Concat, a, b, max_string);
    }
    let x = numeric_operand(a)?;
    let y = numeric_operand(b)?;
    let r = rank(&x).max(rank(&y));
    match op {
        BinOp::Add | BinOp::Sub | BinOp::Mul => {
            if r == 4 {
                let (p, q) = (x.to_f64()?, y.to_f64()?);
                let v = match op {
                    BinOp::Add => p + q,
                    BinOp::Sub => p - q,
                    _ => p * q,
                };
                if !v.is_finite() {
                    return Err(RuntimeError::overflow());
                }
                Ok(V::Double(v))
            } else {
                let (p, q) = (x.to_i64()?, y.to_i64()?);
                let v = match op {
                    BinOp::Add => p.checked_add(q),
                    BinOp::Sub => p.checked_sub(q),
                    _ => p.checked_mul(q),
                };
                int_result(v.ok_or_else(RuntimeError::overflow)?, r)
            }
        }
        BinOp::Div => {
            let (p, q) = (x.to_f64()?, y.to_f64()?);
            if q == 0.0 {
                return Err(if p == 0.0 { RuntimeError::overflow() } else { RuntimeError::div_zero() });
            }
            let v = p / q;
            if !v.is_finite() {
                return Err(RuntimeError::overflow());
            }
            Ok(V::Double(v))
        }
        BinOp::IntDiv | BinOp::Mod => {
            let (p, q) = (x.to_i64()?, y.to_i64()?);
            if q == 0 {
                return Err(RuntimeError::div_zero());
            }
            let v = if op == BinOp::IntDiv { p.checked_div(q) } else { p.checked_rem(q) };
            int_result(v.ok_or_else(RuntimeError::overflow)?, if r == 4 { 2 } else { r.max(1) })
        }
        BinOp::Pow => {
            let (p, q) = (x.to_f64()?, y.to_f64()?);
            if (p == 0.0 && q < 0.0) || (p < 0.0 && q.fract() != 0.0) {
                return Err(RuntimeError::invalid_call());
            }
            let v = p.powf(q);
            if !v.is_finite() {
                return Err(RuntimeError::overflow());
            }
            Ok(V::Double(v))
        }
        BinOp::And | BinOp::Or | BinOp::Xor | BinOp::Eqv | BinOp::Imp => {
            let (p, q) = (x.to_i64()?, y.to_i64()?);
            let v = match op {
                BinOp::And => p & q,
                BinOp::Or => p | q,
                BinOp::Xor => p ^ q,
                BinOp::Eqv => !(p ^ q),
                _ => !p | q,
            };
            int_result(v, if r == 4 { 2 } else { r.max(1) })
        }
        _ => Err(RuntimeError::type_mismatch()),
    }
}
