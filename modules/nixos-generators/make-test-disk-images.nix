{
  inputs,
  config,
  lib,
  pkgs,
  modulesPath,
  ...
}: {
  # See nixos-generators/formats/raw.efi.nix
  # and nixos-generators/formats/raw.nix
  #
  fileSystems."/boot" = {
    device = "/dev/disk/by-label/ESP";
    fsType = "vfat";
  };
  # Need to specify root fs for `nixos-rebuild`
  #
  fileSystems."/" = lib.mkDefault {
    # device = "/dev/disk/by-label/nixos";
    device = "/dev/disk/by-label/ROOT";
    fsType = "ext4";
    # fsType = "btrfs"; # does not exist
    autoResize = true;
  };
  ## Override nixos-generators config.
  # Add a postVM script to build multible VMs of sizes:
  # - xxs, 20 GiB
  system.build.raw = lib.mkForce (import "${toString modulesPath}/../lib/make-disk-image.nix" {
    inherit lib config pkgs;
    inherit (config.virtualisation) diskSize;

    partitionTableType = "efi";
    format = "raw";
    label = "ROOT";

    postVM = let
      # Copy the image and resize to the given size (in GiB)
      make_disk = name: size:
      # Copy base image
        ''
          cp $out/nixos.img $out/nixos.${name}.efi.img
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
  });
}
