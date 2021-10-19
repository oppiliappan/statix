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
    in
    {

      overlay = final: prev:
        let
          rustChannel = (mozilla final.pkgs).rustChannelOf chanspec;
          inherit (rustChannel) rust rustc rust-src;
        in
        {

          statix = with final; pkgs.stdenv.mkDerivation {
            pname = "statix";
            version = "v0.1.0";
            src = ./.;
            nativeBuildInputs = [
              (importCargo { lockFile = ./Cargo.lock; inherit pkgs; }).cargoHome
              rust
              cargo
            ];
            buildPhase = ''
              cargo build -p statix --release --offline
            '';
            installPhase = ''
              install -Dm775 ./target/release/statix $out/bin/statix
            '';
          };

        };

      packages = forAllSystems (system: {
        inherit (nixpkgsFor."${system}") statix;
      });

      defaultPackage =
        forAllSystems (system: self.packages."${system}".statix);

      devShell = forAllSystems (system:
        let
          pkgs = nixpkgsFor.${system};
          rustChannel = (mozilla pkgs).rustChannelOf chanspec;
        in
        with pkgs;
        mkShell rec {
          buildInputs =
            (with pkgs; [
              rust-analyzer
              rustfmt
              cargo
              cargo-watch
            ]) ++ (with rustChannel; [
              rust
              rust-src
            ]);
          RUST_SRC_PATH = "${rustChannel.rust-src}/lib/rustlib/src/rust/library";
          RUST_LOG = "info";
          RUST_BACKTRACE = 1;
        });


    };
}
