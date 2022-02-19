// Copyright (c) 2021 Scott J Maddox
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::builtin::TERM_DEF_SRCS;
use crate::core::*;
use crate::display::*;
use crate::parse::*;

#[test]
fn test_expr_deshadow() {
    let mut ctx = Context::default();
    let s_sym = StackSymbol(ctx.interner.get_or_intern_static("s"));
    let s_0 = StackId(s_sym, 0);
    let e = Expr::StackContext(
        s_0,
        Box::new(Expr::StackContext(s_0, Box::new(Expr::Compose(vec![])))),
    );
    let mut e_deshadowed = e.clone();
    e_deshadowed.deshadow();
    let s_1 = StackId(s_sym, 1);
    let e_expected = Expr::StackContext(
        s_0,
        Box::new(Expr::StackContext(s_1, Box::new(Expr::Compose(vec![])))),
    );
    assert_eq!(e_deshadowed, e_expected);
}

#[test]
fn test_expr_deshadow_not_through_quote() {
    let mut ctx = Context::default();
    let s_sym = StackSymbol(ctx.interner.get_or_intern_static("s"));
    let s_0 = StackId(s_sym, 0);
    let e = Expr::StackContext(
        s_0,
        Box::new(Expr::Quote(Box::new(Expr::StackContext(
            s_0,
            Box::new(Expr::Compose(vec![])),
        )))),
    );
    let mut e_deshadowed = e.clone();
    e_deshadowed.deshadow();
    assert_eq!(e_deshadowed, e);
}

#[test]
fn test_define_term() {
    let mut ctx = Context::default();
    let sym = TermSymbol(ctx.interner.get_or_intern_static("foo"));
    let term_def1 = TermDefParser::new()
        .parse(&mut ctx.interner, "{term foo = e1}")
        .unwrap();
    let e1 = ExprParser::new().parse(&mut ctx.interner, "e1").unwrap();
    let term_def2 = TermDefParser::new()
        .parse(&mut ctx.interner, "{term foo = e2}")
        .unwrap();
    let e2 = ExprParser::new().parse(&mut ctx.interner, "e2").unwrap();
    assert_eq!(ctx.terms.get(&sym), None);
    assert_eq!(ctx.define_term(term_def1), None);
    assert_eq!(ctx.terms.get(&sym), Some(&e1));
    assert_eq!(ctx.define_term(term_def2), Some(TermDef(sym, e1)));
    assert_eq!(ctx.terms.get(&sym), Some(&e2));
}

#[test]
fn test_define_term_with_shadowing() {
    let mut ctx = Context::default();
    let sym = TermSymbol(ctx.interner.get_or_intern_static("foo"));
    let term_def1 = TermDefParser::new()
        .parse(&mut ctx.interner, "{term foo = (s|(s|e))}")
        .unwrap();
    let e = ExprParser::new()
        .parse(&mut ctx.interner, "(s|(s|e))")
        .unwrap();
    let mut e_deshadowed = e.clone();
    e_deshadowed.deshadow();
    assert_ne!(e, e_deshadowed);
    assert_eq!(ctx.terms.get(&sym), None);
    assert_eq!(ctx.define_term(term_def1), None);
    assert_eq!(ctx.terms.get(&sym), Some(&e_deshadowed));
}

