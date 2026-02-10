{
  virshle_lib,
  lib,
  config,
  pkgs,
  ...
}:
with lib; let
  # When upgrading openflow version do not forget to
  # - delete ovs database entries,
  #  or
  # - manually update briges in ovs database,
  #  `ovs-vsctl set bridge br0 protocols=OpenFlow13,OpenFlow15`
  openFlowVersion = "OpenFlow15";
  supportedOpenFlowVersions = [
    "OpenFlow15"
  ];
in {
  users.users."anon".openssh.authorizedKeys.keys = [
    #v0
    "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAID5YciI0i5sqro+5UYqA+R/BhsbnDv/KJwhaFeow1zFE anon@v0"
  ];

  # Fix network restart failure.
  # Never do `systemctl restart network-setup`
  # Always do `systemctl stop network-setup && sleep 1 && systemctl start network-setup`
  # To flush file descriptors and stop then impossible start.

  ##########################
  ## Security
  # Generate random MAC to hide hardware MAC address.
  networking.localCommands =
    ''
    ''
    # "set +e" returns success even if a command fails.
    + ''
      set +e
      ${pkgs.macchanger}/bin/macchanger -r eno1
      ${pkgs.macchanger}/bin/macchanger -r enp4s0
      ${pkgs.macchanger}/bin/macchanger -r wlp3s0
    ''
    # Bring vm dhcp bridge up
    + ''
      set -e
      ${pkgs.iproute2}/bin/ip link set dev br0-dhcp up
    ''
    # Safeguard: remove potential ip from main interface.
    + ''
      ${pkgs.iproute2}/bin/ip a flush enp4s0
    ''
    # Remove unused ips
    + ''
      ${pkgs.iproute2}/bin/ip a flush vs0p1 deprecated
    '';

  # Secure network params reload.
  networking.wireless.enable = false;
  # systemd.services.network-setup.wantedBy = ["sysinit-reactivation.target"];

  ##########################
  # Dhcp - Automatic address configuration
  networking = {
    # Enable usage of mixed
    # Static local IPv6 address (ULA)
    # and global IPv6 address auto-configuration with SLAAC.
    dhcpcd.IPv6rs = true;

    defaultGateway6 = {
      address = "fe80::1";
      interface = "vs0p1";
    };
    # Enable dhcp on main interface.
    interfaces.vs0p1 = {
      useDHCP = true;
      ipv6.addresses = [
        ## Directives
        # Subnet: 2a02:842b:6361:ad00::/50
        # Router: fe80::1 and
        {
          address = "fe80::c0da";
          prefixLength = 64;
        }
        {
          address = "fd00::c0da";
          prefixLength = 64;
        }
        {
          address = "2a02:842b:6361:ad00::c0da";
          prefixLength = 64;
        }
        {
          address = "2a02:842b:6361:ad02::c0da";
          prefixLength = 64;
        }
      ];
      ipv6.routes = [
        {
          address = "2a02:842b:6361:ad00::";
          prefixLength = 50;
          via = "fe80::1";
        }
      ];
    };
    dhcpcd.extraConfig =
      ''
        ipv4only
      ''
      + ''
        denyinterfaces vm-*
        denyinterfaces en*

        interface vs0p1

        ## ipv6 params
        # ipv6only
        ipv6rs
        slaac token ::c0da temp

        ## privacy params
        # anonymous
        # randomise_hwaddr
      '';
    # Disable dhcp on every other interface.
    useDHCP = false;
    interfaces.enp4s0.useDHCP = false;
  };

  ##########################
  ## Network manager (nmcli)
  networking.networkmanager = {
    enable = false;
    unmanaged = [
      # physical
      "en*"
      # host vswitch
      "vs0"
      "vs0*"
      # vm vswitch
      "br0"
      "br0*"
      # vm tap devices managed by virshle
      "vm-*"
    ];
  };

  ##########################
  # Create openvswitch network interfaces.
  # Packet management is made outside of linux kernel.
  networking.vswitches.vs0 = {
    interfaces = {
      # Disable host default port
      enp4s0 = {};
      # Enable host ovs port
      vs0p1 = {
        type = "internal";
      };
    };
    extraOvsctlCmds = let
      mainInterface = "enp4s0";
    in
      ''
        set bridge vs0 datapath_type=system
        set bridge vs0 protocols=${openFlowVersion}
      ''
      # Add patch port
      # Create a brigde between vs0 and br0
      + ''
        add-port vs0 patch_br0
        set interface patch_br0 type=patch
        set interface patch_br0 options:peer=patch_vs0
      ''
      # Dispaly ovs switch ports id.
      # ```sh
      # sudo ovs-ofctl -O OpenFlow15 show vs0
      # ````
      + ''
        set interface ${mainInterface} ofport_request=1
        set interface vs0p1 ofport_request=2
        set interface patch_br0 ofport_request=3
      ''
      # Set interface mac address based on hostname.
      + ''
        set interface vs0p1 mac=\"${virshle_lib.str_to_mac config.networking.hostName}\"
      '';

    inherit openFlowVersion;
    inherit supportedOpenFlowVersions;
    openFlowRules = let
      crotuiPort = builtins.toString 22;
    in
      # Drop dhcp comming from the VM dedicated switch,
      # Switch br0 --x-> vs0 (patch_br0 = port 3 of this switch).
      ''
        table=0,priority=50,in_port=3,udp6,udp_src=547,action=drop
        table=0,priority=50,in_port=3,icmp6,icmp_type=134,icmp_code=0,action=drop
        table=0,priority=50,in_port=3,udp,udp_src=67,action=drop
        table=0,priority=50,in_port=3,icmp,icmp_type=9,action=drop

      ''
      # Add connection meter for ssh on port 22
      + ''
        # table=0,priority=20,in_port=2,tcp,ct_state=-trk,action=ct(table=1)
        # table=1,priority=20,in_port=2,tcp,ct_state=+trk+new,action=ct(commit)
        # table=1,priority=20,in_port=2,tcp,ct_state=+trk+new,ct_tp_dst=${crotuiPort},action=meter:1,normal
      ''
      + ''
        priority=0,action=normal
      '';
  };
  # Patch network interface creation script.
  # Add rate limiting/metered action.
  # Max: 8Mb/s -> 1MiB/s
  systemd.services.vs0-netdev.preStart = mkAfter ''
    set +e
    echo "Adding meters to Open vSwitch vs0..."
    ovs-ofctl -O ${openFlowVersion} add-meter vs0 'meter=1,kbps,stats,bands=type=drop,rate=8000'
  '';
  systemd.services.vs0-netdev.postStop = mkAfter ''
    set +e
    echo "Removing meters from Open vSwitch vs0..."
    ovs-ofctl -O ${openFlowVersion} del-meters vs0
  '';

  # Vm isolated bridge
  networking.vswitches.br0 = {
    interfaces = {
      br0-dhcp = {
        type = "internal";
      };
    };
    extraOvsctlCmds =
      ''
        set bridge br0 datapath_type=system
        set bridge br0 protocols=${openFlowVersion}
      ''
      # Add patch port
      # Create a brigde between vs0 and br0
      + ''
        add-port br0 patch_vs0
        set interface patch_vs0 type=patch
        set interface patch_vs0 options:peer=patch_br0
      ''
      # Dispaly ovs switch ports id.
      # ```sh
      # sudo ovs-ofctl -O OpenFlow15 show br0
      # ````
      + ''
        set interface patch_vs0 ofport_request=3
        set interface br0-dhcp ofport_request=4
      '';

    inherit openFlowVersion;
    inherit supportedOpenFlowVersions;
    openFlowRules = let
      parentRouter = "fe80::1";
    in
      # Switch vs0 --x-> br0 (patch_vs0 = port 3 of this switch).
      # Dhcpv4 server
      # Drop messages from other dhcp servers.
      ''
        table=0,priority=50,in_port=4,udp,udp_src=67,action=goto_table:1
        table=0,priority=10,udp,udp_src=67,action=drop

        table=1,priority=10,udp,udp_src=67,action=normal
      ''
      # Dhcpv6 server
      # Drop messages from other dhcp servers.
      + ''
        table=0,priority=50,in_port=4,udp6,udp_src=547,action=goto_table:1
        table=0,priority=10,udp6,udp_src=547,action=drop

        table=1,priority=10,udp6,udp_src=547,action=normal
      ''
      # Dhcpv6 client
      # Redirect request to VMs dhcp server.
      + ''
        # table=0,priority=50,udp6,udp_src=546,action=goto_table:1
        # table=1,priority=10,udp6,udp_src=546,action=output(port=4)
      ''
      # Router advertisements(RA) rewrite origin to parent router.
      # So outgoin request pass by parent router.
      # RA = icmp6 type 134
      + ''
        table=0,priority=50,in_port=4,icmp6,icmp_type=134,icmp_code=0,action=goto_table:1
        table=0,priority=10,icmp6,icmp_type=134,icmp_code=0,action=drop

        table=1,priority=50,icmp6,icmp_type=134,icmp_code=0,action=set_field:${parentRouter}->ipv6_src,normal

      ''
      # RA = icmp type 9
      + ''
        table=0,priority=50,in_port=4,icmp,icmp_type=9,action=goto_table:1
        table=0,priority=10,icmp,icmp_type=9,action=drop

        table=1,priority=10,icmp,icmp_type=9,action=normal
      ''
      + ''
        table=1,priority=0,action=drop
        table=0,priority=0,action=normal
      '';
  };

  ##########################
  ## Kernel parameters in case dhcpcd bug.
  ## Value definition at: https://sysctl-explorer.net
  ## Or use command line explorer `sysctl` or `systeroid-tui`
  boot = {
    kernelParams = ["IPv6PrivacyExtensions=1"];
    kernel.sysctl = {
      # Forward direct broadcast
      # 0 = disable
      # 1 = enable
      # "net.ipv6.conf.all.forwarding" = 1;
      # "net.ipv6.conf.default.forwarding" = 1;
      # "net.ipv4.conf.all.forwarding" = 1;
      # "net.ipv4.conf.default.forwarding" = 1;

      # Privacy Extensions
      # 0 = disable
      # 1 = enable
      # 2 = enable + prefer temp addresses over public
      "net.ipv6.conf.default.use_tempaddr" = mkForce 2;
      "net.ipv6.conf.all.use_tempaddr" = 2;

      # Generate random ipv6
      # 0 = "eui64"
      # 1 = "eui64"
      # 2 = "stable-privacy" with secret
      # 3 = "stable-privacy" with random secret
      "net.ipv6.conf.default.addr_gen_mode" = mkForce 3;
      "net.ipv6.conf.all.addr_gen_mode" = mkForce 3;

      # Accept router advertisements
      # 0 = do not accept
      # 1 = accept
      # 2 = accept even if forwarding enabled
      # Enable slaac token for main interface only
      "net.ipv6.conf.all.accept_ra" = 0;
      "net.ipv6.conf.default.accept_ra" = 0;

      # Disable slaac token for other interfaces (bridges...)
      # Removing ip from bridges and glue interfaces
      # prevents packet from successfully reaching main interface.
      # Ovs interfaces
      "net.ipv6.conf.ovs-system.accept_ra" = 0;
      # Host switch
      "net.ipv6.conf.enp4s0.accept_ra" = 0;
      "net.ipv6.conf.eno1.accept_ra" = 0;
      "net.ipv6.conf.vs0.accept_ra" = 0;
      # Vm switch
      "net.ipv6.conf.br0.accept_ra" = 0;

      # On VM dhcp
      "net.ipv6.conf.br0-dhcp.accept_ra" = 0;

      # Host main port
      "net.ipv6.conf.vs0p1.accept_ra" = mkForce 2;
      "net.ipv6.conf.vs0p1.use_tempaddr" = mkForce 2;
      "net.ipv6.conf.vs0p1.addr_gen_mode" = mkForce 3;
    };
  };

  ##########################
  # Firewall
  networking.firewall = {
    enable = true;
    allowedTCPPorts = [
      # ssh
      22
      # web
      80
    ];
    allowedUDPPorts = [
      # dhcp client
      546
      # dhcp server
      547
    ];
  };

  ##########################
  # DHCPv6 extra

  services.kea.ctrl-agent = {
    enable = true;
  };
  services.kea.dhcp-ddns = {
    enable = true;
  };

  ##########################
  # DHCPv6
  networking.interfaces.br0-dhcp.ipv6 = {
    addresses = [
      {
        address = "fd00:a1::647:1";
        prefixLength = 64;
      }
      {
        address = "fe80::647:1";
        prefixLength = 64;
      }
    ];
  };
  services.kea.dhcp6 = {
    enable = true;
    settings = {
      interfaces-config.interfaces = [
        "br0-dhcp"
      ];
      subnet6 = [
        ##########################
        # Inter Vm connection
        {
          id = 10;
          interface = "br0-dhcp";
          subnet = "fd00:a1::/64";
          allocator = "random";
          pools = [
            {
              pool = "fd00:a1::ff - fd00:a1::ffff:ffff:ffff:ffff";
            }
          ];
        }
        ##########################
        # Vm dedicated subnet
        {
          id = 2;
          interface = "br0-dhcp";
          subnet = "2a02:842b:6361:ad02::/64";
          allocator = "random";
          pools = [
            {
              pool = "2a02:842b:6361:ad02::ff - 2a02:842b:6361:ad02:ffff:ffff:ffff:ffff";
            }
          ];
        }
      ];
    };
  };
  ##########################
  # RA- Router announcements
  services.radvd = {
    enable = true;
    debugLevel = 4;
    config = ''

      interface vs0p1 {
        IgnoreIfMissing on;
      };

      interface br0-dhcp {

        AdvSendAdvert on;

        AdvManagedFlag on;
        AdvOtherConfigFlag on;

        MinRtrAdvInterval 200;
        MaxRtrAdvInterval 600;

        AdvDefaultLifetime 1800;

        ##########################
        ## Parent router

        prefix fd00:a1::/64 {
          AdvOnLink on;
          AdvAutonomous on;
          AdvValidLifetime 4000;
          AdvPreferredLifetime 3000;
        };

        # Static address
        prefix 2a02:842b:6361:ad02::/64 {
          AdvOnLink on;
          AdvAutonomous off;
          AdvValidLifetime 4000;
          AdvPreferredLifetime 3000;
        };

        # Privacy addresses
        prefix 2a02:842b:6361:ad10::/64 {
          AdvOnLink on;
          AdvAutonomous on;
          AdvValidLifetime 4000;
          AdvPreferredLifetime 3000;
        };

        route 2a02:842b:6361:ad00::/56 {
          AdvRouteLifetime 3000;
        };

        route fd00:a1::/64 {
          AdvRouteLifetime 3000;
        };

      };
    '';
  };
}
