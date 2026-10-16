// SPDX-License-Identifier: MPL-2.0
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

//! Syntax tree for VBA modules.

use crate::value::Variant;

/// A declared type in `As <type>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeName {
    Byte,
    Integer,
    Long,
    LongLong,
    Single,
    Double,
    Currency,
    Date,
    String,
    Boolean,
    Variant,
    Object,
    /// A class or object type such as `Range` or `Worksheet`.
    Class(String),
}

impl TypeName {
    pub fn parse(word: &str) -> TypeName {
        match word.to_ascii_lowercase().as_str() {
            "byte" => TypeName::Byte,
            "integer" => TypeName::Integer,
            "long" => TypeName::Long,
            "longlong" | "longptr" => TypeName::LongLong,
            "single" => TypeName::Single,
            "double" => TypeName::Double,
            "currency" => TypeName::Currency,
            "date" => TypeName::Date,
            "string" => TypeName::String,
            "boolean" => TypeName::Boolean,
            "variant" => TypeName::Variant,
            "object" => TypeName::Object,
            _ => TypeName::Class(word.to_string()),
        }
    }

    pub fn is_object(&self) -> bool {
        matches!(self, TypeName::Object | TypeName::Class(_))
    }
}

/// A module: procedures plus module-level declarations.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Module {
    pub name: String,
    pub option_explicit: bool,
    /// `Option Base` (0 or 1): default lower bound of arrays.
    pub option_base: i64,
    /// Names declared with `Declare` (Windows API); calling them is denied.
    pub external_declares: Vec<String>,
    pub variables: Vec<DimVar>,
    pub constants: Vec<(String, Expr)>,
    pub procedures: Vec<Procedure>,
}

/// `Sub` or `Function`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcKind {
    Sub,
    Function,
}

/// A procedure declaration.
#[derive(Debug, Clone, PartialEq)]
pub struct Procedure {
    pub name: String,
    pub kind: ProcKind,
    pub public: bool,
    pub params: Vec<Param>,
    pub returns: Option<TypeName>,
    pub body: Vec<Stmt>,
    pub line: u32,
}

/// A formal parameter.
#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub name: String,
    pub by_ref: bool,
    pub optional: bool,
    pub param_array: bool,
    pub is_array: bool,
    pub ty: Option<TypeName>,
    pub default: Option<Expr>,
}

/// One variable in a `Dim`, `ReDim`, `Private`, `Public` or `Static` declaration.
#[derive(Debug, Clone, PartialEq)]
pub struct DimVar {
    pub name: String,
    pub ty: Option<TypeName>,
    /// `Some(empty)` for a dynamic array `a()`, `Some(bounds)` for a fixed array.
    pub bounds: Option<Vec<(Option<Expr>, Expr)>>,
    pub line: u32,
}

/// A `Case` test in `Select Case`.
#[derive(Debug, Clone, PartialEq)]
pub enum CaseTest {
    Value(Expr),
    Range(Expr, Expr),
    Is(BinOp, Expr),
}

/// What `Exit` leaves.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExitKind {
    Sub,
    Function,
    For,
    Do,
}

/// The condition form of a `Do` loop.
#[derive(Debug, Clone, PartialEq)]
pub struct LoopCond {
    /// `While` (true) or `Until` (false).
    pub is_while: bool,
    /// Tested before the body (`Do While x`) or after it (`Loop While x`).
    pub pre: bool,
    pub expr: Expr,
}

/// `On Error` modes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OnError {
    ResumeNext,
    GoToZero,
    GoTo(String),
}

/// A statement, with the source line where it starts.
#[derive(Debug, Clone, PartialEq)]
pub struct Stmt {
    pub line: u32,
    pub kind: StmtKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StmtKind {
    Dim(Vec<DimVar>),
    ReDim {
        preserve: bool,
        vars: Vec<DimVar>,
    },
    Const(Vec<(String, Expr)>),
    Assign {
        target: Expr,
        value: Expr,
        set: bool,
    },
    Call(Expr),
    If {
        arms: Vec<(Expr, Vec<Stmt>)>,
        otherwise: Option<Vec<Stmt>>,
    },
    For {
        var: String,
        from: Expr,
        to: Expr,
        step: Option<Expr>,
        body: Vec<Stmt>,
    },
    ForEach {
        var: String,
        collection: Expr,
        body: Vec<Stmt>,
    },
    Do {
        cond: Option<LoopCond>,
        body: Vec<Stmt>,
    },
    While {
        cond: Expr,
        body: Vec<Stmt>,
    },
    Select {
        subject: Expr,
        cases: Vec<(Vec<CaseTest>, Vec<Stmt>)>,
        otherwise: Option<Vec<Stmt>>,
    },
    With {
        object: Expr,
        body: Vec<Stmt>,
    },
    Exit(ExitKind),
    OnError(OnError),
    Label(String),
    GoTo(String),
    /// `Resume Next`, which continues after the statement that raised the error.
    ResumeNext,
    /// `End`: stops the macro.
    End,
    /// `Print`-style statement produced by `Debug.Print`.
    DebugPrint(Vec<(Expr, char)>),
    /// `Erase a, b`.
    Erase(Vec<String>),
    /// A recognized statement that is not implemented yet; executing it is an error.
    Unsupported(String),
}

/// Binary operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Pow,
    Mul,
    Div,
    IntDiv,
    Mod,
    Add,
    Sub,
    Concat,
    Eq,
    Ne,
    Lt,
    Gt,
    Le,
    Ge,
    Is,
    Like,
    And,
    Or,
    Xor,
    Eqv,
    Imp,
}

/// Unary operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnOp {
    Neg,
    Not,
}

/// A call argument.
#[derive(Debug, Clone, PartialEq)]
pub struct Arg {
    /// `name:=value` named argument.
    pub name: Option<String>,
    /// `None` for an omitted argument (`f(1, , 3)`).
    pub value: Option<Expr>,
}

/// An expression.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Literal(Variant),
    Nothing,
    Ident(String),
    /// `obj.Name`; `object == None` means the innermost `With` object.
    Member {
        object: Option<Box<Expr>>,
        name: String,
    },
    /// `f(args)`, `arr(i)`, `obj.Method(args)` or a call statement's arguments.
    Call {
        callee: Box<Expr>,
        args: Vec<Arg>,
    },
    Unary(UnOp, Box<Expr>),
    Binary(BinOp, Box<Expr>, Box<Expr>),
    /// `New ClassName`.
    New(String),
}
