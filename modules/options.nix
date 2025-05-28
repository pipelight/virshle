{
  lib,
  config,
  inputs,
  pkgs,
  ...
}:
with lib; let
  moduleName = "virshle";
in {
  ## Options
  options.services.${moduleName} = {
    enable = mkEnableOption "Enable ${moduleName}.";
    manageNetwork.enable = mkEnableOption "Configure host network to give VM network access";
  };
}
