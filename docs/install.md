## Install

### NixOs (with flakes).

When using nixos,
you can enable the module by
adding those lines to your configuration.

```nix
# flake.nix
# Add the repo url to your configuration.
inputs = {
  virshle = {
      url = "github:pipelight/virshle";
  };
};
```

```nix
# default.nix
# Enable the service.
services.virshle = {
    enable = true;
    # The user to run the node as.
    user = "anon";
};
```

You have a fresh node running without any further configuration needed.

### Other Linux distributions.

Mandatory dependencies:

- [cloud-hypervisor](https://github.com/cloud-hypervisor/cloud-hypervisor)
- [openvswitch](https://github.com/openvswitch/ovs)

You first need to install ch(cloud-hypervisor), the level 1 hypervisor.
It is a software that will run the vm as a process.

Copy or symlink the required ch files at:
`/run/cloud-hypervisor/hypervisor-fw` and
`/run/cloud-hypervisor/CLOUDVH.fd`

## Homelab, hosting and cloud providers.

Once a node is configured, you may want to have refined control over
network.

Which ip is attributed to which vm? (dhcp).
Can vm communicate between each other? and with the host? (firewall -> openflow)

Be sure to use `tap` devices for vm network.

```toml
# /etc/virshle/config.toml
[[template.vm.net]]
name = "main"
[template.vm.net.type.tap]
```

An example of network configuration can be found at.
[`modules/networking.nix`](https://github.com/pipelight/virshle/modules/config.nix).
