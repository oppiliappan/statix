{
  inputs = {

    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };

  };

  outputs =
    { self
    , nixpkgs
    , fenix
    }:
    let
      supportedSystems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];
      forAllSystems = nixpkgs.lib.genAttrs supportedSystems;
      nixpkgsFor = forAllSystems (system:
        import nixpkgs {
          inherit system;
          overlays = [ self.overlays.default ];
        });

      rustChannel = p: (fenix.overlays.default p p).fenix.stable;
    in
    {

      overlays.default = final: prev: {

        statix = with final;
          let
            pname = "statix";
            packageMeta = (lib.importTOML ./bin/Cargo.toml).package;
            rustPlatform = makeRustPlatform {
              inherit (rustChannel final) cargo rustc;
            };
          in
          rustPlatform.buildRustPackage {
            inherit pname;
            inherit (packageMeta) version;

            src = self;
            cargoLock.lockFile = ./Cargo.lock;

            buildFeatures = [ "json" ];

            meta = with lib; {
              description = "Lints and suggestions for the Nix programming language";
              homepage = "https://git.peppe.rs/languages/statix/about";
              license = licenses.mit;
            };
          };

        statix-vim =
          with final; vimUtils.buildVimPlugin {
            pname = "statix-vim";
            version = "0.1.0";
            src = ./vim-plugin;
          };

      };

      packages = forAllSystems (system: {
        inherit (nixpkgsFor."${system}") statix statix-vim;
        default = (nixpkgsFor."${system}").statix;
      });

      devShells = forAllSystems (system: {
        default = let
          pkgs = nixpkgsFor."${system}";
          toolchain = (rustChannel pkgs).withComponents [
            "rustc"
            "cargo"
            "rust-std"
            "rustfmt"
            "clippy"
            "rust-src"
          ];
          inherit (fenix.packages."${system}") rust-analyzer;
        in
        pkgs.mkShell {
          nativeBuildInputs = [
            pkgs.bacon
            pkgs.cargo-insta
            rust-analyzer
            toolchain
          ];
          RUST_LOG = "info";
          RUST_BACKTRACE = 1;
        };
      });


      apps = forAllSystems
        (system:
          let
            pkgs = nixpkgsFor."${system}";
            cachix-push-script = pkgs.writeScriptBin "cachix-push" ''
              ${pkgs.nix}/bin/nix build --json \
              | ${pkgs.jq}/bin/jq -r '.[].outputs | to_entries[].value' \
              | ${pkgs.cachix}/bin/cachix push statix
            '';
          in
          {
            cachix-push = {
              type = "app";
              program = "${cachix-push-script}/bin/cachix-push";
            };
          }
        );

    };
}
