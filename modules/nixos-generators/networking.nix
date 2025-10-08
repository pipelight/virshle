{
  lib,
  pkgs,
  ...
}:
with lib; {
  networking = {
    networkmanager.enable = false;
    interfaces = {
      ens4 = {
        useDHCP = true;
      };
    };
  };
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
  # Enable slaac token.
  kernel.sysctl."net.ipv6.conf.ens4.accept_ra" = 2;
  kernel.sysctl."net.ipv6.conf.ens4.accept_ra_from_local" = 1;
}
