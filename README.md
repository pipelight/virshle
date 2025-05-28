# Virshle: Virtual Machines with .

Create virtual machines from templates.

## Install (NixOs)

Install the NixOs module via flakes.

But this isn't enough to add network connectivity to VMs,
So make sure you have your host network configuration as in
`modules/networkint.nix`.

## Usage.

### Create your first VM.

Put some template definition into the configuration file.

A functional machine needs at least :

- A bootable OS disk.
- A network configuration.

See the template below that defines a small machine preset named `xs`.

```toml
# /etc/virshle/config.toml
[[template.vm]]
name = "xs"
vcpu = 1
vram = 2

[[template.vm.disk]]
name = "os"
path = "~/Iso/nixos.efi.img"
size = "50G"

[[template.vm.net]]
name = "main"
[template.vm.net.type.vhost]
```

Then you can create a machine from that template.

```sh
v vm create -t xs
```

Of course you can list your vm and their state with a simple command.

```sh
v vm ls
```

![vm_list](https://github.com/pipelight/virshle/blob/master/public/images/vm_list.png)

Then start your vm

```sh
v vm start --id <vm_id>
```

### Access your VM

Virshle only allows you to access VMs through **ssh**.

_However, when you want to attach the VM to your terminal,
you can create the VM with `virshle` and boot it from `cloud-hypervisor`._

As of today, this is the default and only network configuration available.

```toml
[[template.vm.net]]
name = "main"
[template.vm.net.type.vhost]
```

Your VM network is managed by Open_vSwitch.
It essentially creates a virtual switch for your every VM.
this switch is then bridged to your network

## How this work.

### The stack

Virshle is only some **Rust** glue that stick together:

- cloud-hypervisor (VM)
- Open_vSwitch (Network)

I wanted to make a tool that is rather decorrelated from the linux kernel.
So the network is handled in user space, outside of the kernel through
[Open_vSwitch](https://github.com/openvswitch/ovs)
And virtualization is made through
[cloud-hypervisor](https://github.com/cloud-hypervisor/cloud-hypervisor)
which supports multiple hypervisors.

### Resources management

Resources from your template definition
are copied into virshle working directory `/var/lib/virshle/`.

![working_directory_tree](https://github.com/pipelight/virshle/blob/master/public/images/working_directory_tree.png)

Which mean that your virtual machine do not run on the disk at
`~/Iso/nixos.efi.img`, but on a copy of that disk, leaving the original file
unaffected.

You can then twist numerous copies of the same machine
by spamming this same command.

## Alternatives

Virshle is in the vein of our good old [libvirt](https://libvirt.org/).

It's aim is to be a comfortable cli to spin up your VM from.

If you want VM for specific usage like workload isolation,
some hypervisors may suit you better:

- [Firecracker](https://github.com/firecracker-microvm/firecracker)
- [CrosVm](https://chromium.googlesource.com/chromiumos/platform/crosvm)
