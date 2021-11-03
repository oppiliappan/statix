let
  a = 2;
  y = "y";
in
{
  # trivial
  a = a;

  # don't lint
  x.y = y;
}

