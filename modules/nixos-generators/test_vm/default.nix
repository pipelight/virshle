{
  inputs,
  config,
  lib,
  pkgs,
  modulesPath,
  ...
}: {
  imports = [
    ../default_vm
    ./misc.nix
  ];
}
