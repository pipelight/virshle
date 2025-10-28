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

  package = inputs.virshle.packages.${pkgs.system}.default;

  virshleProxyCommand = pkgs.writeShellScriptBin "virshleProxyCommand" ''
    h=$1
    p=$2
    fn() {
      vm_name=$(${pkgs.coreutils}/bin/echo $h | ${pkgs.gnused}/bin/sed -e "s/^vm\///");
      vsock_path=$(${package}/bin/virshle vm get-vsock-path --name $vm_name);
      ${pkgs.systemd}/lib/systemd/systemd-ssh-proxy vsock-mux$vsock_path $p
    }
    fn
  '';
in
  mkIf cfg.enable
  {
    ## Working dir
    # systemd.tmpfiles.rules = lib.mkDefault [
    #   "Z '/var/lib/virshle' 2774 ${cfg.user} users - -"
    #   "d '/var/lib/virshle' 2774 ${cfg.user} users - -"
    #   "Z '/var/lib/virshle/cache' 2774 ${cfg.user} users - -"
    #   "d '/var/lib/virshle/cache' 2774 ${cfg.user} users - -"
    # ];

    security.wrappers.virshle = {
      source = "${package}/bin/virshle";
      owner = "root";
      group = "wheel";

      # setuid = true;
      # setgid = true;
      capabilities = "cap_net_admin,cap_sys_admin+eip";
      permissions = "u+rx,g+rx,o+rx";
    };

    ## Systemd unit file
    systemd.services.virshle = {
      enable = true;
      description = "Virshle node daemon (type 2 hypervisor)";
      documentation = [
        "https://github.com/pipelight/virshle"
      ];
      after = [
        "network.target"
        "socket.target"
        "ovs-vswitchd.service"
        "ovsdb.service"
        # Dhcp
        "kea-ctrl-agent.service"
        "kea-dhcpv4-server.service"
        "kea-dhcpv6-server.service"
      ];
      wantedBy = ["multi-user.target"];

      serviceConfig = let
        verbosity =
          {
            "error" = "";
            "warn" = "-v";
            "info" = "-vv";
            "debug" = "-vvv";
            "trace" = "-vvvv";
          }.${
            cfg.logLevel
          };
      in {
        Type = "simple";
        User = "root";
        Group = "wheel";
        Environment = [
          # "PATH=${config.security.wrapperDir}:/run/current-system/sw/bin"
          "PATH=/run/current-system/sw/bin"
          # If you want "~" to expend as another user's home.
          "HOME=${config.users.users.${cfg.user}.home}"
        ];
        ExecStartPre = [
          "-${package}/bin/virshle node init --all ${verbosity}"
        ];
        ExecStart = "${package}/bin/virshle node serve ${verbosity}";

        WorkingDirectory = "/var/lib/virshle";

        StandardInput = "null";
        StandardOutput = "journal+console";
        StandardError = "journal+console";

        # Ensure orphans are not killed
        KillMode = "process";

        AmbientCapabilities = [
          "CAP_SYS_ADMIN"
          "CAP_NET_ADMIN"

          "CAP_NET_RAW"
          "CAP_NET_BIND_SERVICE"
        ];
      };
    };

    environment.systemPackages = [
      # Network manager
      package
      virshleProxyCommand
    ];

    programs.ssh = {
      # Enable ssh-vsock communication on host side.
      extraConfig = lib.mkAfter ''
        ## Virshle special command
        Host vm/*
          ProxyCommand ${virshleProxyCommand}/bin/virshleProxyCommand %h %p
          ProxyUseFdpass yes
          CheckHostIP no
          # Disable all kinds of host identity checks, since these addresses are generally ephemeral.
          StrictHostKeyChecking no
          UserKnownHostsFile /dev/null
      '';
    };
  }