#[test]
fn test_small_step() {
    let cases = [
        // Intrinsics
        "⟨s|V v⟩⟨sp|Vp vp⟩ (sp|(s|push)) ‒IntrPush⟶ ⟨s|V v vp⟩⟨sp|Vp⟩",
        "⟨s|V v⟩⟨sp|Vp vp⟩ (sp|(s|pop )) ‒IntrPop⟶  ⟨s|V⟩⟨sp|Vp vp v⟩",
        "⟨s|V v⟩ (sp|(s|clone)) ‒IntrClone⟶ ⟨s|V v v⟩",
        "⟨s|V v⟩ (sp|(s|drop)) ‒IntrDrop⟶ ⟨s|V⟩",
        "⟨s|V v⟩ (sp|(s|quote)) ‒IntrQuote⟶ ⟨s|V [v]⟩",
        "⟨s|V [] []⟩ (sp|(s|compose)) ‒IntrCompose⟶ ⟨s|V []⟩",
        "⟨s|V [e1] [e2]⟩ (sp|(s|compose)) ‒IntrCompose⟶ ⟨s|V [e1 e2]⟩",
        "⟨s|V [e1] [e2 e3]⟩ (sp|(s|compose)) ‒IntrCompose⟶ ⟨s|V [e1 e2 e3]⟩",
        "⟨s|V [e1 e2] [e3]⟩ (sp|(s|compose)) ‒IntrCompose⟶ ⟨s|V [e1 e2 e3]⟩",
        "⟨s|V [e1 e2] [e3 e4]⟩ (sp|(s|compose)) ‒IntrCompose⟶ ⟨s|V [e1 e2 e3 e4]⟩",
        "⟨s|V []⟩ (sp|(s|apply)) ‒IntrApply⟶ ⟨s|V⟩ (sp|(s|))",
        "⟨s|V [e]⟩ (sp|(s|apply)) ‒IntrApply⟶ ⟨s|V⟩ (sp|(s|e))",
        "⟨s|V [e1 e2]⟩ (sp|(s|apply)) ‒IntrApply⟶ ⟨s|V⟩ (sp|(s|e1 e2))",
        // Literal Call
        "(sp|(s|quote0)) ‒LitCall⟶ (sp|(s|[]))",
        // Literal Quote
        "(sp|(s|[e])) ‒LitQuote⟶ ⟨s|[e]⟩",
        // Distribution
        "(s|a b c) ‒StkCtxDistr⟶ (s|a) (s|b c)",
        "(s1|(s2|a b c)) ‒StkCtxDistr⟶ (s1|(s2|a) (s2|b c))",
        "(s1|(s2|a) (s2|b c)) ‒StkCtxDistr⟶ (s1|(s2|a)) (s1|(s2|b c))",
        // Redundant stack contexts
        "(s1|(s2|(s3|e))) ‒StkCtx3Redund⟶ (s2|(s3|e))",
        // Empty stack contexts
        "(s1|(s2|)) ‒StkCtxEmpty⟶ (s1|)",
        "(s1|) ‒StkCtxEmpty⟶ ",
    ];
    for case in cases {
        let mut ctx = Context::default();
        for term_def_src in TERM_DEF_SRCS.iter() {
            let term_def = TermDefParser::new()
                .parse(&mut ctx.interner, term_def_src)
                .unwrap();
            assert_eq!(ctx.define_term(term_def), None);
        }
        let (mut vms1, mut e1, rule, vms2, e2) = SmallStepAssertionParser::new()
            .parse(&mut ctx.interner, case)
            .unwrap();
        let result = ctx.small_step(&mut vms1, &mut e1);
        assert_eq!(
            (
                result,
                vms1.resolve(&ctx.interner),
                e1.resolve(&ctx.interner)
            ),
            (
                Ok(rule),
                vms2.resolve(&ctx.interner),
                e2.resolve(&ctx.interner)
            ),
            "Failed on {:?}",
            case
        );
    }
}

