// Copyright (c) 2021 Scott J Maddox
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::core::*;
use crate::parse::*;

#[test]
fn test_parse_expr_empty() {
    let interner = &mut Interner::default();
    let input = "";
    let e = ExprParser::new().parse(interner, input).unwrap();
    assert_eq!(e, Expr::default());
}

#[test]
fn test_parse_expr_intrinsic() {
    let cases = [
        ("push", Expr::Intrinsic(Intrinsic::Push)),
        ("pop", Expr::Intrinsic(Intrinsic::Pop)),
        ("clone", Expr::Intrinsic(Intrinsic::Clone)),
        ("drop", Expr::Intrinsic(Intrinsic::Drop)),
        ("quote", Expr::Intrinsic(Intrinsic::Quote)),
        ("compose", Expr::Intrinsic(Intrinsic::Compose)),
        ("apply", Expr::Intrinsic(Intrinsic::Apply)),
    ];
    for (e_src, e_expected) in cases {
        let interner = &mut Interner::default();
        let e = ExprParser::new().parse(interner, e_src).unwrap();
        assert_eq!(e, e_expected);
    }
}

#[test]
fn test_parse_expr_call() {
    let interner = &mut Interner::default();
    assert_eq!(
        ExprParser::new().parse(interner, "foo").unwrap(),
        Expr::Call(TermSymbol(interner.get("foo").unwrap()))
    );
}

#[test]
fn test_parse_expr_call2() {
    let interner = &mut Interner::default();
    assert_eq!(
        ExprParser::new().parse(interner, "foo bar").unwrap(),
        Expr::Compose(vec![
            Expr::Call(TermSymbol(interner.get("foo").unwrap())),
            Expr::Call(TermSymbol(interner.get("bar").unwrap())),
        ])
    );
}

#[test]
fn test_parse_expr_quote_call() {
    let interner = &mut Interner::default();
    assert_eq!(
        ExprParser::new().parse(interner, "[foo]").unwrap(),
        Expr::Quote(Box::new(Expr::Call(TermSymbol(
            interner.get("foo").unwrap(),
        ))))
    );
}

#[test]
fn test_parse_expr_quote_call2() {
    let interner = &mut Interner::default();
    assert_eq!(
        ExprParser::new().parse(interner, "[foo bar]").unwrap(),
        Expr::Quote(Box::new(Expr::Compose(vec![
            Expr::Call(TermSymbol(interner.get("foo").unwrap())),
            Expr::Call(TermSymbol(interner.get("bar").unwrap())),
        ])))
    );
}

#[test]
fn test_parse_expr_stack_context_call() {
    let interner = &mut Interner::default();
    assert_eq!(
        ExprParser::new().parse(interner, "(s|foo)").unwrap(),
        Expr::StackContext(
            StackId(StackSymbol(interner.get("s").unwrap()), 0),
            Box::new(Expr::Call(TermSymbol(interner.get("foo").unwrap()))),
        )
    );
}

#[test]
fn test_parse_expr_stack_context_call2() {
    let interner = &mut Interner::default();
    assert_eq!(
        ExprParser::new().parse(interner, "(s|foo bar)").unwrap(),
        Expr::StackContext(
            StackId(StackSymbol(interner.get("s").unwrap()), 0),
            Box::new(Expr::Compose(vec![
                Expr::Call(TermSymbol(interner.get("foo").unwrap())),
                Expr::Call(TermSymbol(interner.get("bar").unwrap())),
            ])),
        )
    );
}

#[test]
fn test_parse_term_def() {
    let interner = &mut Interner::default();
    assert_eq!(
        TermDefParser::new()
            .parse(interner, "term empty = ;")
            .unwrap(),
        TermDef(
            TermSymbol(interner.get("empty").unwrap()),
            ExprParser::new().parse(interner, "").unwrap()
        )
    );
}

#[test]
fn test_parse_interp_items() {
    let interner = &mut Interner::default();
    assert_eq!(
        InterpItemsParser::new().parse(interner, "").unwrap(),
        (vec![], Expr::Compose(vec![])),
    );
    assert_eq!(
        InterpItemsParser::new().parse(interner, "foo").unwrap(),
        (vec![], Expr::Call(TermSymbol(interner.get("foo").unwrap()))),
    );
    assert_eq!(
        InterpItemsParser::new()
            .parse(interner, "term empty = ;")
            .unwrap(),
        (
            vec![TermDef(
                TermSymbol(interner.get("empty").unwrap()),
                ExprParser::new().parse(interner, "").unwrap()
            )],
            Expr::Compose(vec![])
        ),
    );
    assert_eq!(
        InterpItemsParser::new()
            .parse(interner, "term empty1 = ; term empty2 = ; foo")
            .unwrap(),
        (
            vec![
                TermDef(
                    TermSymbol(interner.get("empty1").unwrap()),
                    ExprParser::new().parse(interner, "").unwrap()
                ),
                TermDef(
                    TermSymbol(interner.get("empty2").unwrap()),
                    ExprParser::new().parse(interner, "").unwrap()
                )
            ],
            Expr::Call(TermSymbol(interner.get("foo").unwrap()))
        ),
    );
}

#[test]
fn test_parse_value_call() {
    let interner = &mut Interner::default();
    assert_eq!(
        ValueParser::new().parse(interner, "v").unwrap(),
        Value::Call(TermSymbol(interner.get("v").unwrap())),
    );
}

