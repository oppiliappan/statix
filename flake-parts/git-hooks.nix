{
  partitions.dev.module = devPartition: {
    imports = [ devPartition.inputs.git-hooks.flakeModule ];

    gitignore = [
      "/.pre-commit-config.yaml"
    ];

    perSystem =
      { config, ... }:
      {
        make-shells.default.shellHook = config.pre-commit.installationScript;
        pre-commit.check.enable = false;
        treefmt.settings.global.excludes = [ ".pre-commit-config.yaml" ];
      };

  };
}
