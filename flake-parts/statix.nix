{
  lib,
  root,
  ...
}:
{
  perSystem =
    psArgs@{ pkgs, ... }:
    {
      packages = {
        statix = pkgs.rustPlatform.buildRustPackage {
          pname = "statix";
          inherit ((lib.importTOML (root + "/bin/Cargo.toml")).package) version;

          src = lib.fileset.toSource {
            inherit root;
            fileset = lib.fileset.unions [
              (lib.fileset.fileFilter (
                file:
                lib.any lib.id [
                  (file.name == "Cargo.toml")
                  (file.hasExt "rs")
                  (file.hasExt "snap")
                ]
              ) root)
              (root + "/Cargo.lock")
              (root + "/bin/tests/data")
            ];
          };
          cargoLock.lockFile = root + "/Cargo.lock";
          buildFeatures = [ "json" ];
          RUSTFLAGS = "--deny warnings";
          nativeCheckInputs = [ pkgs.clippy ];

          postCheck = ''
            echo "Starting postCheck"
            cargo clippy
            echo "Finished postCheck"
          '';

          meta = {
            mainProgram = "statix";
            description = "Lints and suggestions for the Nix programming language";
            homepage = "https://git.peppe.rs/languages/statix/about";
            license = lib.licenses.mit;
          };
        };

        default = psArgs.config.packages.statix;
      };
    };

  partitions.dev.module.perSystem = psArgs: {
    treefmt.settings.global.excludes = [ "bin/tests/data/*.nix" ];
    checks."packages/statix" = psArgs.config.packages.statix;
  };
}
