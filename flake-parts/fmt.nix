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
          taplo = {
            enable = true;
            settings.formatting = {
              reorder_keys = true;
              reorder_arrays = true;
              reorder_inline_tables = true;
              allowed_blank_lines = 1;
            };
          };
        };
        settings.on-unmatched = "fatal";
      };
    };
  };
}
