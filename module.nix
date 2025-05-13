{
  lib,
  config,
  inputs,
  pkgs,
  ...
}: {
  systemd.tmpfiles.rules = [
    "d '/var/lib/virshle' 774 root users - -"
  ];
  imports = with pkgs; [
    inputs.virshle.packages.${system}.default
  ];
  boot = with lib; {
    kernelModules = ["openvswitch"];
    kernelParams = mkDefault ["nr_hugepages=1024"];
    kernel.sysctl = {
      "vm.nr_hugepages" = mkDefault 1024;
    };
  };
  virtualisation.vswitch = {
    package = pkgs.openvswitch-dpdk;
    enable = true;
  };
  environment.systemPackages = with pkgs; [
    # Network manager
    openvswitch-dpdk
  ];
}
