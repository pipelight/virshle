+++
date = 2025-09-11
updated = 2026-02-10

weight = 0

title = "Introduction"

description = """

"""

draft=false
+++

# Introduction

Virshle is a VMM (virtual machine manager).
It is a cli (command line interface) written in Rust.

It fills the same role as its alternatives like
libvirt and virt-manager,
virtualbox,
or gnome-boxes.

## Why does it exists?

I wanted to test
[pipelight](https://github.com/pipelight/pipelight)
inside a VM to ensure that it could work as a git forge on a remote server.
But oh boy virtualization isn't easy.

Wrestling with existing VMMs and network tooling, had me learned the fundamentals to finally
bring together a VMM I enjoy using.

{% container(type="tip") %}
**The ultimate goal.**

Virshle aims to provide working VMs (virtual machines) with a
predefined configuration and internet access
**as fast as possible, and with the lightest burden possible for the end user.**

{% end %}

Of course living up to those assumptions has a price.

Virshle can only be installed on
[NixOs](https://nixos.org/)
due to heavy network pimping,
and only NixOs based VMs are supported (need to use a compatibility module/flake).

## Show me the stack!

The building blocks are _rusty_ as well.

Virshle works on top of
[cloud-hypervisor](https://github.com/cloud-hypervisor/cloud-hypervisor)
and
[linux-kvm](https://linux-kvm.org/page/Main_Page)
for machine virtualization.

| Tool type         | Crocuda stack    | A trivial stack |
| ----------------- | ---------------- | --------------- |
| multi VM manager  | virshle          | libvirt         |
| single VM manager | cloud-hypervisor | qemu            |
| hypervisor        | kvm              | kvm             |

It uses
[pipelight-init](https://github.com/pipelight/pipelight)
as a
[cloud-init](https://cloud-init.io/)
replacement for fast vm provisioning on boot.

Network wise packet routing is currently handled by
[openvswitch](https://github.com/openvswitch/ovs)
a more flexible framework than
[netfilter](https://netfilter.org/)
and common networking tools, devices and concepts(ip, tap, bridges...)

But network will ultimately be taken care of with
[rex](https://github.com/rex-rs/rex)
to programmatically route packets with short and straightforward code.
