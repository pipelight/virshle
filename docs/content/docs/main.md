+++
date = 2025-09-11
updated = 2026-02-13

weight = 0

title = "Introduction"

description = """

"""

draft=false
+++

# Introduction

Virshle is a virtual machine manager (VMM).

It is a single command line utility (cli) written in Rust,
to be used from inside a terminal.
It allows you to create and manage multiple virtual machines (VM:
isolated computers running inside your computer).

Virshle primarily focuses on **terminal experience** and **agreeable configuration files**.

## Why does it exists?

I wanted to test
[pipelight](https://github.com/pipelight/pipelight)
inside a VM to ensure that it could work as a git forge on a remote server.
But oh boy virtualization isn't easy.

Wrestling with existing VMMs and network tooling, had me learned the fundamentals to finally
bring together a VMM I enjoy using.

{% container(type="tip") %}

**The ultimate goal.**

Virshle aims to provide immediately usable pre-configured VMs with internet access
**as fast as possible, and with the lightest burden possible for the end user.**

{% end %}

Of course living up to those assumptions has a price.

Virshle can only be installed on
[NixOs](https://nixos.org/)
due to heavy network pimping,
and only NixOs based VMs are supported (need to use a compatibility module/flake).

## Why the name?

Virshle is in the vein of our good old
[libvirt](https://libvirt.org/).

It was originally designed to be a fancy replacement for the virsh (libvirt cli)
with a hint of **toml** (virsh + toml = virshle),
however the need of flexibility quickly evinced libvirt,
in favour of
[cloud-hypervisor](https://github.com/cloud-hypervisor/cloud-hypervisor)

And the name stuck.

## Show me the stack!

The building blocks are _rusty_ as well.

Here is a comparision of virshle building blocks
and the well-known kvm/qemu/libvirt setup.

A trivial stack:

| Role              | Tool    | Language | Configuration | Friendliness |
| ----------------- | ------- | -------- | ------------- | ------------ |
| multi VM manager  | libvirt | C        | XML           | 😭           |
| single VM manager | qemu    | C        | cli args      | 😑           |
| hypervisor        | kvm     | C        | ioctl         |              |

Virshle stack:

| Role              | Tool             | Language | Configuration   | Friendliness |
| ----------------- | ---------------- | -------- | --------------- | ------------ |
| multi VM manager  | virshle          | Rust     | Toml            | 😀           |
| single VM manager | cloud-hypervisor | Rust     | Rest API (json) | 😀           |
| hypervisor        | kvm              | C        | ioctl           |              |

Virshle works on top of
[cloud-hypervisor](https://github.com/cloud-hypervisor/cloud-hypervisor)
and
[linux-kvm](https://linux-kvm.org/page/Main_Page)
for machine virtualization.
The Rest API makes it very easy to manage machines on the fly.

It uses
[pipelight-init](https://github.com/pipelight/pipelight)
a way faster
[cloud-init](https://cloud-init.io/)
replacement for vm provisioning on boot.

Network wise packet routing is currently handled by
[openvswitch](https://github.com/openvswitch/ovs)
a more flexible framework than
[netfilter](https://netfilter.org/)
and common networking tools, devices and concepts(ip, tap, bridges...).

However network will ultimately be taken care of with
[rex](https://github.com/rex-rs/rex)
to programmatically route packets with short and straightforward code.

## Alternatives

Some of the most known similar virtualization software can be found at:

- [micorVM.nix](https://github.com/microvm-nix/microvm.nix),
- [libvirt applications](https://libvirt.org/apps.html),
- [multipass](https://github.com/cannonical/multipass),
- [aurae](https://github.com/aurae-runtime/aurae)

If you want VMs for specific usage like workload isolation,
some hypervisors may suit you better:

- [Firecracker](https://github.com/firecracker-microvm/firecracker),
- [CrosVm](https://chromium.googlesource.com/chromiumos/platform/crosvm)
