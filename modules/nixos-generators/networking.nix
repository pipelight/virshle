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
}
