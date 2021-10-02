{
  inputs = {
    utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nmattia/naersk";
    mozillapkgs = {
      url = "github:mozilla/nixpkgs-mozilla";
      flake = false;
    };
    flake-compat = {
      url = "github:edolstra/flake-compat";
      flake = false;
    };
  };

  outputs =
    { self
    , nixpkgs
    , utils
    , naersk
    , mozillapkgs
    , ...
    }:
    utils.lib.eachDefaultSystem (system:
    let
      pkgs = nixpkgs.legacyPackages."${system}";

      # Get a specific rust version
      mozilla = pkgs.callPackage (mozillapkgs + "/package-set.nix") { };
      chanspec = {
        date = "2021-03-31";
        channel = "nightly";
        sha256 = "oK5ebje09MRn988saJMT3Zze/tRE7u9zTeFPV1CEeLc="; # set zeros after modifying channel or date
      };

      rustChannel = mozilla.rustChannelOf chanspec;
      rust = rustChannel.rust;
      rust-src = rustChannel.rust-src;

      naersk-lib = naersk.lib."${system}".override {
        cargo = rust;
        rustc = rust;
      };

      nativeBuildInputs = with pkgs; [ ];

    in
    rec {
      packages.statix = naersk-lib.buildPackage {
        pname = "statix";
        version = "0.1.0";
        root = ./.;
        inherit nativeBuildInputs;
      };

      defaultPackage = packages.statix;
      apps.statix = utils.lib.mkApp {
        drv = packages.statix;
      };

      apps.check = {
        type = "app";
        program = "${pkgs.cargo-watch}/bin/cargo-watch";
      };

      defaultApp = apps.statix;
      devShell = pkgs.mkShell {
        nativeBuildInputs = nativeBuildInputs ++ [
          rust
          rust-src
          pkgs.rust-analyzer
          pkgs.rustfmt
          pkgs.cargo
          pkgs.cargo-watch
        ];
        RUST_SRC_PATH = "${rust-src}/lib/rustlib/src/rust/library";
        RUST_LOG = "info";
        RUST_BACKTRACE = 1;
      };
    });
}
