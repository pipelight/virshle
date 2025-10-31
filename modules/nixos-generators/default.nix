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
    ./networking.nix
    ./misc.nix
    inputs.pipelight.nixosModules.pipelight-init
    {
      services.pipelight-init.enable = true;
    }
  ];
}
