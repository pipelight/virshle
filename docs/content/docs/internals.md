+++
date = 2025-09-11
updated = 2026-02-13

weight = 30

title = "How it works."

draft = false
+++

# How it works.

## The template centric flow.

{% container(type="tip") %}
**tl;dr**

Virshle creates a virtual machine by cloning a reference template
and injecting user-data.

{% end %}

{% container(type="info") %}

**Declarative approach to VM creation.**

Virshle duplicates the template disks
and creates every resource according to the template definition.

This means that your virtual machine do not run on the disk defined in the template
but on a **copy of that disk**,
leaving the **original file unaffected**.

As a result you can twist numerous copies of the same machine
by spamming `v create -t <template_name>`.

{% end %}

You first have to edit a **base template** and custom user data.

- `/etc/virshle/config.toml`
  -> Your virtual machine templates.

- `./user-data.toml`
  -> User defined data for configuration after boot.

{% sbs() %}

```toml
# /etc/virshle/config.toml

#########################
# Templates:
# Vm standard sizes with decents presets.
[[template.vm]]
name = "xxs"
vcpu = 1
vram = "1GiB"
[[template.vm.disk]]
name = "os"
path = "/var/lib/virshle/cache/nixos.xxs.efi.img"
# path = "~/Iso/nixos.xxs.efi.img"
[[template.vm.net]]
name = "main"
[template.vm.net.type.tap]
```

```toml
# ./user-data.toml

#########################
# Conventional user-data added to the VM.
[[user]]
name = "anon"
[user.ssh]
# Key is appended at /etc/ssh/authorized_keys.d/<user.name>
authorized_keys = ["ssh-ed25519 AAAAC3N..."]
```

{% end %}

Then you create a machine based on that template.

```sh
# v vm create --template <template_name> --user-data <user_data_filepath>
v vm create --template xxs --user-data ~./user-data.toml

v vm ls # get the id of newly created vm.

# v vm start --id <vm_id>
v vm start --id 1
```

## Indefectible machines

When you `create` a VM, the VM parameters are persisted to the `virshle.sqlite` database,
and later used when you `start` the VM to spawn a **bare cloud-hypervisor process**.

So much that the following commands result in the same process being launched.

{% sbs() %}

```sh
virshle vm create \
	--template <template_name>

virshle vm start \
	--name <vm_name>
```

```sh
cloud-hypervisor \
	--kernel <bootloader> \
	--disk path=<disk_image> \
	--cpus boot=1 \
	--memory size=1024M \
	--net "tap=,mac=,ip=,mask="
```

{% end %}

See for yourself the running process.

```sh
ps -aux | rg cloud-hypervisor` #or grep
```

By design,
the Virshle node **never takes ownership** of the cloud-hypervisor processes
which are de facto **orphan child processes**.

{% sbs() %}

```sh
virshle
├── cloud-hypervisor
├── cloud-hypervisor
└── cloud-hypervisor
```

```sh
x # dead parent process
├── cloud-hypervisor # keeps running
├── cloud-hypervisor # keeps running
└── cloud-hypervisor # keeps running
```

{% end %}

Which means that whenever the node daemon
restarts, stops, fail, or is unexpectedly interrupted,
**machines keep running**.

{% container(type="success") %}

Unless the host is out of power or severely corrupted,
**machines keep running**.

{% end %}

So of course, you can `nixos rebuild switch` `update and rollback the host`
without it impacting any VM.
The freshly started Virshle daemon regains control of the orphaned cloud-hypervisor processes.

## Client <-> Node communication

Gathered under the same cli,
Virshle is a composed of:
A client and a node that can communicate over a **http Rest API**.

{% sbs() %}

{% container(type="null") %}

- A client that can interact with one or many local and remote nodes:
  for example the command
  `v vm <action> --node <node_name>`
  ->
  _the client ask a node execute an action on a vm on this node_.

- A node (daemon) that manages multiple VMs on the host it is installed on:
  for example the command
  `v node serve`
  ->
  _starts a local node that listen for clients requests_.

{% end %}

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

{% end %}

## Resource usage

Every Virshle resource are stored inside the working directory `/var/lib/virshle`.

```sh
/var/lib/virshle # Virshle working directory
├── cache # A convenient path to keep your favorite disk images.
│   ├── nixos.s.efi.img
│   ├── nixos.xs.efi.img
│   └── nixos.xxs.efi.img
├── virshle.sock # Socket for virshle Rest API.
├── virshle.sqlite # Database containing Vm definitions.
└── vm
    ├── 1dd70ae9-cd1c-41ab-b3c4-9c1839bb12ce # Vm uuid
    │   ├── ch.sock # socket for cloud-hypervisor Rest API
    │   ├── ch.vsock # socket to ssh inside vm without network
    │   ├── disk
    │   │   ├── os # The main disk
    │   │   └── pipelight-init # custom user-data disk
    │   ├── net
    │   └── tmp # Working dir for mounting user-data disk
    └── 70c4f164-3f88-4d8a-89a4-602803630d1a
        ├── ch.sock
        ├── ch.vsock
        ├── disk
        │   ├── os
        │   └── pipelight-init
        ├── net
        └── tmp
