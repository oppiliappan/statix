let
  x = 2;
  y = 3;
  a = { "2" = y; };
in
[
  ${x}
  ${toString (x + y)}
  a.${toString x}

  # multiline test
  ${
    toString x
  }
]
