# Virshle - Virtual machines from the command line.

Virsh + Toml = Virshle.

A command line interface to replace virsh.

- [x] define vm, network and storage with **TOML**.
- [x] use relative paths inside vm definition.
- [x] use templates for trivial VMs creation.
- [ ] autodetect the resource type (vm, network...).

## 🚀 Get started!

```sh
# Create resources
virshle create ./template/vm/base.toml

# List resources
virshle ls
virshle ls vm
virshle ls network

# Delete resources
virshle delete <resource_name>

```

### Define a virtual machine (domain)

The following Toml file defines a VM called "nixos":

- with 2cpu and 4GiB of RAM
- attached to a default network
- based on a custom nixos image

```toml
[domain]
"@type" = "kvm"
name = "nixos"
uuid = "4dea24b3-1d52-d8f3-2516-782e98a23fa0"
memory = 140000
vcpu = 2

[domain.os.type]
"@arch" = "x86_64"
"#text" = "hvm"

[domain.clock]
"@sync" = "localtime"

[domain.devices]
emulator = "/run/qemu-kvm"

[[domain.devices.disk]]
"@type" = "file"
"@device" = "disk"
driver."@name" = "qemu"
driver."@type" = "qcow2"

"@bus" = "virtio"
"@size" = 20

source."@file" = "ISO/nixos.qcow2"
target."@dev" = "hda"
target."@bus" = "virtio"

[[domain.devices.interface]]
"@type" = "network"
source."@network" = "default"
```

Bring the guest up with,

```sh
virshle create ./template/vm/base.toml
```

This is how you would define a network.

```toml
[network]
name = "default"
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
virshle create ./template/network/network.toml
```

### Debug

Increase verbosity.

```sh
virshle create <file> -vvvv
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
nix shell github:pipelight/pipelight
```

Install it on your system.

```nix
{
  description = "NixOS configuration for crocuda development";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    virshle.url = "github:pipelight/pipelight";
  };

  outputs = {
    nixpkgs,
    pipelight,
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
