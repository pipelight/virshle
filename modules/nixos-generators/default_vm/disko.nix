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
  disko.devices = {
    disk = {
      "nixos.efi" = {
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
                  "root" = {
                    mountpoint = "/";
                    mountOptions = ["subvol=root" "compress=zstd"];
                  };
                  "home" = {
                    mountpoint = "/home";
                    mountOptions = ["subvol=home" "compress=zstd"];
                  };
                  "nix" = {
                    mountpoint = "/nix";
                    mountOptions = ["subvol=nix" "compress=zstd" "noatime"];
                  };
                  "persist" = {
                    mountpoint = "/persist";
                    mountOptions = ["subvol=persist" "compress=zstd" "noatime"];
                  };
                  "log" = {
                    mountpoint = "/var/log";
                    mountOptions = ["subvol=log" "compress=zstd" "noatime"];
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
