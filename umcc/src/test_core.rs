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
fn test_define_term() {
    let mut ctx = Context::default();
    let sym = TermSymbol(ctx.interner.get_or_intern_static("foo"));
    let term_def1 = TermDefParser::new()
        .parse(&mut ctx.interner, "term foo = e1;")
        .unwrap();
    let e1 = ExprParser::new().parse(&mut ctx.interner, "e1").unwrap();
    let term_def2 = TermDefParser::new()
        .parse(&mut ctx.interner, "term foo = e2;")
        .unwrap();
    let e2 = ExprParser::new().parse(&mut ctx.interner, "e2").unwrap();
    assert_eq!(ctx.terms.get(&sym), None);
    assert_eq!(ctx.define_term(term_def1), None);
    assert_eq!(ctx.terms.get(&sym), Some(&e1));
    assert_eq!(ctx.define_term(term_def2), Some(TermDef(sym, e1)));
    assert_eq!(ctx.terms.get(&sym), Some(&e2));
}

#[test]
fn test_small_step() {
    let cases = [
        "⟨s1|v1⟩⟨s2|v2⟩ (s1|(s2|push)) ⟶IntrPush ⟨s2|v2 v1⟩ (s1|(s2|))",
        "⟨s1|v1⟩⟨s2|v2⟩ (s1|(s2|pop)) ⟶IntrPop ⟨s1|v1 v2⟩ (s1|(s2|))",
        "⟨s|v⟩ (s1|(s|clone)) ⟶IntrClone ⟨s|v v⟩ (s1|(s|))",
        "⟨s|v⟩ (s1|(s|drop)) ⟶IntrDrop (s1|(s|))",
        "⟨s|v⟩ (s1|(s|quote)) ⟶IntrQuote ⟨s|[v]⟩ (s1|(s|))",
        "⟨s|[e1 e2] [e3 e4]⟩ (s1|(s|compose)) ⟶IntrCompose ⟨s|[e1 e2 e3 e4]⟩ (s1|(s|))",
        "⟨s|[e]⟩ (s1|(s|apply)) ⟶IntrApply (s1|(s|e))",
        "⟨s|[e1 e2]⟩ (s1|(s|apply)) ⟶IntrApply (s1|(s|e1 e2))",
        "(s1|(s|[e])) ⟶LitQuote ⟨s|[e]⟩ (s1|(s|))",
        "(s1|(s2|(s3|e))) ⟶StkCtxRedund (s2|(s3|e))",
        "(s1|(s2|)) ⟶StkCtxEmpty ",
        "(s1|) ⟶StkCtxEmpty ",
        " ⟶Empty ",
    ];
    for case in cases {
        let mut ctx = Context::default();
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
            "Failed on {}",
            case
        );
    }
}

#[test]
fn test_compress() {
    let cases = [
        ("⟨s|[swap drop]⟩", "⟨s|true⟩", true),
        ("⟨s|[drop]⟩", "⟨s|n0⟩", true),
        (
            "⟨s|[[clone] n0 apply [compose] n0 apply apply]⟩",
            "⟨s|n1⟩",
            true,
        ),
        (
            "⟨s|[[clone] n1 apply [compose] n1 apply apply]⟩",
            "⟨s|n2⟩",
            true,
        ),
        (
            "⟨s|[[clone] n2 apply [compose] n2 apply apply]⟩",
            "⟨s|n3⟩",
            true,
        ),
        (
            "⟨s|[[clone] n3 apply [compose] n3 apply apply]⟩",
            "⟨s|n4⟩",
            true,
        ),
    ];
    for (input_src, expected_src, expected_result) in cases {
        let mut ctx = Context::default();
        for term_def_src in TERM_DEF_SRCS.iter() {
            let term_def = TermDefParser::new()
                .parse(&mut ctx.interner, term_def_src)
                .unwrap();
            assert_eq!(ctx.define_term(term_def), None);
        }
        let mut input = ValueMultistackParser::new()
            .parse(&mut ctx.interner, input_src)
            .unwrap();
        let expected = ValueMultistackParser::new()
            .parse(&mut ctx.interner, expected_src)
            .unwrap();
        let result = ctx.compress(&mut input);
        assert_eq!(
            (input.resolve(&ctx.interner), result),
            (expected.resolve(&ctx.interner), expected_result),
            "Failed on ({}, {})",
            input_src,
            expected_src
        );
    }
}

