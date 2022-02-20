let
  _ = !(a == b);
  # do not match here
  _ = !(a != b);
  _ = a != b;
in
  null
