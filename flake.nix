{
  inputs = {

    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";

    mozillapkgs = {
      url = "github:mozilla/nixpkgs-mozilla";
      flake = false;
    };

    import-cargo.url = github:edolstra/import-cargo;

  };

  outputs =
    { self
    , nixpkgs
    , mozillapkgs
    , import-cargo
    , ...
    }:
    let
      inherit (import-cargo.builders) importCargo;

      supportedSystems = [ "x86_64-linux" ];
      forAllSystems = nixpkgs.lib.genAttrs supportedSystems;
      nixpkgsFor = forAllSystems (system:
        import nixpkgs {
          inherit system;
          overlays = [ self.overlay ];
        });

      mozilla = p: p.callPackage (mozillapkgs + "/package-set.nix") { };
      chanspec = {
        date = "2021-09-30";
        channel = "nightly";
        sha256 = "Elqn7GDBDE/QT1XTDyj0EvivbC//uwjWX8d+J3Pi0dY="; # set zeros after modifying channel or date
      };
      rustChannel = p: (mozilla p).rustChannelOf chanspec;

    in
    {

      overlay = final: prev:
        let
          inherit (rustChannel final.pkgs) rust rust-src;
        in
        {

          statix = with final; pkgs.stdenv.mkDerivation {
            pname = "statix";
            version = "v0.3.0";
            src = builtins.path {
              path = ./.;
              name = "statix";
            };
            nativeBuildInputs = [
              (importCargo { lockFile = ./Cargo.lock; inherit pkgs; }).cargoHome
              rust
              cargo
            ];
            buildPhase = ''
              cargo build -p statix --all-features --release --offline
            '';
            # statix does not have any tests currently
            doCheck = false;
            installPhase = ''
              install -Dm775 ./target/release/statix $out/bin/statix
            '';

            meta = with pkgs.lib; {
              description = "Lints and suggestions for the Nix programming language";
              homepage = "https://git.peppe.rs/languages/statix/about";
              license = licenses.mit;
            };
          };

          statix-vim =
            with final; pkgs.vimUtils.buildVimPlugin {
              pname = "statix-vim";
              version = "0.1.0";
              src = ./vim-plugin;
            };

        };

      packages = forAllSystems (system: {
        inherit (nixpkgsFor."${system}") statix statix-vim;
      });

      defaultPackage =
        forAllSystems (system: self.packages."${system}".statix);

      defaultApp = forAllSystems (system:
        {
          type = "app";
          program = "${self.packages."${system}".statix}/bin/statix";
        });

      devShell = forAllSystems (system:
        let
          pkgs = nixpkgsFor."${system}";
          inherit (rustChannel pkgs) rust rust-src rust-analysis;
        in
        with pkgs;
        mkShell rec {
          buildInputs = [
            rustfmt
            cargo
            cargo-watch
            rust
            rust-src
          ];
          RUST_SRC_PATH = "${rust-src}/lib/rustlib/src/rust/library";
          RUST_LOG = "info";
          RUST_BACKTRACE = 1;
        });


    };
}