#[test]
fn test_big_step() {
    const MAX_SMALL_STEPS: usize = 1000;
    let cases = [
        "⟨s|v1 v2⟩ (sp|(s|swap)) ⇓ ⟨s|v2 v1⟩",
        "⟨s|v1 v2⟩ (sp|(s|swap swap)) ⇓ ⟨s|v1 v2⟩",
        "⟨s|v1 v2⟩ (sp|(s|false apply)) ⇓ ⟨s|v1⟩",
        "⟨s|v1 v2⟩ (sp|(s|true apply)) ⇓ ⟨s|v2⟩",
        "⟨s|false false⟩ (sp|(s|or)) ⇓ ⟨s|false⟩",
        "⟨s|false true⟩ (sp|(s|or)) ⇓ ⟨s|true⟩",
        "⟨s|true false⟩ (sp|(s|or)) ⇓ ⟨s|true⟩",
        "⟨s|true true⟩ (sp|(s|or)) ⇓ ⟨s|true⟩",
        "⟨s|v1 v2⟩ (sp|(s|quote2)) ⇓ ⟨s|[v1 v2]⟩",
        "⟨s|v1 v2 v3⟩ (sp|(s|quote3)) ⇓ ⟨s|[v1 v2 v3]⟩",
        "⟨s|v1 v2 v3⟩ (sp|(s|rotate3)) ⇓ ⟨s|v2 v3 v1⟩",
        "⟨s|v1 v2 v3 v4⟩ (sp|(s|rotate4)) ⇓ ⟨s|v2 v3 v4 v1⟩",
        "⟨s|[v1] [v2]⟩ (sp|(s|compose2)) ⇓ ⟨s|[v1 v2]⟩",
        "⟨s|[v1] [v2] [v3]⟩ (sp|(s|compose3)) ⇓ ⟨s|[v1 v2 v3]⟩",
        "⟨s|[v1] [v2] [v3] [v4]⟩ (sp|(s|compose4)) ⇓ ⟨s|[v1 v2 v3 v4]⟩",
        "⟨s|[v1] [v2] [v3] [v4] [v5]⟩ (sp|(s|compose5)) ⇓ ⟨s|[v1 v2 v3 v4 v5]⟩",
        "⟨s|[e]⟩ (sp|(s|n0 apply)) ⇓ ",
        // "⟨s|[e]⟩ (sp|(s|n1 apply)) ⇓ e",
        // "⟨s|[e]⟩ (sp|(s|n2 apply)) ⇓ e e",
        // "⟨s|[e]⟩ (sp|(s|n3 apply)) ⇓ e e e",
        // "⟨s|[e]⟩ (sp|(s|n4 apply)) ⇓ e e e e",
        // "⟨s|[e] n0⟩ (sp|(s|succ apply)) ⇓ e",
        // "⟨s|[e] n0⟩ (sp|(s|succ succ apply)) ⇓ e e",
        // "⟨s|[e] n0⟩ (sp|(s|succ succ succ apply)) ⇓ e e e",
        // "⟨s|[e] n1⟩ (sp|(s|succ apply)) ⇓ e e",
        // "⟨s|[e] n2⟩ (sp|(s|succ apply)) ⇓ e e e",
        // "⟨s|[e] n0 n0⟩ (sp|(s|add apply)) ⇓ ",
        // "⟨s|[e] n0 n1⟩ (sp|(s|add apply)) ⇓ e",
        // "⟨s|[e] n1 n0⟩ (sp|(s|add apply)) ⇓ e",
        // "⟨s|[e] n1 n1⟩ (sp|(s|add apply)) ⇓ e e",
        // "⟨s|[e] n1 n2⟩ (sp|(s|add apply)) ⇓ e e e",
        // "⟨s|[e] n2 n1⟩ (sp|(s|add apply)) ⇓ e e e",
        // "⟨s|[e] n2 n2⟩ (sp|(s|add apply)) ⇓ e e e e",
        // "⟨s|[e] n0 n0⟩ (sp|(s|mul apply)) ⇓ ",
        // "⟨s|[e] n0 n1⟩ (sp|(s|mul apply)) ⇓ ",
        // "⟨s|[e] n1 n0⟩ (sp|(s|mul apply)) ⇓ ",
        // "⟨s|[e] n1 n1⟩ (sp|(s|mul apply)) ⇓ e",
        // "⟨s|[e] n1 n2⟩ (sp|(s|mul apply)) ⇓ e e",
        // "⟨s|[e] n2 n1⟩ (sp|(s|mul apply)) ⇓ e e",
        // "⟨s|[e] n2 n2⟩ (sp|(s|mul apply)) ⇓ e e e e",
        // "[clone apply] (sp|(s|clone apply)) ⇓ ⟨s|[clone apply]⟩ clone apply",
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
                    panic!("Failed on {}", case);
                }
            };
            println!(
                "⟶{:?} {} {}",
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
