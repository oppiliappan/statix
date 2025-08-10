{
  partitionedAttrs.formatter = "dev";
  partitions.dev.module = devPartition: {
    imports = [ devPartition.inputs.treefmt.flakeModule ];

    perSystem = psArgs: {
      pre-commit.settings.hooks.treefmt.enable = true;

      treefmt = {
        projectRootFile = "flake.nix";
        programs = {
          nixfmt = {
            enable = true;
            priority = 2;
          };
          prettier.enable = true;
          taplo = {
            enable = true;
            priority = 1;
            settings = {
              formatting = {
                align_entries = true; # Align entries vertically. Entries that have table headers, comments, or blank lines between them are not aligned.
                reorder_keys = true; # Alphabetically reorder keys that are not separated by blank lines.
                reorder_arrays = true; # Alphabetically reorder array values that are not separated by blank lines.
                reorder_inline_tables = true; # Alphabetically reorder inline tables.
              };
            };
          };
          statix = {
            enable = true;
            priority = 1;
            package = psArgs.config.packages.statix;
          };
        };
        settings.on-unmatched = "fatal";
      };
    };
  };
}
