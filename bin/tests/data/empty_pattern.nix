[
  # match
  ({ ... }: 42)

  # don't match
  ({ a, ... }: a)
  ({ ... } @ inputs: inputs)

  # nixos module, don't match
  ({ ... }: {
    imports = [
      ./module.nix
      /path/to/absolute/module.nix
    ];
  })
]

