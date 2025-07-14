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

## Node management.

Virshle is a cli(client) that can control multiple nodes(servers)
that manage multiple vm(virtual machine).
So before creating a vm, you'll have to **spin up a node first**.

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

### Start a node.

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

### Connect to a local node.

When running a node on your local machine,
the cli automatically connects to the local node unix-socket
without further configuration.

While listing available nodes, your local node appears with the name `default`.

```sh
virshle node ls -vvv
```

![node_list_default](https://github.com/pipelight/virshle/blob/master/public/images/v_node_ls_vvv_default.png)

### Connect to remote nodes.

Connection between the client and servers are done through
**unix-sockets** or **ssh**.

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

For virshle to access a node through ssh, it needs the **authorized_key**
into a running **ssh-agent**.
Make sure you have an ssh-agent running with your key loaded inside.

```sh
virshle node ls -vvv
```

![node_list_multi](https://github.com/pipelight/virshle/blob/master/public/images/v_node_ls_vvv_multi.png)

### Node load balancing.

When you work with multiple nodes, and create a machine with
`v vm create -t xs`
without giving a node to work on
`--node <node_name>`,

The **load balancer** chooses a random (and not saturated) node,
You can add a `weight` to the node if you want it to be chosen
more often.

```toml
# /etc/virshle/config.toml

# local host
[[node]]
name = "remote_1"
url = "ssh://anon@remote_1:22/var/lib/virshle/virshle.sock"
weight = 10

# local host through ssh
[[node]]
name = "remote_2"
url = "ssh://anon@remote_2:22/var/lib/virshle/virshle.sock"
weight = 2
```

### Node health check.

Instead of troubleshooting the node by hand with your favourite tools(df, free, htop...),
you may have a quick glance at your node global state.

```sh
virshle node ls -all -vvv
```

![node_list_all](https://github.com/pipelight/virshle/blob/master/public/images/v_node_ls_all_vvv.png)

Here can you see **used resources**,
plus **reserved resources** for your VMs.

For example, you can, of course, reserve more CPUs than what you physically have on a host
and the linux kernel will share the power between guests.

## Vm management.

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
[template.vm.net.type.tap]
```

Then only can you create a machine from that template.

```sh
v vm create -t xs
```

Or create a vm on a remote node.

```sh
v vm create -t xs --node <node_name>
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

## Network configuration

Virshle tries to lower the amount of black magic needed for network configuration.
Every command needed to create a network interface can be seen in the logs of
`v node serve -vvvv`

### MacVTap

The fastest way to add networking connectivity to a VM is with a `mac_v_tap`.
Add the following network configuration to a vm template.

```toml
[[template.vm.net]]
name = "main"
[template.vm.net.type.mac_v_tap]
```

It uses the linux network [ip](https://www.man7.org/linux/man-pages/man8/ip.8.html) command
to create a mac_v_tap interface that will be bound to the VM.

No need for a bridge here, the upstream router gives ips and network access to the VM.

### Bridge and Tap

For a fine-grained control of your network, prefer the tap device.

```toml
[[template.vm.net]]
name = "main"
[template.vm.net.type.tap]
```

It uses the [ip](https://www.man7.org/linux/man-pages/man8/ip.8.html) command and
[openvswitch](https://github.com/openvswitch/ovs)
commands (`ovs-vsctl`) to create a VM dedicated brigde/switch (br0).

Then freshly created VMs are attached to this bridge(br0) via a tap device.

To add outside network connectivity, you need to add your main
interface to the bridge.

Checkout your network configuration with.

`ip l`
`ovs-vsctl show`

More on network: [https://github.com/pipelight/virshle/virshle_core/src/network/README.md]

### DHCP (Work in progress)

Virshle relies on external software to manage Vm ips,
like [KeaDHCP](https://kea.readthedocs.io/en/latest/)
You need a configured KeaDHCP(v4 or v6 or both) instance running somewhere.

Then add the connection url to your configuration.

```toml
[dhcp]
[dhcp.kea]
url = "tcp://localhost:5547"
```

Dhcp leases managed by KeaDHCP shows up once configured.

```sh
v vm ls -v
```

![vm_list](https://github.com/pipelight/virshle/blob/master/public/images/v_vm_ls_v.png)

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
};
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
by spamming the same command.

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
