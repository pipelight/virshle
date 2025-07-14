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

## Architecture

There is a cli(client) that can control multiple nodes(servers) that manage multiple vms.

Connection between the client and servers are done through
unix-socket, or ssh.

```txt
┌──────┬──────┐            ┌──────┬──────┐
│      │      │            │      │      │
│      │      │            │      │      │
│ vm_1 │ vm_2 │            │ vm_1 │ vm_2 │
│      │      │            │      │      │
│      │      │            │      │      │
├──────┴──────┴──────┐     ├──────┴──────┴──────┐
│    node_1          │     │    node_2          │
└─────▲──────────────┘     └─────▲──────────────┘
      │                          │
      │                          │
      │                          │
     ┌┴────────┬─────────────────┘
     │         │
     │  cli    │
     │         │
     └─────────┘
```

## Getting started.

### Start a node.

Create the required resources:

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

### Connect to the node.

#### Local node

When running a node on your local machine,
the cli automatically connects to the node unix-socket without further
configuration.

While listing available nodes, your local node appears with the name `default`.

```sh
virshle node ls -vvv
```

![node_list_default](https://github.com/pipelight/virshle/blob/master/public/images/v_node_ls_vvv_default.png)

#### Remote node

You can create a list of manageable nodes in the configuration file at
`/etc/virshle/config.toml`

```toml
# /etc/virshle/config.toml

# local host
[[node]]
name = "local"
url = "unix:///var/lib/virshle/virshle.sock"

# local host through ssh
[[node]]
name = "local-ssh"
url = "ssh://anon@deku:22/var/lib/virshle/virshle.sock"
```

_When specifying nodes url,
you have to explicitly write your local node address if you want to use it._

```sh
virshle node ls -vvv
```

![node_list_multi](https://github.com/pipelight/virshle/blob/master/public/images/v_node_ls_vvv_multi.png)

### Create your first VM.

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
[template.vm.net.type.tap]
```

Then you can create a machine from that template.

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
[template.vm.net.type.tap]
```

```sh
v vm start --id <vm_id>
```

```sh
ssh <vm_ip>
```

## Internals

### Resources management

Resources from your template definition
are copied into virshle working directory `/var/lib/virshle/`.

![working_directory_tree](https://github.com/pipelight/virshle/blob/master/public/images/working_directory_tree.png)

Which mean that your virtual machine do not run on the disk at
`~/Iso/nixos.efi.img`, but on a copy of that disk, leaving the original file
unaffected.

You can then twist numerous copies of the same machine
by spamming this same command.

### Network management

VMs are attached to a virtual openvswitch bridge (br0).

To add outside network connectivity, you need to add your main
interface to the bridge.

[https://github.com/pipelight/virshle/virshle_core/src/network/README.md]

### DHCP (Work in progress)

You may want virshle to report VM ips.

![vm_list](https://github.com/pipelight/virshle/blob/master/public/images/v_vm_ls_v.png)

You need a configured KeaDHCP(v4 or v6 or both) instance running somewhere.
Then add the connection url to your configuration.

```toml
[dhcp]
[dhcp.kea]
url = "tcp://localhost:5547"

```

## Install

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
};
```

## Alternatives

Virshle is a **level 2 hypervisor** in the vein of our good old
[libvirt](https://libvirt.org/).
Its aim is to be a comfortable cli to spin up your VM from.

It was originally designed to be fancy replacement of the virsh command line
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
