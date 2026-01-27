{ config, lib, pkgs, ... }:

let
  cfg = config.programs.qmpo;
in
{
  options.programs.qmpo = {
    enable = lib.mkEnableOption "qmpo directory:// URI handler";

    package = lib.mkOption {
      type = lib.types.package;
      default = pkgs.qmpo;
      defaultText = lib.literalExpression "pkgs.qmpo";
      description = "The qmpo package to use.";
    };
  };

  config = lib.mkIf cfg.enable {
    home.packages = [ cfg.package ];

    xdg.mimeApps = {
      enable = true;
      defaultApplications = {
        "x-scheme-handler/directory" = "qmpo.desktop";
      };
    };

    xdg.desktopEntries.qmpo = {
      name = "qmpo";
      comment = "Directory URI Handler";
      exec = "${cfg.package}/bin/qmpo %u";
      terminal = false;
      noDisplay = true;
      mimeType = [ "x-scheme-handler/directory" ];
    };
  };
}
