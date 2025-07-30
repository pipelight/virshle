{...}: {
  imports = [
    # module configuration options
    ./options.nix

    # virshle
    ./config.nix

    # virshle deps
    ./openvswitch.nix
    ./cloud-hypervisor.nix
    # ssh host to guest communication via vsock
    ./ssh-vsock.nix
  ];
}
