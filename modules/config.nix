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
      "Z '/var/lib/virshle' 774 root users - -"
      "d '/var/lib/virshle' 774 root users - -"
    ];

    systemd.services.virshle = {
      enable = true;
      description = "Virshle node daemon (level 2 hypervisor)";
      documentation = [
        "https://github.com/pipelight/virshle"
      ];
      after = [
        "network.target"
        "socket.target"
        "ovs-vswitchd.service"
        "ovsdb.service"
      ];
      wantedBy = ["multi-user.target"];

      serviceConfig = with pkgs; let
        package = inputs.virshle.packages.${system}.default;
      in {
        Type = "simple";
        User = "root";
        Group = "users";
        Environment = "PATH=/run/current-system/sw/bin";
        # ExecStartPre = [
        #   "-${package}/bin/virshle init -vvv"
        # ];
        ExecStart = ''
          ${package}/bin/virshle node serve -vvv
        '';
        WorkingDirectory = "/var/lib/virshle";
        # StandardInput = "null";
        StandardOutput = "journal+console";
        StandardError = "journal+console";

        AmbientCapabilities = [
          # "CAP_NET_BIND_SERVICE"
          # "CAP_SET_PROC"
          # "CAP_SETUID"
          # "CAP_SETGID"
          "CAP_SYS_ADMIN"
          "CAP_NET_ADMIN"
        ];
      };
    };

    environment.systemPackages = with pkgs; [
      # Network manager
      inputs.virshle.packages.${system}.default
      openvswitch-dpdk
    ];
  }
