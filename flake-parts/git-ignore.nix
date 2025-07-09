{ lib, ... }:
{
  partitions.dev.module = devPartition: {
    options.gitignore = lib.mkOption {
      type = lib.types.listOf lib.types.singleLineStr;
      apply = lib.flip lib.pipe [
        lib.naturalSort
        lib.concatLines
      ];
    };
    config = {
      gitignore = [ "/result" ];

      perSystem =
        { pkgs, ... }:
        {
          files.files = [
            {
              path_ = ".gitignore";
              drv = pkgs.writeText ".gitignore" devPartition.config.gitignore;
            }
          ];
        };
    };
  };
}
