{inputs, ...}: {
  imports = [
    # module configuration options
    ./options.nix

    # virshle
    ./config.nix

    ##########################
    ### Virshle dependencies

    ## Virtual machines management
    ./cloud-hypervisor.nix

    ## Network management
    ./openvswitch
    # DHCP - Automatic ip address attribution
    ./dhcp/kea.nix
  ];
}
