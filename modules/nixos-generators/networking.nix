{...}: {
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
  programs.ssh.systemd-ssh-proxy.enable = true;

  # Enable slaac token.
  boot = {
    kernel.sysctl."net.ipv6.conf.ens4.accept_ra" = 2;
    kernel.sysctl."net.ipv6.conf.ens4.accept_ra_from_local" = 1;
  };
}
