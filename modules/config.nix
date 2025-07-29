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

  user = cfg.user;
  logLevel = cfg.logLevel;
  # security = pkgs.callPackage (pkgs.modulesPath + "/security/wrappers/default.nix");
in
  mkIf cfg.enable {
    ## Ensure /var/lib/virshle exists.
    # systemd.tmpfiles.rules = mkDefault [
    #   "Z '/var/lib/virshle' 774 root users - -"
    #   "d '/var/lib/virshle' 774 root users - -"
    # ];

    ## Mount iso
    users.groups.disk.members = [user];
    security.wrappers.umount = with pkgs; {
      setuid = true;
      setgid = true;
      owner = mkForce "root";
      group = mkForce "wheel";
      permissions = "u+rx,g+rx";
    };
    security.wrappers.mount = with pkgs; {
      setuid = true;
      setgid = true;
      owner = mkForce "root";
      group = mkForce "wheel";
      permissions = mkForce "u+rx,g+rx";
    };

    security.wrappers.virshle = with pkgs; let
      package = inputs.virshle.packages.${system}.default;
    in {
      source = "${package}/bin/virshle";
      capabilities = "cap_net_admin,cap_sys_admin+ep";
      owner = user;
      group = "wheel";
      permissions = "u+rx,g+rx,o+rx";
    };

    ## Systemd unit file
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
        verbosity =
          {
            "error" = "";
            "warn" = "-v";
            "info" = "-vv";
            "debug" = "-vvv";
            "trace" = "-vvvv";
          }.${
            logLevel
          };
      in {
        Type = "simple";
        User = user;
        Group = "wheel";
        Environment = "PATH=${config.security.wrapperDir}:/run/current-system/sw/bin";
        ExecStartPre = [
          "-${pkgs.coreutils}/bin/chown -R ${user}:wheel /var/lib/virshle"
          "-${config.security.wrapperDir}/virshle node init --all ${verbosity}"
        ];
        ExecStart = ''
          ${config.security.wrapperDir}/virshle node serve ${verbosity}
        '';

        # WorkingDirectory = "/var/lib/virshle";
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
    ];
  }
