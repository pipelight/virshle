{inputs, ...}: {
  imports = [
    # module configuration options
    ./options.nix

    # virshle
    ./config.nix

    # virshle deps
    ./openvswitch

    ./cloud-hypervisor.nix
    inputs.pipelight.nixosModules.pipelight-init
  ];
}
