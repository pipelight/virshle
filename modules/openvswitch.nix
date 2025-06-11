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
      package = mkIf cfg.dpdk.enable pkgs.openvswitch-dpdk;
      enable = true;
    };

    boot = let
      # Power base 10
      pow = n: i:
        if i == 1
        then n
        else if i == 0
        then 1
        else n * pow n (i - 1);

      # Set dedicated RAM in GB (ex: 16),
      # and hhugepage size in kb (default 2048)
      ram_to_hugepage = dedicated_ram: hugepage_size: toString ((dedicated_ram * pow 1024 2) / hugepage_size);
    in {
      kernelModules = ["openvswitch"];
      kernelParams = mkIf cfg.dpdk.enable (mkBefore ["nr_hugepages=${ram_to_hugepage 16 2048}"]);
      kernel.sysctl = {
        "vm.nr_hugepages" = mkIf cfg.dpdk.enable (mkBefore 4096);
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
      (mkIf
        cfg.dpdk.enable
        openvswitch-dpdk)

      (mkIf
        (!cfg.dpdk.enable)
        openvswitch)
    ];
  }
