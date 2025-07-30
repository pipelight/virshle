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
        Host vm/*
          ProxyCommand ${pkgs.systemd}/lib/systemd/systemd-ssh-proxy %h %p
          ProxyUseFdpass yes
          CheckHostIP no

          # Disable all kinds of host identity checks, since these addresses are generally ephemeral.
          StrictHostKeyChecking no
          UserKnownHostsFile /dev/null
      '';
  };
}
