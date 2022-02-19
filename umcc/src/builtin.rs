// Copyright (c) 2021 Scott J Maddox
//
// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

pub(crate) static TERM_DEF_SRCS: [&'static str; 22] = [
    "{term swap = (s1|push)(s2|push)(s1|pop)(s2|pop)}",
    "{term compose2 = compose}",
    "{term compose3 = compose compose2}",
    "{term quote0 = []}",
    "{term quote1 = quote}",
    "{term quote2 =
        (s2|push) (s1|push)
        (s1|quote pop) (s2|quote pop)
        compose2
    }",
    "{term quote3 =
        (s3|push) (s2|push) (s1|push)
        (s1|quote pop) (s2|quote pop)  (s3|quote pop)
        compose3
    }",
    "{term False = quote0 [_False] compose}",
    "{term True  = quote0 [_True]  compose}",
    "{term _False = (case_False|pop)  (case_True|drop) apply}",
    "{term _True  = (case_False|drop) (case_True|pop)  apply}",
    "{term not =
        (case_False|[True])
        (case_True|[False])
        apply
    }",
    "{term or =
        (case_False|[
            (case_False|[False])
            (case_True|[True])
            apply
        ])
        (case_True|[drop True])
        apply
    }",
    "{term and =
        (case_False|[drop False])
        (case_True|[
            (case_False|[False])
            (case_True|[True])
            apply
        ])
        apply
    }",
    "{term Z = quote0 [_Z] compose}",
    "{term S = quote1 [_S] compose}",
    "{term _Z = (case_Z|pop)  (case_S|drop) apply}",
    "{term _S = (case_Z|drop) (case_S|pop)  apply}",
    "{term succ = S}",
    "{term add =
        (case_Z|[])
        (case_S|[(b|push) succ (b|pop) add])
        apply
    }",
    "{term mul = (_|push push) Z (_|pop pop) _mul}",
    "{term _mul =
        (case_Z|[drop])
        (case_S|[(b|push) clone (a|push) add (a|pop) (b|pop) _mul])
        apply
    }",
];
