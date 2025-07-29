{
  lib,
  config,
  inputs,
  pkgs,
  ...
}:
with lib; {
  ##########################
  ## Ssh
  programs.ssh = {
    # Enable ssh-vsock communication on host side.
    systemd-ssh-proxy.enable = true;
    extraConfig =
      # Systemd temporary patch
      # Using the latest systemd-ssh config file while waiting for it
      # to reach the upstream stable nix release.
      # https://github.com/systemd/systemd/blob/main/src/ssh-generator/20-systemd-ssh-proxy.conf.in
      ''
        # SPDX-License-Identifier: LGPL-2.1-or-later

        # Allow connecting to the local host directly via ".host"
        Host .host machine/.host
          ProxyCommand ${pkgs.systemd}/lib/systemd/systemd-ssh-proxy unix/run/ssh-unix-local/socket %p
          ProxyUseFdpass yes
          CheckHostIP no

        # Make sure unix/* and vsock/* can be used to connect to AF_UNIX and AF_VSOCK paths.
        # Make sure machine/* can be used to connect to local machines registered in machined.

        Host unix/* unix%* vsock/* vsock%* vsock-mux/* vsock-mux%* machine/* machine%*
          ProxyCommand ${pkgs.systemd}/lib/systemd/systemd-ssh-proxy %h %p
          ProxyUseFdpass yes
          CheckHostIP no

          # Disable all kinds of host identity checks, since these addresses are generally ephemeral.
          StrictHostKeyChecking no
          UserKnownHostsFile /dev/null
      ''
      ## Plus virshle special command
      + ''
        Host *.vsock
          ProxyCommand ${pkgs.systemd}/lib/systemd/systemd-ssh-proxy %h %p
          ProxyUseFdpass yes
          CheckHostIP no

          # Disable all kinds of host identity checks, since these addresses are generally ephemeral.
          StrictHostKeyChecking no
          UserKnownHostsFile /dev/null
      '';
  };

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
          ## For use with DoraDhcp or KeaDhcp.
          ##
          ## This interface is for when you need to give ip addresses to your VM
          ## from a local dhcpcd server (on the same machine).
          ##
          ## But Most of the time, your router takes care of this.
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
