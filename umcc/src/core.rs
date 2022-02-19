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

impl Expr {
    pub(crate) fn deshadow(&mut self) {
        self._deshadow(&mut Map::default());
    }

    fn _deshadow(&mut self, max_stack_symbol_index: &mut Map<StackSymbol, u32>) {
        match self {
            Expr::Intrinsic(_) => {}
            Expr::Call(_) => {}
            Expr::Quote(e) => e._deshadow(&mut Map::default()),
            Expr::Compose(es) => {
                for e in es {
                    e._deshadow(max_stack_symbol_index);
                }
            }
            Expr::StackContext(s, e) => {
                if max_stack_symbol_index.contains_key(&s.0) {
                    s.1 += 1;
                    *max_stack_symbol_index.get_mut(&s.0).unwrap() += 1;
                    e._deshadow(max_stack_symbol_index);
                    *max_stack_symbol_index.get_mut(&s.0).unwrap() -= 1;
                } else {
                    max_stack_symbol_index.insert(s.0, 0);
                    e._deshadow(max_stack_symbol_index);
                    max_stack_symbol_index.remove(&s.0);
                }
            }
        }
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

impl From<Value> for Expr {
    fn from(v: Value) -> Expr {
        match v {
            Value::Call(sym) => Expr::Call(sym),
            Value::Quote(e) => Expr::Quote(e),
        }
    }
}

impl ValueMultistack {
    fn remove_empty_stacks(&mut self) {
        self.0.retain(|_s, vs| !vs.0.is_empty());
    }
}

pub struct Context {
    pub(crate) interner: Interner,
    pub(crate) terms: Map<TermSymbol, Expr>,
    pub(crate) exprs: Map<Expr, TermSymbol>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SmallStepRule {
    IntrPush,
    IntrPop,
    IntrClone,
    IntrDrop,
    IntrQuote,
    IntrCompose,
    IntrApply,
    LitCall,
    LitQuote,
    StkCtxDistr,
    StkCtx3Redund,
    StkCtxEmpty,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvalError {
    EmptyExpr,
    TooFewValues { available: usize, expected: usize },
    UndefinedTerm(TermSymbol),
    Missing1StackContext,
    Missing2StackContexts,
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

    pub fn small_step(
        &mut self,
        vms: &mut ValueMultistack,
        e: &mut Expr,
    ) -> Result<SmallStepRule, EvalError> {
        match e {
            Expr::Compose(ref mut es) => {
                let es_len = es.len();
                if es_len == 0 {
                    Err(EvalError::EmptyExpr)
                } else {
                    // Recurse on the first sub-expression
                    let e1 = es.first_mut().unwrap();
                    let rule = self.small_step(vms, e1)?;
                    match e1 {
                        Expr::Compose(e1s) => {
                            // concatenate e1s and es
                            if e1s.is_empty() {
                                if es_len > 2 {
                                    *e = Expr::Compose(es.drain(1..).collect());
                                } else {
                                    *e = es.pop().unwrap();
                                }
                            } else {
                                let mut new_es = Vec::with_capacity(e1s.len() + es_len - 1);
                                new_es.extend(e1s.drain(..));
                                new_es.extend(es.drain(1..));
                                let new_e = if new_es.len() == 1 {
                                    new_es.drain(..).next().unwrap()
                                } else {
                                    Expr::Compose(new_es)
                                };
                                *e = new_e;
                            }
                        }
                        _ => {}
                    }
                    Ok(rule)
                }
            }
            Expr::StackContext(si, ei) => {
                match &mut (**ei) {
                    Expr::Compose(ref mut es) => {
                        let es_len = es.len();
                        if es_len == 0 {
                            *e = Expr::default();
                            Ok(SmallStepRule::StkCtxEmpty)
                        } else {
                            // Distribute the stack context.
                            let e2 = if es_len > 2 {
                                Expr::StackContext(
                                    *si,
                                    Box::new(Expr::Compose(es.drain(1..).collect())),
                                )
                            } else {
                                Expr::StackContext(*si, Box::new(es.pop().unwrap()))
                            };
                            let e1 = Expr::StackContext(*si, Box::new(es.pop().unwrap()));
                            *e = Expr::Compose(vec![e1, e2]);
                            Ok(SmallStepRule::StkCtxDistr)
                        }
                    }
                    Expr::StackContext(sii, eii) => {
                        match &mut (**eii) {
                            Expr::Intrinsic(intr) => match intr {
                                Intrinsic::Push => {
                                    if !vms.0.contains_key(si) {
                                        Err(EvalError::TooFewValues {
                                            available: 0,
                                            expected: 1,
                                        })
                                    } else {
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
                                            vms.remove_empty_stacks();
                                            *e = Expr::default();
                                            Ok(SmallStepRule::IntrPush)
                                        }
                                    }
                                }
                                Intrinsic::Pop => {
                                    if !vms.0.contains_key(sii) {
                                        Err(EvalError::TooFewValues {
                                            available: 0,
                                            expected: 1,
                                        })
                                    } else {
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
                                            vms.remove_empty_stacks();
                                            *e = Expr::default();
                                            Ok(SmallStepRule::IntrPop)
                                        }
                                    }
                                }
                                Intrinsic::Clone => {
                                    if !vms.0.contains_key(sii) {
                                        Err(EvalError::TooFewValues {
                                            available: 0,
                                            expected: 1,
                                        })
                                    } else {
                                        let vs = vms.0.entry(*sii).or_default();
                                        if vs.0.len() < 1 {
                                            Err(EvalError::TooFewValues {
                                                available: vs.0.len(),
                                                expected: 1,
                                            })
                                        } else {
                                            vs.0.push(vs.0.last().unwrap().clone());
                                            *e = Expr::default();
                                            Ok(SmallStepRule::IntrClone)
                                        }
                                    }
                                }
                                Intrinsic::Drop => {
                                    if !vms.0.contains_key(sii) {
                                        Err(EvalError::TooFewValues {
                                            available: 0,
                                            expected: 1,
                                        })
                                    } else {
                                        let vs = vms.0.entry(*sii).or_default();
                                        if vs.0.len() < 1 {
                                            Err(EvalError::TooFewValues {
                                                available: vs.0.len(),
                                                expected: 1,
                                            })
                                        } else {
                                            vs.0.pop();
                                            vms.remove_empty_stacks();
                                            *e = Expr::default();
                                            Ok(SmallStepRule::IntrDrop)
                                        }
                                    }
                                }
                                Intrinsic::Quote => {
                                    if !vms.0.contains_key(sii) {
                                        Err(EvalError::TooFewValues {
                                            available: 0,
                                            expected: 1,
                                        })
                                    } else {
                                        let vs = vms.0.entry(*sii).or_default();
                                        if vs.0.len() < 1 {
                                            Err(EvalError::TooFewValues {
                                                available: vs.0.len(),
                                                expected: 1,
                                            })
                                        } else {
                                            let v = vs.0.pop().unwrap();
                                            let qe = Expr::from(v);
                                            vs.0.push(Value::Quote(Box::new(qe)));
                                            *e = Expr::default();
                                            Ok(SmallStepRule::IntrQuote)
                                        }
                                    }
                                }
                                Intrinsic::Compose => {
                                    if !vms.0.contains_key(sii) {
                                        Err(EvalError::TooFewValues {
                                            available: 0,
                                            expected: 2,
                                        })
                                    } else {
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
                                                (
                                                    Expr::Compose(mut e1s),
                                                    Expr::Compose(mut e2s),
                                                ) => {
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
                                            *e = Expr::default();
                                            Ok(SmallStepRule::IntrCompose)
                                        }
                                    }
                                }
                                Intrinsic::Apply => {
                                    if !vms.0.contains_key(sii) {
                                        Err(EvalError::TooFewValues {
                                            available: 0,
                                            expected: 1,
                                        })
                                    } else {
                                        let vs = vms.0.entry(*sii).or_default();
                                        if vs.0.len() < 1 {
                                            Err(EvalError::TooFewValues {
                                                available: vs.0.len(),
                                                expected: 1,
                                            })
                                        } else {
                                            let e1 = self.unquote_value(vs.0.pop().unwrap())?;
                                            vms.remove_empty_stacks();
                                            *eii = Box::new(e1);
                                            e.deshadow();
                                            Ok(SmallStepRule::IntrApply)
                                        }
                                    }
                                }
                            },
                            Expr::Call(sym) => {
                                if let Some(new_e) = self.terms.get(sym) {
                                    *eii = Box::new(new_e.clone());
                                    e.deshadow();
                                    Ok(SmallStepRule::LitCall)
                                } else {
                                    Err(EvalError::UndefinedTerm(*sym))
                                }
                            }
                            Expr::Quote(qe) => {
                                let vs = vms.0.entry(*sii).or_default();
                                vs.0.push(Value::Quote(qe.clone()));
                                *e = Expr::default();
                                Ok(SmallStepRule::LitQuote)
                            }
                            Expr::Compose(ref mut es) => {
                                let es_len = es.len();
                                if es_len == 0 {
                                    *ei = Box::new(Expr::default());
                                    Ok(SmallStepRule::StkCtxEmpty)
                                } else {
                                    // Distribute the stack context.
                                    let e2 = if es_len > 2 {
                                        Expr::StackContext(
                                            *sii,
                                            Box::new(Expr::Compose(es.drain(1..).collect())),
                                        )
                                    } else {
                                        Expr::StackContext(*sii, Box::new(es.pop().unwrap()))
                                    };
                                    let e1 = Expr::StackContext(*sii, Box::new(es.pop().unwrap()));
                                    *ei = Box::new(Expr::Compose(vec![e1, e2]));
                                    Ok(SmallStepRule::StkCtxDistr)
                                }
                            }
                            Expr::StackContext(..) => {
                                *e = (**ei).clone();
                                Ok(SmallStepRule::StkCtx3Redund)
                            }
                        }
                    }
                    _ => Err(EvalError::Missing1StackContext),
                }
            }
            _ => Err(EvalError::Missing2StackContexts),
        }
    }
}

//////////////////////
// Term Definitions //
//////////////////////

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TermDef(pub TermSymbol, pub Expr);

impl Context {
    pub fn define_term(&mut self, mut fn_def: TermDef) -> Option<TermDef> {
        fn_def.1.deshadow();
        let result = self.terms.remove(&fn_def.0).map(|e| TermDef(fn_def.0, e));
        self.terms.insert(fn_def.0, fn_def.1.clone());
        self.exprs.insert(fn_def.1, fn_def.0);
        result
    }
}