#[test]
fn test_parse_value_quote() {
    let interner = &mut Interner::default();
    assert_eq!(
        ValueParser::new().parse(interner, "[]").unwrap(),
        Value::Quote(Box::new(Expr::default())),
    );
}

#[test]
fn test_parse_value_multistack_empty() {
    let interner = &mut Interner::default();
    assert_eq!(
        ValueMultistackParser::new().parse(interner, "").unwrap(),
        ValueMultistack(Map::default()),
    );
}

#[test]
fn test_parse_value_multistack_s_empty() {
    let interner = &mut Interner::default();
    assert_eq!(
        ValueMultistackParser::new()
            .parse(interner, "⟨s|⟩")
            .unwrap(),
        ValueMultistack(crate::map! {
            StackId(StackSymbol(interner.get("s").unwrap()), 0) =>
                ValueStack(vec![]),
        }),
    );
}

#[test]
fn test_parse_value_multistack_s_call() {
    let interner = &mut Interner::default();
    assert_eq!(
        ValueMultistackParser::new()
            .parse(interner, "⟨s|foo⟩")
            .unwrap(),
        ValueMultistack(crate::map! {
            StackId(StackSymbol(interner.get("s").unwrap()), 0) =>
                ValueStack(vec![Value::Call(TermSymbol(interner.get("foo").unwrap()))]),
        }),
    );
}

#[test]
fn test_parse_value_multistack_s_quote() {
    let interner = &mut Interner::default();
    assert_eq!(
        ValueMultistackParser::new()
            .parse(interner, "⟨s|[]⟩")
            .unwrap(),
        ValueMultistack(crate::map! {
            StackId(StackSymbol(interner.get("s").unwrap()), 0) =>
                ValueStack(vec![Value::Quote(Box::new(Expr::default()))]),
        }),
    );
}

#[test]
fn test_parse_value_multistack_s1_call_s2_call() {
    let interner = &mut Interner::default();
    assert_eq!(
        ValueMultistackParser::new()
            .parse(interner, "⟨s1|foo⟩ ⟨s2|bar⟩")
            .unwrap(),
        ValueMultistack(crate::map! {
            StackId(StackSymbol(interner.get("s1").unwrap()), 0) =>
                ValueStack(vec![Value::Call(TermSymbol(interner.get("foo").unwrap()))]),
            StackId(StackSymbol(interner.get("s2").unwrap()), 0) =>
                ValueStack(vec![Value::Call(TermSymbol(interner.get("bar").unwrap()))]),
        }),
    );
}

#[test]
fn test_parse_small_step_rule() {
    let cases = &[
        ("IntrPush", SmallStepRule::IntrPush),
        ("IntrPop", SmallStepRule::IntrPop),
        ("IntrClone", SmallStepRule::IntrClone),
        ("IntrDrop", SmallStepRule::IntrDrop),
        ("IntrQuote", SmallStepRule::IntrQuote),
        ("IntrCompose", SmallStepRule::IntrCompose),
        ("IntrApply", SmallStepRule::IntrApply),
        ("LitCallQuote", SmallStepRule::LitCallQuote),
        ("LitCall", SmallStepRule::LitCall),
        ("LitQuote", SmallStepRule::LitQuote),
        ("StkCtxDistr", SmallStepRule::StkCtxDistr),
        ("StkCtx3Redund", SmallStepRule::StkCtx3Redund),
        ("StkCtxEmpty", SmallStepRule::StkCtxEmpty),
    ][..];
    for (src, expected) in cases {
        let interner = &mut Interner::default();
        assert_eq!(
            SmallStepRuleParser::new().parse(interner, src).unwrap(),
            *expected,
        )
    }
}

#[test]
fn test_parse_small_step_assertion() {
    let interner = &mut Interner::default();
    assert_eq!(
        SmallStepAssertionParser::new()
            .parse(interner, "⟨s1|v1⟩ e1 ⟶IntrPush ⟨s2|v2⟩ e2")
            .unwrap(),
        (
            ValueMultistack(crate::map! {
                StackId(StackSymbol(interner.get("s1").unwrap()), 0) =>
                    ValueStack(vec![Value::Call(TermSymbol(interner.get("v1").unwrap()))]),
            }),
            Expr::Call(TermSymbol(interner.get("e1").unwrap())),
            SmallStepRule::IntrPush,
            ValueMultistack(crate::map! {
                StackId(StackSymbol(interner.get("s2").unwrap()), 0) =>
                    ValueStack(vec![Value::Call(TermSymbol(interner.get("v2").unwrap()))]),
            }),
            Expr::Call(TermSymbol(interner.get("e2").unwrap())),
        )
    )
}

#[test]
fn test_parse_big_step_assertion() {
    let interner = &mut Interner::default();
    assert_eq!(
        BigStepAssertionParser::new()
            .parse(interner, "⟨s1|v1⟩ e1 ⇓ ⟨s2|v2⟩ e2")
            .unwrap(),
        (
            ValueMultistack(crate::map! {
                StackId(StackSymbol(interner.get("s1").unwrap()), 0) =>
                    ValueStack(vec![Value::Call(TermSymbol(interner.get("v1").unwrap()))]),
            }),
            Expr::Call(TermSymbol(interner.get("e1").unwrap())),
            ValueMultistack(crate::map! {
                StackId(StackSymbol(interner.get("s2").unwrap()), 0) =>
                    ValueStack(vec![Value::Call(TermSymbol(interner.get("v2").unwrap()))]),
            }),
            Expr::Call(TermSymbol(interner.get("e2").unwrap())),
        )
    )
}
