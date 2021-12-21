// Copyright (c) 2021 Scott J Maddox
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

pub(crate) static TERM_DEF_SRCS: [&'static str; 24] = [
    "term swap = (s1|push)(s2|push)(s1|pop)(s2|pop);",
    "term v1 = [];",
    "term v2 = [];",
    "term v3 = [];",
    "term v4 = [];",
    "term false = [drop];",
    "term true = [swap drop];",
    "term or = clone apply;",
    "term quote2 = quote swap quote swap compose;",
    "term quote3 = quote2 swap quote swap compose;",
    "term rotate3 = quote2 swap quote compose apply;",
    "term rotate4 = quote3 swap quote compose apply;",
    "term compose2 = compose;",
    "term compose3 = compose compose2;",
    "term compose4 = compose compose3;",
    "term compose5 = compose compose4;",
    "term n0 = [drop];",
    "term n1 = [[clone] n0 apply [compose] n0 apply apply];",
    "term n2 = [[clone] n1 apply [compose] n1 apply apply];",
    "term n3 = [[clone] n2 apply [compose] n2 apply apply];",
    "term n4 = [[clone] n3 apply [compose] n3 apply apply];",
    "term succ = quote [apply] compose [[clone]] swap clone [[compose]] swap [apply] compose5;",
    "term add = [succ] swap apply;",
    "term mul = n0 rotate3 quote [add] compose rotate3 apply;",
];
