{
  # trivial case
  _ = lib.groupBy (x: if x > 2 then "big" else "small") [ 1 2 3 4 5 ];

  # offer lint heuristically on this too
  _ = nixpkgs.lib.groupBy (x: if x > 2 then "big" else "small") [ 1 2 3 4 5 ];

  # do not lint on `builtins`
  _ = builtins.groupBy (x: x.name) [
    { name = "foo"; idx = 1; }
    { name = "foo"; idx = 2; }
    { name = "bar"; idx = 1; }
    { name = "bar"; idx = 2; }
  ];
}
