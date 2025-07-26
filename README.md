# Virshle: Virtual Machine Manager.

Virshle is a single command line utility to manage multiple virtual machines.

It works on top of
[cloud-hypervisor](https://github.com/cloud-hypervisor/cloud-hypervisor)
and
[linux-kvm](https://linux-kvm.org/page/Main_Page)
for machines virtualization,
and makes extensive use of
[openvswitch](https://github.com/openvswitch/ovs)
for network configuration.

## Install

### NixOs (with flakes).

Add the repo url to your configuration.

```nix
# flake.nix
inputs = {
  virshle = {
      url = "github:pipelight/virshle";
  };
};
```

Enable the service.

```nix
# default.nix
services.virshle = {
    enable = true;
    logLevel = "info";
    # The user to run the node as.
    user = "anon";
};
```

See [docs/install.md](https://github.com/pipelight/virshle/docs/install.md)
for other distributions.

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

## Other configurations

- **Custom disk images**,
  See [docs/custom_disk.md](https://github.com/pipelight/virshle/blob/master/docs/custom_disk.md)

- **Multiple nodes**,
  Configure a cluster of multiple nodes.
  See [docs/multi_node.md](https://github.com/pipelight/virshle/blob/master/docs/multi_node.md)

- **Network configuration**,
  Different network configurations.
  See [docs/network.md](https://github.com/pipelight/virshle/blob/master/docs/network.md)

## Alternatives

Similar software can be found at:

- [libvirt applications](https://libvirt.org/apps.html),
- [multipass](https://github.com/cannonical/multipass)
- [aurae](https://github.com/aurae-runtime/aurae)

If you want VM for specific usage like workload isolation,
some hypervisors may suit you better:

- [Firecracker](https://github.com/firecracker-microvm/firecracker),
- [CrosVm](https://chromium.googlesource.com/chromiumos/platform/crosvm)
