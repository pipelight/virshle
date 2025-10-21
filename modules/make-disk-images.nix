{
  inputs,
  config,
  lib,
  pkgs,
  modulesPath,
  ...
}: {
  ## Override nixos-generators config.
  # Add a postVM script to build multible VMs of sizes:
  # - xxs, 20 GiB
  # - xs, 50 GiB
  # - and s, 80 GiB
  system.build.raw = lib.mkForce (import "${toString modulesPath}/../lib/make-disk-image.nix" {
    inherit lib config pkgs;
    partitionTableType = "efi";
    inherit (config.virtualisation) diskSize;
    format = "raw";
    postVM = let
      # Make copy of file with size in name.
      copy_to = size: {
        cmd = "cp nixos.img nixos.${size}.efi.img";
      };
      # Resize the image to the given size (in GiB)
      to_size = size: {
        cmd = ''
          dd \
          if=/dev/null \
          of=./nixos.img \
          count=0 bs=1G seek=${size}
        '';
      };
    in ''
      echo "starting postVM script..."
      ${pkgs.coreutils}/bin/ls -alh $out
      echo "end postVM script."
    '';
  });
}
