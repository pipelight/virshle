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

    # Virshle only runs well as root.
    # This options sets the user environment and permissions.
    user = mkOption {
      default = "root";
      type = types.str;
    };

    # Wether to manage host network interface.
    manageNetwork.enable = mkEnableOption "Configure host network to give VM network access";
    dpdk.enable = mkEnableOption "Enable dpdk only interfaces (warning: not compatible with system interfaces.)";
    afxdp.enable = mkEnableOption "Enable AF_XDP socket support through eBPF";
  };
}
