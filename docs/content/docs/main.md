+++
date = 2025-09-11
updated = 2026-02-10

weight = 10

title = "Virshle documentation 📖"

description = """

"""

draft=false
+++

# Introduction.

Virshle is a virtual machine manager.
It is shipped as a single binary which is a light command line interface.

It fills the same role as its alternative like libvirt, virt-manager, virtualbox, or gnome-boxes.

| Tool type         | Crocuda stack    | A trivial stack |
| ----------------- | ---------------- | --------------- |
| multi VM manager  | virshle          | libvirt         |
| single VM manager | cloud-hypervisor | qemu            |
| hypervisor        | kvm              | kvm             |

## Why does it exists?

I have been facing many issues while digging the virtual machine rabbit hole.

- Configure and Provision the same machine over and over.
- Network understanding and configuration is tough.
  Need to use bridges, interface, routers...

Answer:

- Preconfigured VM with nixos.

- Stop juggling with linux network concepts and directly rewrite network packet headers
  with the lowest API possible.
  [openvswitch](https://github.com/openvswitch/ovs)
  and
  [rex](https://github.com/rex-rs/rex)

# Installation.

## NixOs (with flakes).

When using nixos, you can enable the module by adding those lines to your configuration.
Add the repo url to your configuration.

{% container(type="warning") %}

You'll face a substential compilation time due to **virshle** and **pipelight**
not being precompiled in any official repository.

{% end %}

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

### Set a custom storage.

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
