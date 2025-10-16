{
  config,
  pkgs,
  pkgs-unstable,
  pkgs-stable,
  lib,
  inputs,
  ...
}: let
  moduleName = "virshle";
  cfg = config.services.${moduleName};
  nsdEnabled = config.services.nsd.enable;
  nsdPort = config.services.nsd.port;

  # Service listening ports.
  keaDDnsPort = 53010;
  keaCtrlPort = 5547;
in
  with lib;
    mkIf cfg.dhcp.defaultConfig {
      ## DhcpV6
      environment.systemPackages = with pkgs; [
        # dnsmasq
        kea
      ];

      services.kea = {
        ctrl-agent = {
          settings = {
            http-port = keaCtrlPort;
            control-sockets = {
              dhcp4 = {
                socket-type = "unix";
                socket-name = "/run/kea/dhcp4.sock";
              };
              dhcp6 = {
                socket-type = "unix";
                socket-name = "/run/kea/dhcp6.sock";
              };
              d2 = {
                socket-type = "unix";
                socket-name = "/run/kea/ddns.sock";
              };
            };
          };
        };
        dhcp6 = {
          settings = {
            #Json
            control-socket = {
              socket-type = "unix";
              socket-name = "/run/kea/dhcp6.sock";
            };
            interfaces-config = {
              interfaces = mkDefault [
                "br0-dhcp"
              ];
              # service-sockets-max-retries = 5;
              # service-sockets-retry-wait-time = 5000;
              # re-detect = true;
            };
            lease-database = {
              name = "/var/lib/kea/dhcp6.leases";
              persist = true;
              type = "memfile";
              lfc-interval = 60;
            };
            rebind-timer = 1000;
            renew-timer = 600;

            preferred-lifetime = -1;
            valid-lifetime = -1;

            # Force renewal of ipv6 leases on vm up.
            # Because when setting ipv6 (not 4) lifetimes to forever,
            #
            # preferred-lifetime = 3000;
            # valid-lifetime = 4000;
            #
            #```
            # preferred-lifetime = -1;
            # valid-lifetime = -1;
            #```

            # The vm never tries to speak to dhcp again.
            # So dhcp can lose track of lease!
            expired-leases-processing = {
              reclaim-timer-wait-time = 3;
              flush-reclaimed-timer-wait-time = 5;
              hold-reclaimed-time = 2592000; #30 days
            };

            hooks-libraries = [
              {
                library = "${pkgs.kea}/lib/kea/hooks/libdhcp_lease_cmds.so";
              }
              # {
              #   library = "${pkgs.kea}/lib/kea/hooks/libdhcp_ddns_cmds.so";
              # }
            ];
            dhcp-ddns = {
              enable-updates = true;
              server-ip = "::1";
              server-port = keaDDnsPort;
            };
            ddns-update-on-renew = true;
            ddns-qualifying-suffix = "vm";
          };
        };
        dhcp4 = {
          settings = {
            control-socket = {
              socket-type = "unix";
              socket-name = "/run/kea/dhcp4.sock";
            };
            interfaces-config = {
              interfaces = mkDefault [
                "br0-dhcp"
              ];
              # service-sockets-max-retries = 5;
              # service-sockets-retry-wait-time = 5000;
              # re-detect = true;
            };
            lease-database = {
              name = "/var/lib/kea/dhcp4.leases";
              persist = true;
              type = "memfile";
              # lfc-interval = 1000;
              lfc-interval = 60;
            };
            rebind-timer = 1000;
            renew-timer = 600;
            valid-lifetime = -1;
            hooks-libraries = [
              {
                library = "${pkgs.kea}/lib/kea/hooks/libdhcp_lease_cmds.so";
              }
              # {
              #   library = "${pkgs.kea}/lib/kea/hooks/libdhcp_ddns_cmds.so";
              # }
            ];
            dhcp-ddns = {
              enable-updates = true;
              server-ip = "::1";
              server-port = keaDDnsPort;
            };
            ddns-update-on-renew = true;
            ddns-qualifying-suffix = "vm";
          };
        };

        dhcp-ddns = {
          settings = {
            control-socket = {
              socket-type = "unix";
              socket-name = "/run/kea/ddns.sock";
            };

            dns-server-timeout = 500;
            port = keaDDnsPort;
            ip-address = "::1";
            forward-ddns = {
              ddns-domains = [
                {
                  name = "vm.";
                  dns-servers = [
                    {
                      ip-address = "::1";
                      port =
                        if nsdEnabled
                        then nsdPort
                        else 53;
                    }
                  ];
                }
              ];
            };
          };
        };
      };
      ###############################
      # Sytemd unit rework

      systemd.tmpfiles.rules = [
        # "Z '/var/lib/kea' 2764 root users - -"
      ];
      systemd.services = with lib; let
        serviceConfig = {
          User = mkForce "root";
          Group = "users";
          UMask = mkForce "0007";

          # Is reloaded when network is reloaded
          # to bind the fresh interfaces.
          WantedBy = ["network.target"];
          # Starts only after interfaces creation.
          After = ["network.target"];

          # Do not store tmp files in private dir.
          DynamicUser = mkForce false;
          # Set file permissions
          ExecStartPost = [
            # "-${pkgs.coreutils}/bin/chmod -R 7660 /var/lib/kea"
            # "-${pkgs.coreutils}/bin/chmod -R g+r /var/lib/kea"
            # "-${pkgs.coreutils}/bin/chmod -R g+w /var/lib/kea"
          ];

          # StateDirectory = "kea"; # default
          LogsDirectory = "/var/log/kea";

          AmbientCapabilities = [
            "CAP_NET_BIND_SERVICE"
            "CAP_NET_RAW"
          ];
          CapabilityBoundingSet = [
            "CAP_NET_BIND_SERVICE"
            "CAP_NET_RAW"
          ];
        };
      in {
        kea-ctrl-agent.serviceConfig = serviceConfig;
        kea-dhcp6-server.serviceConfig = serviceConfig;
        kea-dhcp4-server.serviceConfig = serviceConfig;
        kea-dhcp-ddns-server.serviceConfig = serviceConfig;
      };
    }
