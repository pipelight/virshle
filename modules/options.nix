{
  lib,
  config,
  inputs,
  pkgs,
  ...
}:
with lib; let
  moduleName = "virshle";
in {
  ## Options
  options.services.${moduleName} = {
    enable = mkEnableOption "Enable ${moduleName}.";

    user = mkOption {
      default = "root";
      type = types.str;
    };
    logLevel = mkOption {
      default = "info";
      type = types.enum ["error" "warn" "info" "debug" "trace"];
    };

    manageNetwork.enable = mkEnableOption "Configure host network to give VM network access";
    dpdk.enable = mkEnableOption "Configure host network to give VM network access";
  };
}
