let
  # parens around primitives
  a = {
    b = ("hello");
    c = (d);
    e = ({ f = 2; });
  };

  # parens around let-value
  g = (1 + 2);
  h = ({ inherit i; });

  # binary exprs with superflous parens
  # TODO: we could implement associativity check to remove more redundant parens in the future
  f =
    let id = x: x; in
    (id [3])
    ++ (id [1] ++ [2])
  ;

  # binary exprs with necessary parens
  u =
    (1 + 1)
    * (2 + 2)
    ;

  # precedence
  prec1 =
    4 + (5 * 3)
    ;
  prec2 =
    (4 * 5) / 5
    ;
  prec3_no =
    4 * (5 / 5)
    ;

  # string concat
  s =
    (builtins.readFile ./x.txt)
    + (lib.optionalString true ''
      foo
    '')
    + (lib.optionalString true ''
      bar
    '')
    ;

  # TODO: function args etc.
in
  # parens around let body
  (null)
