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
  virtualisation.vmVariantWithDisko = {
    virtualisation = {
    };
  };

  # Disko configuration
  disko.imageBuilder = lib.mkForce {
    imageFormat = "raw";
    copyNixStore = true;
    # Add a postVM script to build multible VMs of sizes:
    # - xxs, 20 GiB
    extraPostVM = let
      # Copy the image and resize to the given size (in GiB)
      make_disk = name: size:
      # Copy base image
        ''
          cp $out/nixos.efi.raw $out/nixos.${name}.efi.img
        ''
        # Extend disk file
        + ''
          dd \
          if=/dev/null \
          of=$out/nixos.${name}.efi.img \
          count=0 bs=1G seek=${builtins.toString size}
        ''
        # Extend main partition
        + ''
          echo -e ",+" | sfdisk $out/nixos.${name}.efi.img -N 2
        '';
    in ''
      echo "starting postVM script..."
      ${pkgs.coreutils}/bin/ls -alh $out
      ${make_disk "test.xxs" 20}
      ${pkgs.coreutils}/bin/ls -alh $out
      echo "end postVM script."
    '';
  };
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
