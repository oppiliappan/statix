{
  # trivial case
  _ = lib.zipAttrsWith (name: values: values) [{ a = 1; } { a = 2; b = 3; }];

  # offer lint heuristically on this too
  _ = nixpkgs.lib.zipAttrsWith (name: values: values) [{ a = 1; } { a = 2; b = 3; }];

  # do not lint on `builtins`
  _ = builtins.zipAttrsWith (name: values: values) [
    { a = 1; }
    { a = 2; b = 3; }
  ];
}
