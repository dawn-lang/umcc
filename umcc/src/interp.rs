// Copyright (c) 2021 Scott J Maddox
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::builtin::TERM_DEF_SRCS;
use crate::core::*;
use crate::display::*;
use crate::parse::*;
use std::io;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum InterpCommand {
    Eval(Vec<TermDef>, Expr),
    Trace(Expr),
    Show(TermSymbol),
    List,
    Drop,
    Clear,
    Reset,
    Help,
}

pub(crate) static HELP: &'static str = "\
Commands available:

   term <sym> = <expr>;     define <sym> as <expr>
   <expr>                   evaluate <expr>
   :trace <expr>            trace the evaluation of <expr>
   :show <sym>              show the definition of <sym>
   :list                    list the defined symbols
   :drop                    drop the current value stack
   :clear                   clear all definitions
   :reset                   reset the interpreter
   :help                    display this list of commands
";

pub struct Interp {
    ctx: Context,
    _id: StackId,
    __id: StackId,
    vms: ValueMultistack,
    command: Option<InterpCommand>,
}

impl Default for Interp {
    fn default() -> Self {
        let mut ctx = Context::default();
        let _id = StackId(StackSymbol(ctx.interner.get_or_intern_static("_")), 0);
        let __id = StackId(StackSymbol(ctx.interner.get_or_intern_static("__")), 0);
        for term_def_src in TERM_DEF_SRCS.iter() {
            let term_def = TermDefParser::new()
                .parse(&mut ctx.interner, term_def_src)
                .unwrap();
            assert_eq!(ctx.define_term(term_def), None);
        }
        Self {
            ctx,
            _id,
            __id,
            vms: ValueMultistack::default(),
            command: None,
        }
    }
}

impl Interp {
    pub fn is_done(&self) -> bool {
        self.command.is_none()
    }

    fn add_missing_stack_contexts(&mut self, e: Expr) -> Expr {
        match &e {
            Expr::StackContext(_si, ei) => match &(**ei) {
                Expr::Compose(es) if es.is_empty() => e,
                Expr::StackContext(_sii, _eii) => e,
                _ => Expr::StackContext(self._id, Box::new(e)),
            },
            _ => Expr::StackContext(
                self.__id,
                Box::new(Expr::StackContext(self._id, Box::new(e))),
            ),
        }
    }

    pub fn interp_start(&mut self, input: &str, w: &mut dyn io::Write) -> io::Result<()> {
        match InterpCommandParser::new().parse(&mut self.ctx.interner, input) {
            Err(err) => {
                // TODO: better error messages
                w.write_fmt(format_args!("{:?}\n", err))?;
            }
            Ok(InterpCommand::Eval(term_defs, e)) => {
                for term_def in term_defs {
                    let name = term_def.0.resolve(&self.ctx.interner);
                    if let Some(_) = self.ctx.define_term(term_def) {
                        w.write_fmt(format_args!("Redefined `{}`.\n", name))?;
                    } else {
                        w.write_fmt(format_args!("Defined `{}`.\n", name))?;
                    }
                }
                if e != Expr::default() {
                    let e = self.add_missing_stack_contexts(e);
                    w.write_fmt(format_args!(
                        "{} {}\n",
                        self.vms.resolve(&self.ctx.interner),
                        e.resolve(&self.ctx.interner)
                    ))?;
                    self.command = Some(InterpCommand::Eval(vec![], e));
                }
            }
            Ok(InterpCommand::Trace(e)) => {
                if e != Expr::default() {
                    let e = self.add_missing_stack_contexts(e);
                    w.write_fmt(format_args!(
                        "{} {}\n",
                        self.vms.resolve(&self.ctx.interner),
                        e.resolve(&self.ctx.interner)
                    ))?;
                    self.command = Some(InterpCommand::Trace(e));
                }
            }
            Ok(InterpCommand::Show(sym)) => {
                if let Some(e) = self.ctx.terms.get(&sym) {
                    w.write_fmt(format_args!(
                        "term {} = {};\n",
                        sym.resolve(&self.ctx.interner),
                        e.resolve(&self.ctx.interner)
                    ))?;
                } else {
                    w.write_fmt(format_args!("Not defined.\n"))?;
                }
            }
            Ok(InterpCommand::List) => {
                let mut names: Vec<String> = self
                    .ctx
                    .terms
                    .keys()
                    .map(|sym| format!("{}", sym.resolve(&self.ctx.interner)))
                    .collect();
                names.sort_unstable();
                if let Some(name) = names.first() {
                    w.write_all(name.as_bytes())?;
                }
                for name in names.iter().skip(1) {
                    w.write_all(" ".as_bytes())?;
                    w.write_all(name.as_bytes())?;
                }
                w.write_all("\n".as_bytes())?;
            }
            Ok(InterpCommand::Drop) => {
                self.vms = ValueMultistack::default();
                w.write_fmt(format_args!("Values dropped.\n"))?;
            }
            Ok(InterpCommand::Clear) => {
                self.ctx.terms.clear();
                self.ctx.exprs.clear();
                w.write_fmt(format_args!("Definitions cleared.\n"))?;
            }
            Ok(InterpCommand::Reset) => {
                *self = Self::default();
                w.write_fmt(format_args!("Reset.\n"))?;
            }
            Ok(InterpCommand::Help) => {
                w.write_all(HELP.as_bytes())?;
            }
        }
        w.flush()
    }

    pub fn interp_step(&mut self, w: &mut dyn io::Write) -> io::Result<()> {
        match self.command.take() {
            Some(InterpCommand::Eval(_, mut e)) => {
                if e != Expr::default() {
                    if let Err(err) = self.ctx.small_step(&mut self.vms, &mut e) {
                        w.write_fmt(format_args!(
                            "⇓ {} {}\n",
                            self.vms.resolve(&self.ctx.interner),
                            e.resolve(&self.ctx.interner)
                        ))?;
                        // TODO: better error messages
                        w.write_fmt(format_args!("{}\n", err.resolve(&self.ctx.interner)))?;
                        return w.flush();
                    } else {
                        self.ctx.compress(&mut self.vms);
                    }
                    self.command = Some(InterpCommand::Eval(vec![], e));
                } else {
                    w.write_fmt(format_args!(
                        "⇓ {} {}\n",
                        self.vms.resolve(&self.ctx.interner),
                        e.resolve(&self.ctx.interner)
                    ))?;
                }
            }
            Some(InterpCommand::Trace(mut e)) => {
                if e != Expr::default() {
                    let rule = match self.ctx.small_step(&mut self.vms, &mut e) {
                        Ok(rule) => rule,
                        Err(err) => {
                            // TODO: better error messages
                            w.write_fmt(format_args!("{}\n", err.resolve(&self.ctx.interner)))?;
                            return w.flush();
                        }
                    };
                    // TODO: show function expansion as equality, not as small step?
                    w.write_fmt(format_args!(
                        "⟶{} {} {}\n",
                        rule,
                        self.vms.resolve(&self.ctx.interner),
                        e.resolve(&self.ctx.interner)
                    ))?;
                    if self.ctx.compress(&mut self.vms) {
                        w.write_fmt(format_args!(
                            "= {} {}\n",
                            self.vms.resolve(&self.ctx.interner),
                            e.resolve(&self.ctx.interner)
                        ))?;
                    }
                    self.command = Some(InterpCommand::Trace(e));
                }
            }
            _ => panic!(),
        }
        w.flush()
    }
}
