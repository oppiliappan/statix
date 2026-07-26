mod _utils;

use indoc::indoc;

use macros::generate_tests;

generate_tests! {
    rule: repeated_keys,
    expressions: [
        // fine
        "{ foo.bar = 1; }",

        // do not raise on rec
        indoc! {"
            rec {
              foo.x = foo.y;
              foo.y = 2;
              foo.z = 3;
            }
        "},

        // exactly 3 occurrences
        indoc! {r#"
            {
              foo.bar = 1;
              foo.bar."hello" = 1;
              foo.again = 1;
            }
        "#},

        // more than 3, omit the extra
        indoc! {"
            {
              foo.baz.bar1 = 1;
              foo.baz.bar2 = 2;
              foo.baz.bar3 = 3;
              foo.baz.bar4 = 4;
              foo.baz.bar5 = 5;
            }
        "},

        // non-contiguous entries should still be grouped into a fix
        indoc! {"
            {
              hardware.bluetooth = {
                enable = true;
              };
              networking.hostName = \"nixbox\";
              hardware.nvidia-container-toolkit.enable = true;
              hardware.nvidia = {
                modesetting.enable = true;
              };
            }
        "},
    ],
}

#[test]
fn repeated_keys_fix_removes_nested_repeated_key_warnings() {
    let expression = indoc! {r#"
        {
          services.pcscd.enable = true;
          services.xserver.xkb = {
            layout = "us";
            variant = "";
          };
          services.swapspace.enable = true;
          services.xserver.videoDrivers = [ "nvidia" ];
          services.xserver.enable = true;

          environment.plasma6.excludePackages = [ ];
          environment.etc."environment.d/desktop-environment.conf".text = ''
            DESKTOP_SESSION=plasma
          '';
          environment.variables.KWIN_DRM_PREFER_COLOR_DEPTH = "24";

          programs.dconf.enable = true;
          programs.nix-ld.enable = true;
          programs.nix-ld.libraries = [ ];

          security.rtkit.enable = true;
          security.polkit.enable = true;
          security.sudo.package = pkgs.sudo;
        }
    "#};

    let (_, fixed, check_output) =
        _utils::apply_and_check(expression, &["fix"], &["check"]).unwrap();

    assert!(
        !check_output.contains("Avoid repeated keys in attribute sets"),
        "repeated_keys warning remained after fix\nfixed:\n{fixed}\ncheck output:\n{check_output}",
    );
}
