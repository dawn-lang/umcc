// Copyright (c) 2021 Scott J Maddox
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use crate::interp::{Interp, HELP};

struct TestSession(Vec<TestCommand>);

struct TestCommand {
    input: &'static str,
    start_output: &'static str,
    step_output: &'static [&'static str],
}

#[test]
fn test_interp() {
    let sessions = [
        TestSession(vec![TestCommand {
            input: "",
            start_output: "",
            step_output: &[][..],
        }]),
        TestSession(vec![TestCommand {
            input: "(s|)",
            start_output: " (s|)\n",
            step_output: &["", "⇓  \n"][..],
        }]),
        TestSession(vec![TestCommand {
            input: ":trace",
            start_output: "",
            step_output: &[][..],
        }]),
        TestSession(vec![TestCommand {
            input: ":trace (s|)",
            start_output: " (s|)\n",
            step_output: &["‒StkCtxEmpty⟶  \n", ""][..],
        }]),
        TestSession(vec![
            TestCommand {
                input: ":show foo",
                start_output: "Not defined.\n",
                step_output: &[][..],
            },
            TestCommand {
                input: "{term foo=}",
                start_output: "Defined `foo`.\n",
                step_output: &[][..],
            },
            TestCommand {
                input: ":show foo",
                start_output: "{term foo = }\n",
                step_output: &[][..],
            },
            TestCommand {
                input: "{term foo=drop}",
                start_output: "Redefined `foo`.\n",
                step_output: &[][..],
            },
            TestCommand {
                input: ":show foo",
                start_output: "{term foo = drop}\n",
                step_output: &[][..],
            },
        ]),
        TestSession(vec![
            TestCommand {
                input: ":clear",
                start_output: "Definitions cleared.\n",
                step_output: &[][..],
            },
            TestCommand {
                input: ":list",
                start_output: "\n",
                step_output: &[][..],
            },
            TestCommand {
                input: "{term foo=}",
                start_output: "Defined `foo`.\n",
                step_output: &[][..],
            },
            TestCommand {
                input: ":list",
                start_output: "foo\n",
                step_output: &[][..],
            },
            TestCommand {
                input: "{term bar=}",
                start_output: "Defined `bar`.\n",
                step_output: &[][..],
            },
            TestCommand {
                input: ":list",
                start_output: "bar foo\n",
                step_output: &[][..],
            },
            TestCommand {
                input: ":clear",
                start_output: "Definitions cleared.\n",
                step_output: &[][..],
            },
            TestCommand {
                input: ":list",
                start_output: "\n",
                step_output: &[][..],
            },
        ]),
        TestSession(vec![
            TestCommand {
                input: "(sp|(s|[foo]))",
                start_output: " (sp|(s|[foo]))\n",
                step_output: &["", "⇓ ⟨s|[foo]⟩ \n"][..],
            },
            TestCommand {
                input: "(sp|(s|[bar]))",
                start_output: "⟨s|[foo]⟩ (sp|(s|[bar]))\n",
                step_output: &["", "⇓ ⟨s|[foo] [bar]⟩ \n"][..],
            },
            TestCommand {
                input: ":drop",
                start_output: "Values dropped.\n",
                step_output: &[][..],
            },
            TestCommand {
                input: "(sp|(s|[foobar]))",
                start_output: " (sp|(s|[foobar]))\n",
                step_output: &["", "⇓ ⟨s|[foobar]⟩ \n"][..],
            },
        ]),
        TestSession(vec![TestCommand {
            input: ":reset",
            start_output: "Reset.\n",
            step_output: &[][..],
        }]),
        TestSession(vec![TestCommand {
            input: ":help",
            start_output: HELP,
            step_output: &[][..],
        }]),
        TestSession(vec![TestCommand {
            input: "(sp|(s|clone))",
            start_output: " (sp|(s|clone))\n",
            step_output: &["⇓  (sp|(s|clone))\nExpected 1 values. Found 0.\n"][..],
        }]),
        TestSession(vec![TestCommand {
            input: "(sp|(s|foo))",
            start_output: " (sp|(s|foo))\n",
            step_output: &["⇓  (sp|(s|foo))\nUndefined term: `foo`.\n"][..],
        }]),
        TestSession(vec![TestCommand {
            input: "(s|foo)",
            start_output: " (_|(s|foo))\n",
            step_output: &["⇓  (_|(s|foo))\nUndefined term: `foo`.\n"][..],
        }]),
        TestSession(vec![TestCommand {
            input: "foo",
            start_output: " (__|(_|foo))\n",
            step_output: &["⇓  (__|(_|foo))\nUndefined term: `foo`.\n"][..],
        }]),
        TestSession(vec![TestCommand {
            input: "(sp|(s|true foo))",
            start_output: " (sp|(s|true foo))\n",
            step_output: &[
                "",
                "",
                "",
                "⇓ ⟨s|true⟩ (sp|(s|foo))\nUndefined term: `foo`.\n",
            ][..],
        }]),
        TestSession(vec![TestCommand {
            input: ":trace (sp|(s|true foo))",
            start_output: " (sp|(s|true foo))\n",
            step_output: &[
                "‒StkCtxDistr⟶  (sp|(s|true) (s|foo))\n",
                "‒StkCtxDistr⟶  (sp|(s|true)) (sp|(s|foo))\n",
                "‒LitCallQuote⟶ ⟨s|true⟩ (sp|(s|foo))\n",
                "Undefined term: `foo`.\n",
            ][..],
        }]),
        TestSession(vec![TestCommand {
            input: "(sp|(s|false false or))",
            start_output: " (sp|(s|false false or))\n",
            step_output: &[
                "",
                "",
                "",
                "",
                "",
                "",
                "",
                "",
                "",
                "",
                "",
                "",
                "⇓ ⟨s|false⟩ \n",
            ][..],
        }]),
        TestSession(vec![TestCommand {
            input: ":trace (sp|(s|false false or))",
            start_output: " (sp|(s|false false or))\n",
            step_output: &[
                "‒StkCtxDistr⟶  (sp|(s|false) (s|false or))\n",
                "‒StkCtxDistr⟶  (sp|(s|false)) (sp|(s|false or))\n",
                "‒LitCallQuote⟶ ⟨s|false⟩ (sp|(s|false or))\n",
                "‒StkCtxDistr⟶ ⟨s|false⟩ (sp|(s|false) (s|or))\n",
                "‒StkCtxDistr⟶ ⟨s|false⟩ (sp|(s|false)) (sp|(s|or))\n",
                "‒LitCallQuote⟶ ⟨s|false false⟩ (sp|(s|or))\n",
                "‒LitCall⟶ ⟨s|false false⟩ (sp|(s|clone apply))\n",
                "‒StkCtxDistr⟶ ⟨s|false false⟩ (sp|(s|clone) (s|apply))\n",
                "‒StkCtxDistr⟶ ⟨s|false false⟩ (sp|(s|clone)) (sp|(s|apply))\n",
                "‒IntrClone⟶ ⟨s|false false false⟩ (sp|(s|apply))\n",
                "‒IntrApply⟶ ⟨s|false false⟩ (sp|(s|drop))\n",
                "‒IntrDrop⟶ ⟨s|false⟩ \n",
                "",
            ][..],
        }]),
        TestSession(vec![TestCommand {
            input: "(sp|(s|n0 succ))",
            start_output: " (sp|(s|n0 succ))\n",
            step_output: &["", "", "", "", "", "", "", "", "", "", "", "⇓ ⟨s|n1⟩ \n"][..],
        }]),
        TestSession(vec![TestCommand {
            input: "(s|(s|))",
            start_output: " (s|(s'1|))\n",
            step_output: &["", "", "⇓  \n"][..],
        }]),
    ];
    let mut buffer = Vec::with_capacity(4096);
    for session in sessions {
        let mut interp = Interp::default();
        for command in session.0 {
            buffer.clear();
            interp.interp_start(command.input, &mut buffer).unwrap();
            let output = unsafe { std::str::from_utf8_unchecked(&buffer[..]) };
            assert_eq!(
                output, command.start_output,
                "Failed at start on input {:?}",
                command.input
            );
            let mut step = 0;
            while !interp.is_done() {
                buffer.clear();
                interp.interp_step(&mut buffer).unwrap();
                let output = unsafe { std::str::from_utf8_unchecked(&buffer[..]) };
                assert!(
                    step < command.step_output.len(),
                    "Missing expected output for step {} on input {:?}",
                    step,
                    command.input
                );
                assert_eq!(
                    output, command.step_output[step],
                    "Failed at step {} on input {:?}",
                    step, command.input
                );
                step += 1;
            }
            assert_eq!(step, command.step_output.len(), "Expected more steps.")
        }
    }
}
