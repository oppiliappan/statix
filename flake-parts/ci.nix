{ lib, ... }:
let
  filePaths = {
    check = ".github/workflows/check.yaml";
    updateFlakeLock = ".github/workflows/update-flake-lock.yaml";
  };

  ids = {
    jobs = {
      getCheckNames = "get-check-names";
      check = "check";
    };
    steps.getCheckNames = "get-check-names";
    outputs = {
      jobs.getCheckNames = "checks";
      steps.getCheckNames = "checks";
    };
  };

  matrixParam = "checks";

  nixArgs = "--accept-flake-config";

  runner = {
    name = "ubuntu-latest";
    system = "x86_64-linux";
  };

  steps = {
    checkout.uses = "actions/checkout@v4";
    cachixInstallNix = {
      uses = "cachix/install-nix-action@v31";
      "with".github_access_token = "\${{ secrets.GITHUB_TOKEN }}";
    };
  };
in
{
  partitions.dev.module = {
    perSystem =
      { pkgs, ... }:
      {
        files.files = [
          {
            path_ = filePaths.check;
            drv = pkgs.writers.writeJSON "gh-actions-workflow-check.yaml" {
              name = "Check";
              on = {
                pull_request = { };
                push = { };
                workflow_call = { };
              };
              jobs = {
                ${ids.jobs.getCheckNames} = {
                  runs-on = runner.name;
                  outputs.${ids.outputs.jobs.getCheckNames} = "\${{ steps.${ids.steps.getCheckNames}.outputs.${ids.outputs.steps.getCheckNames} }}";
                  steps = [
                    steps.checkout
                    steps.cachixInstallNix
                    {
                      id = ids.steps.getCheckNames;
                      run = ''
                        checks="$(nix ${nixArgs} eval --json .#checks.${runner.system} --apply builtins.attrNames)"
                        echo "${ids.outputs.steps.getCheckNames}=$checks" >> $GITHUB_OUTPUT
                      '';
                    }
                  ];
                };

                ${ids.jobs.check} = {
                  needs = ids.jobs.getCheckNames;
                  runs-on = runner.name;
                  strategy.matrix.${matrixParam} = "\${{ fromJson(needs.${ids.jobs.getCheckNames}.outputs.${ids.outputs.jobs.getCheckNames}) }}";
                  steps = [
                    steps.checkout
                    steps.cachixInstallNix
                    {
                      run = ''
                        nix ${nixArgs} build '.#checks.${runner.system}."''${{ matrix.${matrixParam} }}"'
                      '';
                    }
                  ];
                };

                legacy = {
                  name = "Build statix via flake-compat and install it using `nix-env`";
                  runs-on = runner.name;
                  steps = [
                    steps.checkout
                    steps.cachixInstallNix
                    { run = "nix-env --install --file default.nix"; }
                  ];
                };
              };
            };
          }
          {
            path_ = filePaths.updateFlakeLock;
            drv = pkgs.writers.writeJSON "update-flake-lock.yaml" {
              name = "Update flake.lock";
              on = {
                workflow_dispatch = { };
                schedule = [ { cron = "0 0 * * 5"; } ];
              };
              jobs.nix-flake-update = {
                permissions = {
                  contents = "write";
                  id-token = "write";
                  issues = "write";
                  pull-requests = "write";
                };
                runs-on = runner.name;
                steps = [
                  steps.checkout
                  steps.cachixInstallNix
                  { uses = "DeterminateSystems/update-flake-lock@main"; }
                ];
              };
            };
          }
        ];

        treefmt.settings.global.excludes = lib.attrValues filePaths;
      };
  };
}
