# Virshle: Virtual Machine Manager.

Virshle is a single command line utility to manage multiple virtual machines.

It works on top of
[cloud-hypervisor](https://github.com/cloud-hypervisor/cloud-hypervisor)
and
[linux-kvm](https://linux-kvm.org/page/Main_Page)
for machines virtualization.
And makes use of
[openvswitch](https://github.com/openvswitch/ovs)
for network configuration.

## Install

Mandatory dependencies:

- [cloud-hypervisor](https://github.com/cloud-hypervisor/cloud-hypervisor)
- [openvswitch](https://github.com/openvswitch/ovs)

Optional dependencies:

- [KeaDHCP](https://kea.readthedocs.io/en/latest/)

### Cargo

```sh
cargo install --git https://github.com/pipelight/virshle
```

It is recommended to create a systemd unit file to run virshle in the background
and on server boot.

See the
[nixos systemd unit](https://github.com/pipelight/virshle/modules/config.nix).

### NixOs with flakes

Install the nixos module via flakes.

But this isn't enough to add network connectivity to VMs,
So make sure you have your host network configuration as in
[`modules/networking.nix`](https://github.com/pipelight/virshle/modules/config.nix).

```nix
services.virshle = {
    enable = true;
    # Mandator:
    # The user to run the node as.
    user = "anon";
};
```

## Getting started.

Virshle is a cli(client) that can control multiple nodes(servers)
that manage multiple vm(virtual machine) themselves.

So before creating a vm, you'll have to **spin up a node first**.

```txt
┌──────┬──────┐
│      │      │
│      │      │
│ vm_1 │ vm_2 │
│      │      │
│      │      │
├──────┴──────┴──────┐
│   node             │
└─────▲──────────────┘
      │
      │
      │
┌─────┴───┐
│         │
│   cli   │
│         │
└─────────┘
```

### Start a local node.

It is what you want for fast and easy vm creation on your local machine.

The following command creates the required resources on host:

- filesystem paths(/var/lib/virshle) and config path (/etc/virshle),
- vm database(/var/lib/virshle.sqlite),
- client-server communication socket(/var/lib/virshle.sock),
- and host network configuration (ovs-system and br0 switches)

```sh
virshle node init --all

```

Then run the virshle node daemon.

```sh
virshle node serve -vvv
```

### Connect to your local node.

When running a node on your local machine,
the cli automatically connects to the local node
without further configuration.

While listing available nodes, your local node appears with the name `default`.

```sh
virshle node ls -vvv
```

![node_list_default](https://github.com/pipelight/virshle/blob/master/public/images/v_node_ls_vvv_default.png)

### Create a VM.

The preferred way to create VMs with virshle is by the usage of templates.

You add some template definitions into the configuration file.
A functional machine needs at least :

- A bootable OS disk (mandatory),
- Some cpu,
- Some ram,

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

[[template.vm.net]]
name = "main"
[template.vm.net.type.mac_v_tap]
```

Then only can you create a machine from that template.

```sh
v vm create -t xs
```

List your VMs and associated information.

```sh
v vm ls
```

![vm_list](https://github.com/pipelight/virshle/blob/master/public/images/v_vm_ls.png)

Then start your vm.

```sh
v vm start --id <vm_id>
```

### Access your VM

Either attach the vm to a terminal standard outputs.

```sh
v vm start --id <vm_id> --attach
```

Or add a network configuration and connect to the VM through ssh.

```toml
[[template.vm.net]]
name = "main"
[template.vm.net.type.mac_v_tap]
```

```sh
v vm start --id <vm_id>
```

```sh
ssh <vm_ip>
```

![vm_list](https://github.com/pipelight/virshle/blob/master/public/images/v_vm_ls_v.png)

## Alternatives

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

Similar software can be found at:

- [libvirt applications](https://libvirt.org/apps.html),
- [multipass](https://github.com/cannonical/multipass)

If you want VM for specific usage like workload isolation,
some hypervisors may suit you better:

- [Firecracker](https://github.com/firecracker-microvm/firecracker),
- [CrosVm](https://chromium.googlesource.com/chromiumos/platform/crosvm)

## Others

### Comparison with libvirt stack.

|            | virshle  | libvirt              |
| ---------- | -------- | -------------------- |
| config     | toml/kdl | xml                  |
| hypervisor | ch       | many (ch, qemu...)   |
| kernel     | linux    | many (linux, mac...) |

Schemas generated with [asciiflow](https://github.com/lewish/asciiflow).
