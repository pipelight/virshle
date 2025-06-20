{
  lib,
  config,
  inputs,
  pkgs,
  ...
}:
with lib; {
  ##########################
  ## Network default
  networking = {
    networkmanager = {
      enable = true;
      unmanaged = [
        # phisical
        "eno1"
        # vswitches
        "vs0"
        "vs0p1"
        "br0"
        "vm-dhcp"
      ];
    };

    # Here we split the host single ethernet port into 2 virtual switches (vs0 for host and br0 for VMs)
    # that can handle some thousand ports each.

    # See: /virshle_core/src/network/README.md
    # for a better understanding of how virshle splits host network.

    # We seek to prevent adding ip address to switch themselves as it breaks
    # subport connectivity.
    # So we prevent dhcp(ipv4/6) and ra(ipv6) at OS and kernel level
    # on the required interfaces: eno1, vs0, br0

    # Disable dhcpcd auto conf on phisical interface eno1
    # And host virtulal port.
    useDHCP = false;
    interfaces.eno1 = {
      useDHCP = false;
    };
    interfaces.vs0p1 = {
      useDHCP = true;
    };

    vswitches = {
      # Host bridge
      vs0 = {
        interfaces = {
          eno1 = {};
          vs0p1 = {
            type = "internal";
          };
        };
        # Add configuration to host virturl switch vs0
        # and add a weak stable mac for dhcpcd.
        extraOvsctlCmds = ''
          set bridge vs0 datapath_type=system
          set interface vs0p1 mac=\"${slib.str_to_mac config.networking.hostName}\"
        '';
      };
      # Vm isolated bridge
      br0 = {
        interfaces = {
          vm-dhcp = {
            type = "internal";
          };
        };
        extraOvsctlCmds = ''
          set bridge br0 datapath_type=system
        '';
      };
    };
    dhcpcd.extraConfig = ''
      interface vs0p1
      slaac token ::c0:ffe temp
    '';
  };

  ## Kernel parameters in case dhcpcd bug.
  boot.kernel.sysctl = {
    # "net.ipv6.conf.eno1.accept_ra" = 0;

    "net.ipv6.conf.vs0.accept_ra" = 0;
    "net.ipv6.conf.br0.accept_ra" = 0;
    "net.ipv6.conf.ovs-system.accept_ra" = 0;

    "net.ipv6.conf.vs0p1.accept_ra" = 1;
  };
}
