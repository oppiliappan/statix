{
  nixConfig = {
    abort-on-warn = true;
    allow-import-from-derivation = false;
  };

  inputs = {
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    flake-parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };

    nixpkgs.url = "github:nixos/nixpkgs/nixpkgs-unstable";

    systems = {
      url = "github:nix-systems/default";
      flake = false;
    };
  };
  outputs =
    inputs:
    inputs.flake-parts.lib.mkFlake { inherit inputs; } (
      { lib, ... }:
      {
        _module.args.root = ./.;
        systems = import inputs.systems;

        imports = [
          inputs.flake-parts.flakeModules.partitions
          ./docs/flake-part.nix
          ./flake-parts/cachix.nix
          ./flake-parts/ci.nix
          ./flake-parts/dependabot.nix
          ./flake-parts/dev-shell.nix
          ./flake-parts/files.nix
          ./flake-parts/fmt.nix
          ./flake-parts/git-hooks.nix
          ./flake-parts/git-ignore.nix
          ./flake-parts/license.nix
          ./flake-parts/overlay.nix
          ./flake-parts/rust.nix
          ./flake-parts/statix.nix
          ./vim-plugin/flake-part.nix
        ];

        partitionedAttrs = lib.genAttrs [
          "checks"
          "apps"
        ] (_: "dev");

        partitions.dev.extraInputsFlake = ./dev-flake;
      }
    );
}
