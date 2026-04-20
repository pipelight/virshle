{
  config,
  pkgs,
  lib,
  inputs,
  ...
}: {
  imports = [
    inputs.disko.nixosModules.disko
    ./disko.nix
  ];

  virtualisation.vmVariantWithDisko = {
    virtualisation = {
    };
  };

  # Disko configuration
  disko.imageBuilder = lib.mkForce {
    imageFormat = "raw";
    copyNixStore = true;
  };
}
