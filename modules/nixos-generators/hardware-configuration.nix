{
  inputs,
  lib,
  pkgs,
  modulesPath,
  ...
}: {
  # for virtio kernel drivers
  imports = [
    (modulesPath + "/profiles/qemu-guest.nix")
    inputs.pipelight.nixosModules.pipelight-init
  ];

  ###################################
  ## Console output
  # Expose guest VM tty to host through virsh
  boot.kernelParams = lib.mkForce [
    "console=hvc0"
    "console=ttyS0,115200"
  ];
  systemd.services."serial-getty@hvc0".enable = true;
  systemd.services."serial-getty@ttyS0".enable = true;

  boot = {
    # Do not try to resize partition on boot.
    growPartition = lib.mkForce false;
    tmp.cleanOnBoot = true;
    # tmpOnTmpfs = false;
    # tmp.tmpfsHugeMemoryPages = "within_size";

    kernelPackages = pkgs.linuxPackages; #lts
    loader = {
      efi = {
        canTouchEfiVariables = true;
      };
      # Systemd boot
      systemd-boot = {
        enable = true;
        graceful = true;
      };
    };
  };

  # Provisioning volume
  systemd.tmpfiles.rules = [
    "Z '/pipelight-init' 774 root users - -"
  ];

  services.pipelight-init.enable = true;

  fileSystems."/pipelight-init" = {
    device = "/dev/disk/by-label/INIT";
    fsType = "vfat";
    options = [
      "nofail"
    ];
  };

  # Need to specify root fs for `nixos-rebuild`
  fileSystems."/" = lib.mkDefault {
    # device = "/dev/disk/by-label/nixos";
    device = "/dev/disk/by-label/ROOT";
    fsType = "ext4";
    autoResize = true;
  };
  services.dbus.implementation = "broker";
}
