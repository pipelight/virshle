{
  config,
  pkgs,
  lib,
  inputs,
  ...
}: {
  imports = [
    inputs.disko.nixosModules.disko
  ];

  systemd.tmpfiles.rules = [
    "d /persist 740 root users - -"
    "d /var/log 740 root root - -"
  ];

  disko.memSize = 6 * 1024;
  disko.imageBuilder = lib.mkForce {
    imageFormat = "raw";
    copyNixStore = true;
  };

  disko.devices = {
    disk = {
      "main" = {
        # Set default imageSize
        # imageSize = "20G";
        type = "disk";
        content = {
          type = "gpt";
          partitions = {
            ESP = {
              priority = 1;
              # name = "ESP";
              type = "EF00";
              size = "250M";
              content = {
                type = "filesystem";
                format = "vfat";
                mountpoint = "/boot";
                mountOptions = ["umask=0077"];
                extraArgs = ["-nESP"];
              };
            };
            ROOT = {
              size = "100%";
              content = {
                type = "btrfs";
                extraArgs = ["-f" "-LROOT"];
                subvolumes = {
                  # Disable zstd compression for CPU friendly I/O.
                  "root" = {
                    mountpoint = "/";
                    mountOptions = ["subvol=root"];
                  };
                  "home" = {
                    mountpoint = "/home";
                    mountOptions = ["subvol=home"];
                  };
                  "nix" = {
                    mountpoint = "/nix";
                    mountOptions = ["subvol=nix" "noatime"];
                  };
                  "persist" = {
                    mountpoint = "/persist";
                    mountOptions = ["subvol=persist" "noatime"];
                  };
                  "log" = {
                    mountpoint = "/var/log";
                    mountOptions = ["subvol=log" "noatime"];
                  };
                };
              };
            };
          };
        };
      };
    };
  };
}
