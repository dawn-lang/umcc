// Copyright (c) 2021 Scott J Maddox
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::core::{
    EvalError, Expr, Interner, Intrinsic, Map, StackId, StackSymbol, TermSymbol, Value,
    ValueMultistack, ValueStack,
};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct ResolvedTermSymbol(pub(crate) String);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct ResolvedStackSymbol(pub(crate) String);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ResolvedStackId(pub(crate) ResolvedStackSymbol, pub(crate) u32);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolvedExpr {
    Empty,
    Intrinsic(Intrinsic),
    Call(ResolvedTermSymbol),
    Quote(Box<ResolvedExpr>),
    Compose(Vec<ResolvedExpr>),
    StackContext(ResolvedStackId, Box<ResolvedExpr>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolvedValue {
    Call(ResolvedTermSymbol),
    Quote(Box<ResolvedExpr>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedValueStack(pub(crate) Vec<ResolvedValue>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedValueMultistack(pub(crate) Map<ResolvedStackId, ResolvedValueStack>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolvedEvalError {
    TooFewValues { available: usize, expected: usize },
    UndefinedTerm(ResolvedTermSymbol),
    MissingStackContexts(ResolvedExpr),
}

pub(crate) trait Resolve {
    type Output;
    fn resolve(&self, interner: &Interner) -> Self::Output;
}

impl Resolve for () {
    type Output = ();
    fn resolve(&self, _: &Interner) -> Self::Output {
        ()
    }
}

impl<T, E> Resolve for Result<T, E>
where
    T: Resolve,
    E: Resolve,
{
    type Output = Result<<T as Resolve>::Output, <E as Resolve>::Output>;
    fn resolve(&self, interner: &Interner) -> Self::Output {
        match self {
            Ok(t) => Ok(t.resolve(interner)),
            Err(e) => Err(e.resolve(interner)),
        }
    }
}

impl Resolve for TermSymbol {
    type Output = ResolvedTermSymbol;
    fn resolve(&self, interner: &Interner) -> Self::Output {
        ResolvedTermSymbol(interner.resolve(&self.0).to_owned())
    }
}

impl Resolve for StackSymbol {
    type Output = ResolvedStackSymbol;
    fn resolve(&self, interner: &Interner) -> Self::Output {
        ResolvedStackSymbol(interner.resolve(&self.0).to_owned())
    }
}

impl Resolve for StackId {
    type Output = ResolvedStackId;
    fn resolve(&self, interner: &Interner) -> Self::Output {
        ResolvedStackId(self.0.resolve(interner), self.1)
    }
}

impl Resolve for Expr {
    type Output = ResolvedExpr;
    fn resolve(&self, interner: &Interner) -> Self::Output {
        match self {
            Expr::Intrinsic(i) => ResolvedExpr::Intrinsic(*i),
            Expr::Call(sym) => ResolvedExpr::Call(sym.resolve(interner)),
            Expr::Quote(e) => ResolvedExpr::Quote(Box::new(e.resolve(interner))),
            Expr::Compose(es) => {
                ResolvedExpr::Compose(es.iter().map(|e| e.resolve(interner)).collect())
            }
            Expr::StackContext(s, e) => {
                ResolvedExpr::StackContext(s.resolve(interner), Box::new(e.resolve(interner)))
            }
        }
    }
}

impl Resolve for Value {
    type Output = ResolvedValue;
    fn resolve(&self, interner: &Interner) -> Self::Output {
        match self {
            Value::Call(sym) => ResolvedValue::Call(sym.resolve(interner)),
            Value::Quote(e) => ResolvedValue::Quote(Box::new(e.resolve(interner))),
        }
    }
}

impl Resolve for ValueStack {
    type Output = ResolvedValueStack;
    fn resolve(&self, interner: &Interner) -> Self::Output {
        ResolvedValueStack(self.0.iter().map(|v| v.resolve(interner)).collect())
    }
}

impl Resolve for ValueMultistack {
    type Output = ResolvedValueMultistack;
    fn resolve(&self, interner: &Interner) -> Self::Output {
        ResolvedValueMultistack(
            self.0
                .iter()
                .map(|(k, v)| (k.resolve(interner), v.resolve(interner)))
                .collect(),
        )
    }
}

impl Resolve for EvalError {
    type Output = ResolvedEvalError;
    fn resolve(&self, interner: &Interner) -> Self::Output {
        match self {
            &EvalError::TooFewValues {
                available,
                expected,
            } => ResolvedEvalError::TooFewValues {
                available,
                expected,
            },
            EvalError::UndefinedTerm(sym) => {
                ResolvedEvalError::UndefinedTerm(sym.resolve(interner))
            }
            EvalError::MissingStackContexts(e) => {
                ResolvedEvalError::MissingStackContexts(e.resolve(interner))
            }
        }
    }
}

impl ResolvedExpr {
    fn is_compose(&self) -> bool {
        match self {
            ResolvedExpr::Compose(..) => true,
            _ => false,
        }
    }
}

impl fmt::Display for ResolvedTermSymbol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::Display for ResolvedStackSymbol {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::Display for ResolvedStackId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)?;
        if self.1 != 0 {
            "'".fmt(f)?;
            self.1.fmt(f)?;
        }
        Ok(())
    }
}

impl fmt::Display for Intrinsic {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Intrinsic::Push => "push".fmt(f),
            Intrinsic::Pop => "pop".fmt(f),
            Intrinsic::Clone => "clone".fmt(f),
            Intrinsic::Drop => "drop".fmt(f),
            Intrinsic::Quote => "quote".fmt(f),
            Intrinsic::Compose => "compose".fmt(f),
            Intrinsic::Apply => "apply".fmt(f),
        }
    }
}

impl fmt::Display for ResolvedExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ResolvedExpr::Empty => Ok(()),
            ResolvedExpr::Intrinsic(i) => i.fmt(f),
            ResolvedExpr::Call(t) => write!(f, "{}", t),
            ResolvedExpr::Quote(e) => write!(f, "[{}]", e),
            ResolvedExpr::Compose(es) => {
                if let Some(e) = es.first() {
                    if e.is_compose() {
                        write!(f, "({})", e)?;
                    } else {
                        write!(f, "{}", e)?;
                    }
                }
                for e in es.iter().skip(1) {
                    " ".fmt(f)?;
                    if e.is_compose() {
                        write!(f, "({})", e)?;
                    } else {
                        write!(f, "{}", e)?;
                    }
                }
                Ok(())
            }
            ResolvedExpr::StackContext(s, e) => write!(f, "({}|{})", s, e),
        }
    }
}

impl fmt::Display for ResolvedValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ResolvedValue::Call(sym) => sym.fmt(f),
            ResolvedValue::Quote(v) => write!(f, "[{}]", v),
        }
    }
}

impl fmt::Display for ResolvedValueStack {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        "⟨".fmt(f)?;
        if let Some(v) = self.0.first() {
            v.fmt(f)?;
        }
        for v in self.0.iter().skip(1) {
            " ".fmt(f)?;
            v.fmt(f)?;
        }
        "⟩".fmt(f)
    }
}

impl fmt::Display for ResolvedValueMultistack {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut sids: Vec<ResolvedStackId> = self.0.keys().cloned().collect();
        sids.sort_unstable();
        if let Some(sid) = sids.first() {
            write!(f, "{}", self.0.get(sid).unwrap())?;
        }
        for sid in sids.iter().skip(1) {
            " ".fmt(f)?;
            sid.fmt(f)?;
        }
        Ok(())
    }
}
