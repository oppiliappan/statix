{
  partitionedAttrs.formatter = "dev";
  partitions.dev.module = devPartition: {
    imports = [ devPartition.inputs.treefmt.flakeModule ];

    perSystem = psArgs: {
      pre-commit.settings.hooks.treefmt.enable = true;

      treefmt = {
        projectRootFile = "flake.nix";
        programs = {
          nixfmt.enable = true;
          prettier.enable = true;
          # https://github.com/pappasam/toml-sort/issues/62
          # toml-sort = {
          #   enable = true;
          #   all = true;
          # };
        };
        settings.on-unmatched = "fatal";
      };
    };
  };
}
