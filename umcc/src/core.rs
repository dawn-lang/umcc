// Copyright (c) 2021 Scott J Maddox
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

pub(crate) use lasso::Rodeo as Interner;
use std::hash::Hash;

pub(crate) type Map<K, V> = fxhash::FxHashMap<K, V>;

#[macro_export]
macro_rules! map {
    ($($k:expr => $v:expr),* $(,)?) => {
        std::iter::Iterator::collect(std::array::IntoIter::new([$(($k, $v),)*]))
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct TermSymbol(pub(crate) lasso::Spur);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(transparent)]
pub struct StackSymbol(pub(crate) lasso::Spur);

/// StackId combines a StackSymbol with a u32 subscript that is used to
/// automatically enforce the restriction that nested stack contexts have
/// disjoint stack identifiers. This enables the user to inline terms without
/// having to rename shadowed stack symbols. The subscript is automatically
/// incremented for each level of shadowed nesting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StackId(pub(crate) StackSymbol, pub(crate) u32);

////////////
// Syntax //
////////////

/// Expressions
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Expr {
    Intrinsic(Intrinsic),
    Call(TermSymbol),
    Quote(Box<Expr>),
    Compose(Vec<Expr>),
    StackContext(StackId, Box<Expr>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Intrinsic {
    Push,
    Pop,
    Clone,
    Drop,
    Quote,
    Compose,
    Apply,
}

impl Default for Expr {
    fn default() -> Self {
        Expr::Compose(vec![])
    }
}

///////////////
// Semantics //
///////////////

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Value {
    Call(TermSymbol),
    Quote(Box<Expr>),
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ValueStack(pub(crate) Vec<Value>);

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ValueMultistack(pub(crate) Map<StackId, ValueStack>);

pub struct Context {
    pub(crate) interner: Interner,
    pub(crate) terms: Map<TermSymbol, Expr>,
    pub(crate) exprs: Map<Expr, TermSymbol>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvalError {
    TooFewValues { available: usize, expected: usize },
    UndefinedTerm(TermSymbol),
    MissingStackContexts(Expr),
}

impl Default for Context {
    fn default() -> Self {
        let interner = Interner::default();
        Context {
            interner,
            terms: Map::default(),
            exprs: Map::default(),
        }
    }
}

impl Context {
    fn unquote_value(&self, v: Value) -> Result<Expr, EvalError> {
        match v {
            Value::Call(sym) => {
                if let Some(e) = self.terms.get(&sym) {
                    match e {
                        Expr::Quote(e) => Ok((**e).clone()),
                        _ => panic!(),
                    }
                } else {
                    Err(EvalError::UndefinedTerm(sym))
                }
            }
            Value::Quote(e) => Ok(*e),
        }
    }

    pub fn small_step(&mut self, vms: &mut ValueMultistack, e: &mut Expr) -> Result<(), EvalError> {
        match e {
            Expr::StackContext(si, ei) => match &mut (**ei) {
                Expr::StackContext(sii, eii) => match &mut (**eii) {
                    Expr::Intrinsic(intr) => match intr {
                        Intrinsic::Push => {
                            let vsi = vms.0.entry(*si).or_default();
                            if vsi.0.len() < 1 {
                                Err(EvalError::TooFewValues {
                                    available: vsi.0.len(),
                                    expected: 1,
                                })
                            } else {
                                let v = vsi.0.pop().unwrap();
                                let vsii = vms.0.entry(*sii).or_default();
                                vsii.0.push(v);
                                *eii = Box::new(Expr::default());
                                Ok(())
                            }
                        }
                        Intrinsic::Pop => {
                            let vsii = vms.0.entry(*sii).or_default();
                            if vsii.0.len() < 1 {
                                Err(EvalError::TooFewValues {
                                    available: vsii.0.len(),
                                    expected: 1,
                                })
                            } else {
                                let v = vsii.0.pop().unwrap();
                                let vsi = vms.0.entry(*si).or_default();
                                vsi.0.push(v);
                                *eii = Box::new(Expr::default());
                                Ok(())
                            }
                        }
                        Intrinsic::Clone => {
                            let vs = vms.0.entry(*sii).or_default();
                            if vs.0.len() < 1 {
                                Err(EvalError::TooFewValues {
                                    available: vs.0.len(),
                                    expected: 1,
                                })
                            } else {
                                vs.0.push(vs.0.last().unwrap().clone());
                                *eii = Box::new(Expr::default());
                                Ok(())
                            }
                        }
                        Intrinsic::Drop => {
                            let vs = vms.0.entry(*sii).or_default();
                            if vs.0.len() < 1 {
                                Err(EvalError::TooFewValues {
                                    available: vs.0.len(),
                                    expected: 1,
                                })
                            } else {
                                vs.0.pop();
                                *eii = Box::new(Expr::default());
                                Ok(())
                            }
                        }
                        Intrinsic::Quote => {
                            let vs = vms.0.entry(*sii).or_default();
                            if vs.0.len() < 1 {
                                Err(EvalError::TooFewValues {
                                    available: vs.0.len(),
                                    expected: 1,
                                })
                            } else {
                                let v = vs.0.pop().unwrap();
                                let qe = match v {
                                    Value::Call(sym) => Expr::Call(sym),
                                    Value::Quote(e) => Expr::Quote(e),
                                };
                                vs.0.push(Value::Quote(Box::new(qe)));
                                *eii = Box::new(Expr::default());
                                Ok(())
                            }
                        }
                        Intrinsic::Compose => {
                            let vs = vms.0.entry(*sii).or_default();
                            if vs.0.len() < 2 {
                                Err(EvalError::TooFewValues {
                                    available: vs.0.len(),
                                    expected: 2,
                                })
                            } else {
                                let e2 = self.unquote_value(vs.0.pop().unwrap())?;
                                let e1 = self.unquote_value(vs.0.pop().unwrap())?;
                                let mut new_es = match (e1, e2) {
                                    (Expr::Compose(mut e1s), Expr::Compose(mut e2s)) => {
                                        e1s.extend(e2s.drain(..));
                                        e1s
                                    }
                                    (Expr::Compose(mut e1s), e2) => {
                                        e1s.push(e2);
                                        e1s
                                    }
                                    (e1, Expr::Compose(mut e2s)) => {
                                        e2s.insert(0, e1);
                                        e2s
                                    }
                                    (e1, e2) => vec![e1, e2],
                                };
                                let new_e = if new_es.len() == 1 {
                                    new_es.drain(..).next().unwrap()
                                } else {
                                    Expr::Compose(new_es)
                                };
                                vs.0.push(Value::Quote(Box::new(new_e)));
                                *eii = Box::new(Expr::default());
                                Ok(())
                            }
                        }
                        Intrinsic::Apply => {
                            let vs = vms.0.entry(*sii).or_default();
                            if vs.0.len() < 1 {
                                Err(EvalError::TooFewValues {
                                    available: vs.0.len(),
                                    expected: 1,
                                })
                            } else {
                                let e1 = self.unquote_value(vs.0.pop().unwrap())?;
                                *eii = Box::new(e1);
                                Ok(())
                            }
                        }
                    },
                    Expr::Call(sym) => {
                        if let Some(new_e) = self.terms.get(sym) {
                            let vs = vms.0.entry(*sii).or_default();
                            match new_e {
                                Expr::Quote(_) => {
                                    vs.0.push(Value::Call(*sym));
                                    *eii = Box::new(Expr::default());
                                    Ok(())
                                }
                                _ => {
                                    *eii = Box::new(new_e.clone());
                                    Ok(())
                                }
                            }
                        } else {
                            Err(EvalError::UndefinedTerm(*sym))
                        }
                    }
                    Expr::Quote(qe) => {
                        let vs = vms.0.entry(*sii).or_default();
                        vs.0.push(Value::Quote(qe.clone()));
                        *eii = Box::new(Expr::default());
                        Ok(())
                    }
                    Expr::Compose(ref mut es) => {
                        let es_len = es.len();
                        if es_len == 0 {
                            *e = Expr::default();
                            Ok(())
                        } else {
                            let e1 = es.first_mut().unwrap();
                            self.small_step(vms, e1)?;
                            match e1 {
                                Expr::Compose(e1s) => {
                                    let mut new_es = Vec::with_capacity(e1s.len() + es_len - 1);
                                    new_es.extend(e1s.drain(..));
                                    new_es.extend(es.drain(1..));
                                    let new_e = if new_es.len() == 1 {
                                        new_es.drain(..).next().unwrap()
                                    } else {
                                        Expr::Compose(new_es)
                                    };
                                    *eii = Box::new(new_e);
                                }
                                _ => {}
                            }
                            Ok(())
                        }
                    }
                    Expr::StackContext(..) => {
                        *e = (**ei).clone();
                        Ok(())
                    }
                },
                Expr::Compose(ref mut es) => {
                    let es_len = es.len();
                    if es_len == 0 {
                        *e = Expr::default();
                        Ok(())
                    } else {
                        Err(EvalError::MissingStackContexts(e.clone()))
                    }
                }
                _ => Err(EvalError::MissingStackContexts(e.clone())),
            },
            Expr::Compose(ref mut es) => {
                let es_len = es.len();
                if es_len == 0 {
                    Ok(())
                } else {
                    Err(EvalError::MissingStackContexts(e.clone()))
                }
            }
            _ => Err(EvalError::MissingStackContexts(e.clone())),
        }
    }
}

//////////////////////
// Term Definitions //
//////////////////////

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TermDef(pub TermSymbol, pub Expr);

impl Context {
    pub fn define_term(&mut self, fn_def: TermDef) -> Option<TermDef> {
        let result = self.terms.remove(&fn_def.0).map(|e| TermDef(fn_def.0, e));
        self.terms.insert(fn_def.0, fn_def.1.clone());
        self.exprs.insert(fn_def.1, fn_def.0);
        result
    }
}
