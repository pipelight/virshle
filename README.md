<span>
<h1>
<img width="125px" alt="virshle_logo" src="https://github.com/pipelight/virshle/blob/master/public/images/logo.png"/>
<p>Virshle: A Virtual Machine Manager.</p>
</h1>
</span>

Virshle is a single command line utility to manage multiple virtual machines.

- It works on top of [cloud-hypervisor](https://github.com/cloud-hypervisor/cloud-hypervisor)
  and
  [linux-kvm](https://linux-kvm.org/page/Main_Page)
  for machines virtualization,
- Makes extensive use of
  [openvswitch](https://github.com/openvswitch/ovs)
  for network configuration.
- Use [pipelight-init](https://github.com/pipelight/pipelight) for fast
  vm provisioning on boot.

> [!WARNING]
> Early development stage.
> Multinode is not stable.

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
[template.vm.net.type.macvtap]
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
# Pattern
v vm start --id <vm_id> --attach
```

### Start and provision a VM.

You need to provide the vm with **user_data** such as an ssh public key
for remote access.

Here for we do not use the now conventional
[cloud-init](https://cloudinit.readthedocs.io/en/latest/explanation/introduction.html#introduction),
but rather
[pipelight-init](https://github.com/pipelight/pipelight)
which shines when it comes to provisioning speed.

You can add user data to the Vm on creation or upon start up.

- From a simple file.

```toml
# user-data.toml
[ssh]
[[ssh.user]]
name = "anon"
authorized_keys = [
  "ssh-ed25519 AAAAC3NzXXX",
]
```

- With the following command

```sh
# Pattern
v vm create --template <template_name> --user_data <user_data_filepath>
v vm start --id <vm_id>

# or
v vm create --template <template_name>
v vm start --id <vm_id> --user_data <user_data_filepath>

# Example
v vm create --template xs --user_data user_data.toml
#or
v vm start --id 1 --user_data user_data.toml
```

Then, you can access your via **ssh**:

- On your local node and without any network configuration (vsock),

  ```sh
  ssh vm/<vm_name>
  ```

- with network,

  Add a network configuration to the vm template

  ```toml
  # /etc/virshle/config.toml
  [[template.vm.net]]
  name = "main"
  [template.vm.net.type.macvtap]
  ```

  ```sh
  v vm create -t <template_name>
  ```

  ```sh
  v vm start --id <vm_id>
  ```

  With the vm_ip,

  ```sh
  ssh <vm_ip>
  ```

  With the vm_name,
  depends on your dhcp/dns configuration.

  ```sh
  ssh vm-name.vm
  ```

### Access your VM

Either attach the vm to a terminal standard outputs.

```sh
v vm start --id <vm_id> --attach
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

## Install

### NixOs (with flakes).

Add the repo url to your flake inputs.

```nix
# flake.nix
inputs = {
  virshle = {
      url = "github:pipelight/virshle";
  };
};
```

Add the module to your configuration.

```nix
nixosConfiguration = {
    default = pkgs.lib.nixosSystem {
        modules = [
            inputs.acceptxmr.nixosModules.default
        ];
    };
}
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

## Build a disk image.

### Minimal image

```nix

```

### Custom image

You can of course create custom disk image by importing
the base module onto a vm configuration.

```nix
xxs = inputs.nixos-generators.nixosGenerate {
  format = "raw-efi";
  modules = [
    inputs.virshle.nixosModule.nixos-generators
    {virtualisation.diskSize = 20 * 1024;}
    # Add your modules
    ./my/custom/config

  ];
};
```

See [docs/install.md](https://github.com/pipelight/virshle/blob/master/docs/install.md)
for in depth nixos installation and **other distributions**.

## Alternatives

Similar software can be found at:

- [libvirt applications](https://libvirt.org/apps.html),
- [multipass](https://github.com/cannonical/multipass)
- [aurae](https://github.com/aurae-runtime/aurae)

If you want VM for specific usage like workload isolation,
some hypervisors may suit you better:

- [Firecracker](https://github.com/firecracker-microvm/firecracker),
- [CrosVm](https://chromium.googlesource.com/chromiumos/platform/crosvm)
