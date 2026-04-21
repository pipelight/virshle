{lib, ...}: {
  # Use dchp on main interface to get static ipv6 from hypervisor.
  networking = {
    networkmanager.enable = false;
    interfaces = {
      ens4 = {
        useDHCP = true;
      };
    };
  };

  # Enable ssh connection from the host via vsock.
  boot = {
    initrd.availableKernelModules = [
      "vsock"
    ];
    initrd.kernelModules = [
      "vsock"
    ];
    kernelParams = [
      "IPv6PrivacyExtensions=1"
    ];
  };

  services.openssh = {
    enable = true;
    # require public key authentication for better security
    settings = {
      PasswordAuthentication = false;
      KbdInteractiveAuthentication = false;
      # Raise the limit in case of ssh-agent
      MaxAuthTries = 12;
      Macs = [
        # rust libssh2 compat
        "hmac-sha2-256"
        "hmac-sha2-512"
        # default
        "hmac-sha2-512-etm@openssh.com"
        "hmac-sha2-256-etm@openssh.com"
        "umac-128-etm@openssh.com"
      ];
    };
  };
  programs.ssh.systemd-ssh-proxy.enable = true;

  # Enable slaac token.
  boot = {
    kernel.sysctl = with lib; {
      "net.ipv6.conf.ens4.accept_ra_from_local" = 1;

      "net.ipv6.conf.ens4.accept_ra" = 2;
      "net.ipv6.conf.ens4.use_tempaddr" = mkForce 2;
      "net.ipv6.conf.ens4.addr_gen_mode" = 3;
    };
  };

  # Default dns servers.
  networking.nameservers = with lib;
    mkDefault [
      # Ipv6 first
      # Mullvad
      "2a07:e340::4"
      # Quad9
      "2620:fe::fe"
      "2620:fe::9"

      # Ipv4 support
      # Mullvad
      "194.242.2.4"
      # Quad9
      "9.9.9.9"
    ];
}
