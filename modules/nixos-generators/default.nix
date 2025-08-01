{
  config,
  lib,
  pkgs,
  inputs,
  ...
}: {
  imports = [
    ./networking.nix.nix
    ./hardware-configuration.nix
  ];
}
