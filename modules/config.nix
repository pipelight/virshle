{
  lib,
  config,
  inputs,
  pkgs,
  ...
}:
with lib;
with pkgs; let
  moduleName = "virshle";

  cfg = config.services.${moduleName};
  user = cfg.user ? "root";
  logLevel = cfg.logLevel;

  package = inputs.virshle.packages.${system}.default;
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
  mkIf cfg.enable {
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

      serviceConfig = let
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
        User = "root";
        Group = "wheel";
        Environment = [
          # "PATH=${config.security.wrapperDir}:/run/current-system/sw/bin"
          "PATH=/run/current-system/sw/bin"
          # Set home to user.
          "HOME=${config.users.users.${user}.home}"
        ];
        ExecStartPre = [
          # "-${pkgs.coreutils}/bin/chown -R ${user}:wheel /var/lib/virshle"
          # "-${config.security.wrapperDir}/virshle node init --all ${verbosity}"
          "-${package}/virshle node init --all ${verbosity}"
        ];
        # ExecStart = "${config.security.wrapperDir}/virshle node serve ${verbosity}"
        ExecStart = "${package}/bin/virshle node serve ${verbosity}";

        WorkingDirectory = "/var/lib/virshle";
        StandardInput = "null";
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

    environment.systemPackages = [
      # Network manager
      package
      virshleProxyCommand
    ];

    programs.ssh = with lib; {
      systemd-ssh-proxy.enable = true;
      # Enable ssh-vsock communication on host side.
      extraConfig = mkAfter ''
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
