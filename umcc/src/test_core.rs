// Copyright (c) 2021 Scott J Maddox
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::core::*;
use crate::display::*;
use crate::parse::*;

#[test]
fn test_define_term() {
    let mut ctx = Context::default();
    let sym = TermSymbol(ctx.interner.get_or_intern_static("foo"));
    let fn_def1 = TermDefParser::new()
        .parse(&mut ctx.interner, "term foo = e1;")
        .unwrap();
    let e1 = ExprParser::new().parse(&mut ctx.interner, "e1").unwrap();
    let fn_def2 = TermDefParser::new()
        .parse(&mut ctx.interner, "term foo = e2;")
        .unwrap();
    let e2 = ExprParser::new().parse(&mut ctx.interner, "e2").unwrap();
    assert_eq!(ctx.terms.get(&sym), None);
    assert_eq!(ctx.define_term(fn_def1), None);
    assert_eq!(ctx.terms.get(&sym), Some(&e1));
    assert_eq!(ctx.define_term(fn_def2), Some(TermDef(sym, e1)));
    assert_eq!(ctx.terms.get(&sym), Some(&e2));
}

#[test]
fn test_small_step() {
    let cases = [
        "⟨s1|v1⟩⟨s2|v2⟩ (s1|(s2|push)) ⟶ ⟨s1|⟩⟨s2|v2 v1⟩ (s1|(s2|))",
        "⟨s1|v1⟩⟨s2|v2⟩ (s1|(s2|pop)) ⟶ ⟨s1|v1 v2⟩⟨s2|⟩ (s1|(s2|))",
        "⟨s|v⟩ (s1|(s|clone)) ⟶ ⟨s|v v⟩ (s1|(s|))",
        "⟨s|v⟩ (s1|(s|drop)) ⟶ ⟨s|⟩ (s1|(s|))",
        "⟨s|v⟩ (s1|(s|quote)) ⟶ ⟨s|[v]⟩ (s1|(s|))",
        "⟨s|[e1 e2] [e3 e4]⟩ (s1|(s|compose)) ⟶ ⟨s|[e1 e2 e3 e4]⟩ (s1|(s|))",
        "⟨s|[e]⟩ (s1|(s|apply)) ⟶ ⟨s|⟩ (s1|(s|e))",
        "⟨s|[e1 e2]⟩ (s1|(s|apply)) ⟶ ⟨s|⟩ (s1|(s|e1 e2))",
        "⟨s|⟩ (s1|(s|[e])) ⟶ ⟨s|[e]⟩ (s1|(s|))",
        "(s1|(s2|(s3|e))) ⟶ (s2|(s3|e))",
        "(s1|(s2|)) ⟶ ",
        "(s1|) ⟶ ",
        " ⟶ ",
    ];
    for case in cases {
        let mut ctx = Context::default();
        let (mut vms1, mut e1, vms2, e2) = SmallStepAssertionParser::new()
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
                Ok(()),
                vms2.resolve(&ctx.interner),
                e2.resolve(&ctx.interner)
            ),
            "Failed on {}",
            case
        );
    }
}
