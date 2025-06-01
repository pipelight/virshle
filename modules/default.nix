{...}: {
  imports = [
    # module confguration options
    ./options.nix

    # virshle
    ./config.nix

    # virshle deps
    ./openvswitch.nix
    ./cloud-hypervisor.nix
  ];
}
