# Virshle - Painless virtual machines.

Features:

- Manage virtual machines with a **nobrainer cli**.
- Write resource definitions in **TOML**.
- Use **predefined [templates](https://github.com/pipelight/virshle/templates)**.
- Twist numerous clones of the same machine.

> [!IMPORTANT]  
> Tool in very early development stage.
> Should be used complementay to [virsh](https://github.com/libvirt/libvirt)

## 🚀 Get started!

### Debug

You can increase verbosity for each commands and get detailed logs.

```sh
virshle -vvvv
```

### Bulk create from templates

You can define multiple resources in the same file.
And create/ensure them with a single command.

Checkout example in the predefined
[templates](https://github.com/pipelight/virshle/templates) directory.

The following commands creates a different vm everytime it is called.

```sh
virshle create <template_file>
```

###

Manage virtual machines and networks easily.
Commands have been simplified to a minimal CRUD api.

Here is the cli struct.

```sh
virshle <resource> <method>
```

You can manipulate those resources.

| resources     |
| ------------- |
| vm (domain)   |
| net (network) |
| secret        |

Simple operations on resources.

| methods     |
| ----------- |
| create      |
| rm (delete) |
| ls (list)   |

Here is an example of command line usage.

```sh
# List domains (virtual machines, guests)
virshle vm ls
```

![tables comparison](https://github.com/pipelight/virshle/blob/master/public/images/table_base.png)

```sh

virshle vm ls -vv
```

![tables comparison](https://github.com/pipelight/virshle/blob/master/public/images/table_ips.png)

```sh
# List networks

virshle net ls

# Create a domain

virshle vm create ./template/vm/base.toml

# Delete resources

virshle vm rm <vm_name>

```

### Define a virtual machine (domain)

The following Toml file defines a VM called "nixos":

- with 2cpu and 4GiB of RAM
- attached to a default network
- based on a custom nixos image

```toml
[domain]
"@type" = "kvm"
name = "vm-nixos"
uuid = "4dea24b3-1d52-d8f3-2516-782e98a23fa0"
vcpu = 2
[domain.memory]
"@unit" = "GiB"
"#text" = 4

[domain.clock]
"@sync" = "localtime"
[domain.devices]
emulator = "/run/libvirt/nix-emulators/qemu-kvm"

[[domain.devices.disk]]
"@type" = "file"
"@device" = "disk"
driver."@name" = "qemu"
driver."@type" = "qcow2"
"@bus" = "virtio"
"@size" = 20
source."@file" = "./iso/nixos.qcow2"
target."@dev" = "hda"
target."@bus" = "virtio"

[[domain.devices.interface]]
"@type" = "network"
source."@network" = "default"
```

Bring the guest up with,

```sh
virshle vm create ./template/vm/base.toml
```

This is how you would define a network.

```toml
[network]
name = "default_4"
uuid = "9a05da11-e96b-47f3-8253-a3a482e445f5"

forward."@mode" = 'nat'
[network.bridge]
"@name" = "virbr0"
"@stp" = "on"
"@delay" = 0

[network.mac]
"@address" = "52:54:00:0a:cd:21"

[[network.ip]]
"@familly" = "ipv4"
"@address" = "192.168.122.1"
"@netmask" = "255.255.255.0"

[network.ip.dhcp.range]
"@start" = "192.168.122.2"
"@end" = "192.168.122.254"
```

Bring it up with

```sh
virshle net create ./template/network/network.toml
```

## 🛠️ Install

You must have libvirt already installed.

### With Cargo (the Rust package manager)

```sh-vue
cargo install --git https://github.com/pipelight/virshle
```

### With Nixos (and flakes)

Try it in an isolated shell.

```nix
nix shell github:pipelight/virshle
```

Install it on your system.

```nix
{
  description = "NixOS configuration for crocuda development";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    virshle.url = "github:pipelight/virshle";
  };

  outputs = {
    nixpkgs,
    virshle,
  }: {

    # Put this somewhere in your
    # environment system packages
    # user packages
    # or
    # home manager packages
    virshle.packages.${system}.default

  };
}
```

## Roadmap

v0.4.0

- [x] Toml/Xml: automaticaly guess what resource to manipulate based on file root element

Commandes,

- [x] list:
  - [x] vms
  - [x] networks
  - [x] secrets
- [x] create:
  - [x] vms,
  - [x] networks
  - [x] secrets
- [x] delete:
  - [x] vms,
  - [x] networks
  - [x] secrets
- [ ] update:
  - [ ] vms,
  - [ ] networks
  - [ ] secrets

Resources management

- [ ] display vm IPs when verbosity increased (-v)

```toml
[domain.devices.disk.source]
"@file" = "./iso/encrypted.qcow2"
```

## Community/Contrib

Join the matrix room.
https://matrix.to/#/#virshle:matrix.org

## Thanks

Big thanks to libvirt teams who mad it possible with the
[virsh](https://github.com/libvirt/libvirt) cli
and rust libvirt mappings.
Docker [https://github.com/docker/compose] for inspiration.
