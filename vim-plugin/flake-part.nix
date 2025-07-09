{ lib, ... }:
{
  perSystem =
    { pkgs, ... }:
    {
      packages.statix-vim = pkgs.vimUtils.buildVimPlugin {
        pname = "statix-vim";
        version = "0.1.0";
        src = lib.fileset.toSource {
          root = ./.;
          fileset = lib.fileset.union ./plugin/statix.vim ./ftplugin/nix.vim;
        };
      };
    };

  partitions.dev.module.perSystem = psArgs: {
    treefmt.settings.global.excludes = [ "*.vim" ];
    checks."packages/statix-vim" = psArgs.config.packages.statix-vim;
  };
}
