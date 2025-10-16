mod _utils;

use macros::generate_tests;

generate_tests! {
    rule: eta_reduction,
    expressions: [
        "let double = x: x * 2; in map (x: double x) [1 2 3]",

        // don't lint on non-free exprs
        "let f = { double = x: x *2; val = 2; }; in map (f: f.double f.val) [ f ]",

        // other non-free forms
        "let f = { double = x: x *2; val = 2; }; in map (f: {inherit f;}.double f.val) [ f ]",

        // don't reduce on more complex lambda bodies
        "map (x: builtins.div 3 x) [1 2 3]",
    ],
}
