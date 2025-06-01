{
  lib,
  config,
  inputs,
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
      package = pkgs.openvswitch-dpdk;
      enable = true;
    };

    boot = with lib; {
      kernelModules = ["openvswitch"];
      kernelParams = mkDefault ["nr_hugepages=1024"];
      kernel.sysctl = {
        "vm.nr_hugepages" = mkDefault 1024;
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
      openvswitch-dpdk
    ];
  }
