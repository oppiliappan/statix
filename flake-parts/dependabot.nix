let
  filePath = ".github/dependabot.yml";
in
{
  partitions.dev.module = {
    perSystem =
      { pkgs, ... }:
      {
        files.files = [
          {
            path_ = filePath;
            drv = pkgs.writers.writeJSON "dependabot.yml" {
              version = 2;
              updates = [
                {
                  package-ecosystem = "cargo";
                  directory = "/";
                  schedule.interval = "daily";
                  groups.everything.patterns = [ "*" ];
                  commit-message = {
                    prefix = "chore";
                    include = "scope";
                  };
                }
              ];
            };
          }
        ];
        treefmt.settings.global.excludes = [ filePath ];
      };
  };
}
