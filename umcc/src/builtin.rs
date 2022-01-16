// Copyright (c) 2021 Scott J Maddox
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

pub(crate) static TERM_DEF_SRCS: [&'static str; 21] = [
    "{term swap = (s1|push)(s2|push)(s1|pop)(s2|pop)}",
    "{term v1 = []}",
    "{term v2 = []}",
    "{term v3 = []}",
    "{term v4 = []}",
    "{term false = [drop]}",
    "{term true = [swap drop]}",
    "{term or = clone apply}",
    "{term quote2 = quote swap quote swap compose}",
    "{term quote3 = quote2 swap quote swap compose}",
    "{term rotate3 = quote2 swap quote compose apply}",
    "{term rotate4 = quote3 swap quote compose apply}",
    "{term _succ = (n|push) [clone] (n|clone pop) apply [compose] (n|pop) apply apply}",
    "{term n0 = [drop]}",
    "{term n1 = [n0 _succ]}",
    "{term n2 = [n1 _succ]}",
    "{term n3 = [n2 _succ]}",
    "{term n4 = [n3 _succ]}",
    "{term succ = quote [_succ] compose}",
    "{term add = [succ] swap apply}",
    "{term mul = n0 rotate3 quote [add] compose rotate3 apply}",
];
