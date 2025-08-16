{
  partitions.dev.module = {
    gitignore = [ "/target" ];
    perSystem =
      psArgs@{ pkgs, ... }:
      {
        make-shells.default = {
          inputsFrom = [ psArgs.config.packages.default ];
          packages = [
            pkgs.bacon
            pkgs.cargo-insta
            pkgs.rust-analyzer
          ];
          env = {
            RUST_LOG = "info";
            RUST_BACKTRACE = 1;
          };
        };
        treefmt = {
          programs.rustfmt.enable = true;
          settings.global.excludes = [
            "bin/tests/snapshots/*.snap"
          ];
        };
      };
  };
}
