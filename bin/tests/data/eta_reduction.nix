let
  double = x: x * 2;
  inherit (builtins) map;
  xs = [ 1 2 3 ];
  f = {
    inherit double;
    val = 2;
  };
in
[
  (map (x: double x) xs)

  # don't lint on non-free exprs
  (map (f: f.double f.val) [ f ])

  # other non-free forms
  (map (f: {inherit f;}.double f.val) [ f ])

  # don't reduce on more complex lambda bodies
  (map (x: builtins.div 3 x) xs)
]
