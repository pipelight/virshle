# Virshle - A modern libvirt wrapper.

A wrapper around the virsh cli. It makes it possible to define your virtual
machines in readable formats like **TOML, YAML and JSON** instead of XML.

## Motivations

There is actually a lot of tool that enable abstraction around libvirt which is
itself an abstraction for many virtualization tool like qemu.

You may ask why another ?

Virshle is not abstraction, it is a convenience wrapper for those who want to use
libvirt to keep control on there vm but with more readable files.

## Warning - Deno bundle size

Virshle is a very small piece of software written in deno wich means it depends
on the deno runtime which weights around **30Mb**. It is light still but quite
heavy for what it does and may be refactor in Rust or Go.

## Convert xml to toml

Replace xml inner tag argumemts by prefixing them with a "@":

```xml
<domain type="kvm" />
```

```toml
[domain]
"@type" = "kvm"
```

## Example

This is how you would define a domain.

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

[network.ip.dhcpcd.range]
"@start" = "192.168.122.2"
"@end" = "192.168.122.254"
```

Plus some extra features

## Usage

Instead of typing

```sh
virsh domain create file.xml
```

Replace the usual virsh by virshle.

```sh
virshle domain create file.toml
```

## S/O

Inpired by mario and nushell.
