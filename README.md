# Virshle - A cli to manage virtual machines.

Define your virtual machines(VM) with **TOML**.

This is essentialy the virsh command line tool with some features.

- [x] support for easy file formats (Toml)
- [x] add support for relative paths inside vm definition
- [ ] templates for trivial VMs.

## Usage

### Prerequisit

Install libvirt.

### Tweak existing VMs

This will open the VM definition within your favorite editor.

```sh
virshle vm edit my_machine.toml
```

### Define new VMs

This is how you would define a VM.

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

Bring it up with

```sh
virshle vm create network.toml
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
virshle net create network.toml
```

## Debug

Virshle adds a verbose flag `-vvvv` for you to see the underlying Markup to XML
convertion.

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
