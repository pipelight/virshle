{
  lib,
  config,
  pkgs,
  ...
}:
with lib; let
  moduleName = "virshle";
  cfg = config.services.${moduleName};
in
  mkIf cfg.enable {
    # OpenVSwitch
    virtualisation.vswitch = {
      package = pkgs.openvswitch;
      enable = true;
    };

    boot = {
      kernelModules = ["openvswitch"];
      kernel.sysctl = {
        "vm.nr_hugepages" = mkIf cfg.dpdk.enable (mkOptionDefault 4096);
      };
    };

    ## Module
    systemd.tmpfiles.rules = [
      # Loosen permissions on openvswitch.
      "Z '/var/run/openvswitch' 774 root users - -"
      "d '/var/run/openvswitch' 774 root users - -"
    ];

    systemd.services.ovsdb.serviceConfig.Group = "users";
    systemd.services.ovsdb.serviceConfig.ExecStartPost = [
      "-${pkgs.coreutils}/bin/chown -R root:users /var/run/openvswitch"
      "-${pkgs.coreutils}/bin/chmod -R 774 /var/run/openvswitch"
    ];
    systemd.services.ovs-vswitchd.serviceConfig.Group = "users";
    systemd.services.ovs-vswitchd.serviceConfig.ExecStartPost = [
      "-${pkgs.coreutils}/bin/chown -R root:users /var/run/openvswitch"
      "-${pkgs.coreutils}/bin/chmod -R 774 /var/run/openvswitch"
    ];

    environment.systemPackages = with pkgs; [
      # Network manager
      openvswitch
    ];
  }
