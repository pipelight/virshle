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
    ## Module
    systemd.tmpfiles.rules = [
      "d '/var/lib/virshle' 774 root users - -"
      # Loosen permissions on openvswitch.
      "d '/var/run/openvswitch' 774 root users - -"
    ];

    systemd.services.virshle = {
      enable = true;
      description = "Virshle node daemon (level 2 hypervisor)";
      documentation = [
        "https://github.com/pipelight/virshle"
        "virshle --help"
      ];
      after = ["network.target" "socket.target"];
      wantedBy = ["multi-user.target"];

      serviceConfig = with pkgs; let
        package = inputs.virshle.packages.${system}.default;
      in {
        Type = "simple";
        User = "root";
        Group = "users";
        Environment = "PATH=/run/current-system/sw/bin";
        ExecStart = ''
          ${package}/bin/virshle daemon -vvv
        '';
        WorkingDirectory = "/var/lib/virshle";
        # StandardInput = "null";
        StandardOutput = "journal+console";
        StandardError = "journal+console";

        AmbientCapabilities = [
          # "CAP_NET_BIND_SERVICE"
          # "CAP_NET_ADMIN"
          "CAP_SYS_ADMIN"
        ];
      };
    };

    boot = with lib; {
      kernelModules = ["openvswitch"];
      kernelParams = mkDefault ["nr_hugepages=1024"];
      kernel.sysctl = {
        "vm.nr_hugepages" = mkDefault 1024;
      };
    };
    # OpenVSwitch
    virtualisation.vswitch = {
      package = pkgs.openvswitch-dpdk;
      enable = true;
    };

    environment.systemPackages = with pkgs; [
      # Network manager
      inputs.virshle.packages.${system}.default
      openvswitch-dpdk
    ];
  }
