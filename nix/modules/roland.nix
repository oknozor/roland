# Home-manager module for roland
{ config, lib, pkgs, ... }:

let
  cfg = config.services.roland;
  toml = pkgs.formats.toml { };
in {
  options.services.roland = {
    enable = lib.mkEnableOption "roland touch gesture daemon";

    package = lib.mkOption {
      type = lib.types.package;
      default = pkgs.roland;
      defaultText = lib.literalExpression "pkgs.roland";
      description = "The roland package to use.";
    };

    systemdTarget = lib.mkOption {
      type = lib.types.str;
      default = cfg.systemdTarget;
      description = "Systemd target to bind the roland service to.";
    };

    # Accept either a path to an existing file or an inline attrset that gets
    # serialised to TOML.  Using an attrset lets you keep your gestures in Nix.
    config = lib.mkOption {
      type = lib.types.either lib.types.path (lib.types.submodule {
        options.gestures = lib.mkOption {
          type = lib.types.listOf lib.types.attrs;
          default = [];
          description = "List of gesture definitions (passed through to TOML as-is).";
        };
      });
      description = ''
        Roland configuration.  Either a path to a TOML file or an attrset
        that will be serialised to TOML automatically.
      '';
      example = lib.literalExpression ''
        {
          gestures = [
            {
              num_fingers = 1;
              kind = "SwipeRight";
              on_edge = { Left = 30; };
              min_distance = 50.0;
              action = "niri msg action focus-workspace-previous";
            }
          ];
        }
      '';
    };

    logLevel = lib.mkOption {
      type = lib.types.enum [ "error" "warn" "info" "debug" "trace" ];
      default = "info";
      description = "Log verbosity passed via RUST_LOG.";
    };
  };

  config = lib.mkIf cfg.enable {
    systemd.user.services.roland =
      let
        configFile =
          if lib.isPath cfg.config || lib.isString cfg.config
          then cfg.config
          else toml.generate "roland-config.toml" cfg.config;
      in {
        Unit = {
          Description = "Roland touch gesture daemon";
          After = [ cfg.systemdTarget ];
          PartOf = [ cfg.systemdTarget ];
        };

        Service = {
          ExecStart = "${lib.getExe cfg.package} --config ${configFile}";
          Environment = [ "RUST_LOG=${cfg.logLevel}" ];
          Restart = "on-failure";
          RestartSec = "2s";
        };

        Install.WantedBy = [ cfg.systemdTarget ];
      };
  };
}
