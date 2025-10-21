{
  inputs,
  config,
  lib,
  pkgs,
  modulesPath,
  ...
}: {
  imports = [
    ./networking.nix
    ./hardware-configuration.nix
    inputs.pipelight.nixosModules.pipelight-init
    {
      services.pipelight-init.enable = true;
    }
  ];
}
