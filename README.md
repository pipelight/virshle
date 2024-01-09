# Virshle - Dig out Virtual machines with TOML/YAML/JSON.

Define your virtual machines, network and volumes **TOML, YAML and JSON**.

Written in typescript/deno. Based on libvirt (virsh command line tool).

- Keep commands concise with few options and arguments (ex:
  `virshle vm create ./base/vm/default.toml`)
- Readable definitions in markup.

## Debug

Virshle adds a verbose flag `-vvvv` for you to see the underlying Markup to XML
convertion.

## Usage

This is how you would define a domain (VMs).

The following defines a VM called "nixos",

- with 2cpu and 4GiB of RAM
- attached to the default network
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

# [domain.devices]
# emulator = "/usr/bin/env qemu-kvm"

[[domain.devices.disk]]
"@type" = "file"
"@device" = "disk"
driver."@name" = "qemu"
driver."@type" = "qcow2"

"@bus" = "virtio"
"@size" = 20

# source."@file" = "~/ISO/nixos-crocuda.qcow2"
source."@file" = "/home/anon/ISO/nixos.qcow2"
target."@dev" = "hda"
# target."@bus" = "virtio"

[[domain.devices.interface]]
"@type" = "network"
source."@network" = "default"
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

On the command line things get a bit different too.

```sh
virshle vm create file.toml
```

is translated to

```sh
virsh domain create file.xml
```

## Contribute

Update dependencies

```sh
deno cache --reload ./mod.ts
```

Run main script.

```sh
deno run -A mod.ts
```

or

```sh
./mod.ts
```

Run tests.

```sh
deno test
```

## Nasty purposes 😈

The goal here is to be able to dig out shit tons personnalized virtual machines.

Nixos has bultin features to build iso based on configuration file. This
bypasses the usual provisionning.

The combination of a custom nixos image and an already provisionned volume for
secret storage allow for extremly fast deployments (~20 seconds) of complete up
and running machines.

## S/O

Inpired by mario and nushell.
