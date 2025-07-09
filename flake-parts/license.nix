{ lib, ... }:
{
  _module.args.license = lib.licenses.mit;
  partitions.dev.module.perSystem.treefmt.settings.global.excludes = [ "LICENSE" ];
}
