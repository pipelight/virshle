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
    # - xs, 50 GiB
    # - and s, 80 GiB
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
      ${make_disk "xxs" 20}
      ${make_disk "xs" 50}
      ${make_disk "s" 80}
      ${pkgs.coreutils}/bin/ls -alh $out
      echo "end postVM script."
    '';
  };
}
