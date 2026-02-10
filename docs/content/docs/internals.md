+++
date = 2025-09-11
updated = 2026-02-10

weight = 10

title = "Virshle documentation 📖"

description = """

"""

draft=false
+++

# Internals

Virshle is a **level 2 hypervisor** in the vein of our good old
[libvirt](https://libvirt.org/).
Its aim is to be a comfortable cli to spin up your VM from.

It was originally designed to be a fancy replacement of the virsh command line
which stood on top of libvirt.

But mid development libvirt has been replaced by
[cloud-hypervisor](https://github.com/cloud-hypervisor/cloud-hypervisor)
and
[openvswitch](https://github.com/openvswitch/ovs)
for more flexibility.
And the name stuck.

## Troubleshoot

Virshle was originally a bash script that simply aggregated commands
in order to spin up cloud-hypervisor vm easily.
As I needed more features it became a prototype written in Typescript and
then reached its current form as a rewrite in Rust.

I like when I can see what this kind of tool does to my system,
without having to dive into the code.

Exactly like when you execute a bash script with `set -xe`.

So when increasing the node verbosity, you can transparently see every command
executed in order to
create disks, networks, and virtual machines.

```sh
v node serve -vvv

```

You can manage vms without the need virshle.

I want my virtual machines to remain unaffected by error thrown by the type 2 hypervisor.

As a side effect,

- When the **node daemon goes down**, **machines keep running**.
- Network can and must be tweaked independently with `ovs-vsctl` and `ovs-ofctl`.
- Commands are logged so you can see where things got bad and get muddy by retrying by hand.

## Defunct virshle node.

When you don't know if your machine is running.

```sh
ps -aux | grep cloud-hypervisor`

```

Checkout the node logs.

```sh
systemctl status virshle
journalct -xeu virshle

```

Stop the service and run the node by hand with extra verbosity.

```sh

systemctl stop virshle
v node serve -vvvv
```

## Resources management

### Disks

Disks from your template definition
are copied into virshle working directory `/var/lib/virshle/`.

![working_directory_tree](https://github.com/pipelight/virshle/blob/master/public/images/working_directory_tree.png)

Which mean that your virtual machine do not run on the disk at
`~/Iso/nixos.efi.img`, but on a copy of that disk, leaving the original file
unaffected.

As a result you can twist numerous copies of the same machine
by spamming the same command (`v create -t <template_name>`).
