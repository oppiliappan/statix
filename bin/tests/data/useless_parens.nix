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

  # LATER: binary exprs, function args etc.
in
  # parens around let body
  (null)
