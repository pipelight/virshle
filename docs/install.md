# Install on NixOs (with flakes).

When using nixos, you can enable the module by adding those lines to your configuration.
Add the repo url to your configuration.

```nix
# flake.nix
inputs = {
  virshle = {
      url = "github:pipelight/virshle";
  };
};
```

Enable the service.

```nix
# default.nix
services.virshle = {
  enable = true;
  logLevel = "info";
  # The user to run the node as.
  user = "anon"; #default to root.
};
```

You may want to create an alias to ease command line usage.

```sh
alias v='virshle'
```

> [!WARNING]
> You'll face a small compilation time due to virshle and pipelight
> not being precompiled in any official repository.

## Custom storage.

You can store VMs resources in another device like an encrypted RAID.
Just symlink `/var/lib/virshle` to the desired path, and set required permissions.

```nix
systemd.tmpfiles.rules = [
  "L+ /var/lib/virshle - - - - /run/media/RAID/storage/virshle"
  "Z '/run/media/RAID/storage/virshle' 2774 ${config.services.virshle.user} users - -"
];
```

## Custom network configuration.

For fine vm network control, you can add a host network configuration like the following
[`modules/networking.nix`](https://github.com/pipelight/virshle/modules/config.nix).

# Install on FHS Linux distributions (Arch/Debian).

Install the binary from source with cargo.

```sh
cargo install --git https://github.com/pipelight/virshle
```

Then create a default systemd unit like the following:
[virshle.service](https://github.com/pipelight/virshle/scripts/virshle.service)
to start a virshle node in the background on server boot.

## Dependencies

Mandatory dependencies:

- [cloud-hypervisor](https://github.com/cloud-hypervisor/cloud-hypervisor)
- [openvswitch](https://github.com/openvswitch/ovs)

Optional dependencies:

- [KeaDHCP](https://kea.readthedocs.io/en/latest/)

### Virtualization: cloud-hypervisor

You first need to install ch(cloud-hypervisor), the level 1 hypervisor.
It is a software that will run the vm as a process.

```sh
# Download binary
wget https://github.com/cloud-hypervisor/cloud-hypervisor/releases/latest/download/cloud-hypervisor
sudo chmod +x cloud-hypervisor

# Tips:
# Add capability to manipulate host network,
# to run the node as a non root user.
# !Do not work for now.!
# sudo setcap cap_net_admin+ep ./cloud-hypervisor

# Move to folder in PATH
sudo mv cloud-hypervisor /usr/local/bin/

```

Copy or symlink the firmware files for direct kernel boot.
Must be available at
`/run/cloud-hypervisor/hypervisor-fw`
or
`/run/cloud-hypervisor/CLOUDVH.fd`

```sh
wget https://github.com/cloud-hypervisor/cloud-hypervisor/releases/latest/download/hypervisor-fw
sudo mv hypervisor-fw /var/run/openvswitch/hypervisor-fw
```

See the [installation instructions](https://github.com/cloud-hypervisor/cloud-hypervisor).

### Network: openvswitch

Install openvswitch,

```sh
sudo apt-get update
sudo apt-get install openvswitch-switch

```

```sh
# Tips:
# Open database permission so that a non root user
# can manipulate the host network.
# !Do not work for now.!
chown root:users /var/run/openvswitch
chmod -R 774 /var/run/openvswitch`
```

Or see the openvswitch [installation instructions](https://docs.openvswitch.org/en/latest/intro/install)