#[test]
fn test_big_step() {
    const MAX_SMALL_STEPS: usize = 2000;
    let cases = [
        "⟨s|v1 v2⟩ (sp|(s|swap)) ⇓ ⟨s|v2 v1⟩",
        "⟨s|v1 v2⟩ (sp|(s|swap swap)) ⇓ ⟨s|v1 v2⟩",
        "⟨s2|v1 v2⟩ (s1|(s2|swap)) ⇓ ⟨s2|v2 v1⟩",
        "⟨s2|v1 v2 [(s1|push)(s2|push)(s1|pop)(s2|pop)]⟩ (s1|(s2|apply)) ⇓ ⟨s2|v2 v1⟩",
        "⟨s|[v1] [v2]⟩ (sp|(s|compose2)) ⇓ ⟨s|[v1 v2]⟩",
        "⟨s|[v1] [v2] [v3]⟩ (sp|(s|compose3)) ⇓ ⟨s|[v1 v2 v3]⟩",
        "⟨s|⟩ (sp|(s|quote0)) ⇓ ⟨s|[]⟩",
        "⟨s|v1⟩ (sp|(s|quote1)) ⇓ ⟨s|[v1]⟩",
        "⟨s|v1 v2⟩ (sp|(s|quote2)) ⇓ ⟨s|[v1 v2]⟩",
        "⟨s|v1 v2 v3⟩ (sp|(s|quote3)) ⇓ ⟨s|[v1 v2 v3]⟩",
        "⟨s|⟩ (sp|(s|False)) ⇓ ⟨s|[_False]⟩",
        "⟨s|⟩ (sp|(s|True)) ⇓ ⟨s|[_True]⟩",
        "⟨s|⟩ (sp|(s|False not)) ⇓ ⟨s|[_True]⟩",
        "⟨s|⟩ (sp|(s|True not)) ⇓ ⟨s|[_False]⟩",
        "⟨s|⟩ (sp|(s|False False or)) ⇓ ⟨s|[_False]⟩",
        "⟨s|⟩ (sp|(s|False True or)) ⇓ ⟨s|[_True]⟩",
        "⟨s|⟩ (sp|(s|True False or)) ⇓ ⟨s|[_True]⟩",
        "⟨s|⟩ (sp|(s|True True or)) ⇓ ⟨s|[_True]⟩",
        "⟨s|⟩ (sp|(s|False False and)) ⇓ ⟨s|[_False]⟩",
        "⟨s|⟩ (sp|(s|False True and)) ⇓ ⟨s|[_False]⟩",
        "⟨s|⟩ (sp|(s|True False and)) ⇓ ⟨s|[_False]⟩",
        "⟨s|⟩ (sp|(s|True True and)) ⇓ ⟨s|[_True]⟩",
        "⟨s|⟩ (sp|(s|Z)) ⇓ ⟨s|[_Z]⟩",
        "⟨s|⟩ (sp|(s|Z S)) ⇓ ⟨s|[[_Z] _S]⟩",
        "⟨s|⟩ (sp|(s|Z S S)) ⇓ ⟨s|[[[_Z] _S] _S]⟩",
        "⟨s|⟩ (sp|(s|Z S S S)) ⇓ ⟨s|[[[[_Z] _S] _S] _S]⟩",
        "⟨s|⟩ (sp|(s|Z succ)) ⇓ ⟨s|[[_Z] _S]⟩",
        "⟨s|⟩ (sp|(s|Z S succ)) ⇓ ⟨s|[[[_Z] _S] _S]⟩",
        "⟨s|⟩ (sp|(s|Z Z add)) ⇓ ⟨s|[_Z]⟩",
        "⟨s|⟩ (sp|(s|Z Z S add)) ⇓ ⟨s|[[_Z] _S]⟩",
        "⟨s|⟩ (sp|(s|Z S Z add)) ⇓ ⟨s|[[_Z] _S]⟩",
        "⟨s|⟩ (sp|(s|Z S Z S add)) ⇓ ⟨s|[[[_Z] _S] _S]⟩",
        "⟨s|⟩ (sp|(s|Z S Z S S add)) ⇓ ⟨s|[[[[_Z] _S] _S] _S]⟩",
        "⟨s|⟩ (sp|(s|Z S S Z S add)) ⇓ ⟨s|[[[[_Z] _S] _S] _S]⟩",
        "⟨s|⟩ (sp|(s|Z S S Z S S add)) ⇓ ⟨s|[[[[[_Z] _S] _S] _S] _S]⟩",
        "⟨s|⟩ (sp|(s|Z Z mul)) ⇓ ⟨s|[_Z]⟩",
        "⟨s|⟩ (sp|(s|Z Z S mul)) ⇓ ⟨s|[_Z]⟩",
        "⟨s|⟩ (sp|(s|Z S Z mul)) ⇓ ⟨s|[_Z]⟩",
        "⟨s|⟩ (sp|(s|Z S Z S mul)) ⇓ ⟨s|[[_Z] _S]⟩",
        "⟨s|⟩ (sp|(s|Z S Z S S mul)) ⇓ ⟨s|[[[_Z] _S] _S]⟩",
        "⟨s|⟩ (sp|(s|Z S S Z S mul)) ⇓ ⟨s|[[[_Z] _S] _S]⟩",
        "⟨s|⟩ (sp|(s|Z S S Z S S mul)) ⇓ ⟨s|[[[[[_Z] _S] _S] _S] _S]⟩",
    ];
    let mut ctx = Context::default();
    for term_def_src in TERM_DEF_SRCS.iter() {
        let term_def = TermDefParser::new()
            .parse(&mut ctx.interner, term_def_src)
            .unwrap();
        assert_eq!(ctx.define_term(term_def), None);
    }
    for case in cases {
        println!("\nCase: {}", case);
        let (mut vms1, mut e1, vms2, e2) = BigStepAssertionParser::new()
            .parse(&mut ctx.interner, case)
            .unwrap();
        println!(
            "{} {}",
            vms1.resolve(&ctx.interner),
            e1.resolve(&ctx.interner)
        );
        'eval: for step in 1..=MAX_SMALL_STEPS {
            let rule = match ctx.small_step(&mut vms1, &mut e1) {
                Ok(rule) => rule,
                Err(err) => {
                    println!("Error: {:?}", err.resolve(&ctx.interner));
                    panic!("Failed on {:?}", case);
                }
            };
            println!(
                "‒{}⟶ {} {}",
                rule,
                vms1.resolve(&ctx.interner),
                e1.resolve(&ctx.interner)
            );
            if vms1 == vms2 && e1 == e2 {
                break 'eval;
            } else if step == MAX_SMALL_STEPS {
                panic!("Reached MAX_SMALL_STEPS on {}", case);
            }
        }
    }
}
