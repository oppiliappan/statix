{ inputs, ... }:
{
  imports = [ inputs.flake-parts.flakeModules.easyOverlay ];

  perSystem = psArgs: {
    overlayAttrs = {
      inherit (psArgs.config.packages) statix statix-vim;
    };
  };
}
