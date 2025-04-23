# Virshle - Painless virtual machines.

Features:

- Light virtual machines.
- Write resource definitions in **TOML**.
- Use **predefined [templates](https://github.com/pipelight/virshle/virshle_core/virshle.config.toml)**
  to twist numerous clones of the same machine.

> [!IMPORTANT]  
> Tool in early development stage.
> I wanted something as fast as possible,
> so of course it runs on edgy tech by default.😈
>
> - openvswitch-dpdk (network)
> - cloud-hypervisor (virtual machine manager)

## 🚀 Get started!

### Debug

You can increase verbosity for each command and get detailed logs.

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
name = "vm-nixos"
uuid = "4dea24b3-1d52-d8f3-2516-782e98a23fa0"
vcpu = 2
vram = 2 # in GiB

[[disk]]
path = "./disk/path"

[[net]]
name = "default"

```

Bring the guest up with,

```sh
virshle vm create <vm_definition>.toml
```

This is how you would define a network.

```toml
[net]
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

## Thanks

Inspired by:

- [virsh](https://github.com/libvirt/libvirt).
- [docker](https://github.com/docker/compose).