```

Having the most of the VM as files makes them extremely portable.
Virshle VMs are easy to migrate across servers, networks and data centre.

### Disks storage

The vm disks and socket are stored under `/var/lib/virshle/vm/<vm_uuid>`.

And every disk mounting tasks (pipelight-init) are made inside the vm temporary directory
`/var/lib/virshle/vm/<vm_uuid>/tmp`

{% container(type="info") %}
**Unstable feature**

The `cache` directory is only used to store some of your favourite disks.
It will be used as way to synchronize template disks between nodes in further releases.
{% end %}

### Network resilience

The network configuration inside the Linux kernel is volatile and can be disturbed at time.

Each time the host **restarts** or **switches configuration** its default network is regenerated against a configuration file (`network.nix`), thus removing network connectivity for every running VMs.

The `virshle-refresh-network.service` unit adds up some persistence
and ensures that network is regenerated for each running VM.

You can do it manually by running the following command:

```sh
v node init --net
```

Nevertheless,
this implies a loss of connectivity of some milliseconds each time when the host `rebuild-switch`,
and this can ramp up to a few seconds in case of failed host update and rollback.

### Host network configuration

See for yourself the resulting host network configuration.

```sh
ip address
# or
ovs-vsctl show
```

```sh
# Output of command: `ip address`

1: lo: # Host loopback interface
2: eno1: # The main ethernet interface split between br0 and vs0 bridges.
3: ovs-system: X
4: br0: # The dedicated VM bridge.
5: vs0: # The dedicated host bridge.

6: vs0p1: # The host interface

## VM interfaces of type ovs-internal-tap(!= tap).
7: vm-akira-skywal:
8: vm-jay-braun--m:

```

## Limitations (&improvements)

### Capped machine number

VM network interfaces have a limited name length of 15 characters (Unix standard residue),
and a natural name for fast manual troubleshooting,
which greatly limit the amount of unique names that can be generated.

Consequently, even if RAM, CPU, and Disks are able to handle the workload,
a **host can't contain more than a few hundred VMs**.

{% container(type="warning") %}

Fortunately this limit (100 VM max.) is to be lifted by the usage of [rex](https://github.com/rex-rs/rex) for network packet routing instead of the Linux kernel network API.

{% end %}

### Much better on NixOs

I originally developed this solution for Arch and Debian,
but NixOs offers a great comfort for painless:

- custom disks images build,
- remote machines update,
- surgical operating system tweaks,

It is the **declarative configuration** that propels
**fearless deep exploration and personalization** of ones operating system,
that allowed Virshle to exist in a fraction of the time it would have taken
to build it on other distributions.
