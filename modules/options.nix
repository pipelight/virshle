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
    logLevel = mkOption {
      default = "info";
      type = types.enum ["error" "warn" "info" "debug" "trace"];
    };

    # Options is silenced because it needs FFI binding
    # for linux network functions.
    # So as of now, virshle only runs well as root.
    # user = mkOption {
    #   default = "root";
    #   type = types.str;
    # };

    # Wether to manage host network interface.
    manageNetwork.enable = mkEnableOption "Configure host network to give VM network access";
    dpdk.enable = mkEnableOption "Enable dpdk only interfaces (warning: not compatible with system interfaces.)";
  };
}
