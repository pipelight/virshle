{
  inputs,
  config,
  lib,
  pkgs,
  modulesPath,
  ...
}: {
  imports = [
    ./hardware-configuration.nix
    ./disko.nix
    ./networking.nix
    ./misc.nix
    ./nix.nix
  ];
}
