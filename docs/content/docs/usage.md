+++
date = 2025-09-11
updated = 2026-02-11

weight = 50

title = "Manage your machines."

draft=false
+++

# Manage your machines.

Once you have installed Virshle and dependencies,
built default bootable disks,
and edited your configuration file as well as your user-data;
you are finally ready to **twist numerous virtual machines
for every imaginable purpose**.

## Most used commands

{% container(type="info") %}

**Verbosity**

Remember to increase verbosity of commands whenever you need to debug
or simply to have a more detailed output.

```sh
# levels are -> error | warn | info | debug | trace
-v
# all the way to
-vvvv
```

{% end %}

### create

Create a virtual machine from a template and user defined data.

```sh
# v vm create --template <template_name> --user-data <user_data_filepath>
v vm create --template xxs --user-data ~./user-data.toml
```

### ls (list)

List your VMs state and display associated information like
the number of allocated virtual **cpu** and the amount of **reserved ram**.

```sh
v vm ls
```

![vm_list](/images/v_vm_ls.png)

Increase the command verbosity to display reserved ips and disks size.

```sh
v vm ls -v
```

![vm_list](/images/v_vm_ls_v.png)

### start

Start a virtual machine by its id, uuid or name.

```sh
# v vm start --id <vm_id>
v vm start --id 1
# or
# v vm start --name <vm_name>
v vm start --name ichigo_kurosaki
```

You can also bulk start multiple VM by state.

```sh
# v vm start --state <vm_state>
v vm start --state not_created
```

{% container(type="info") %}
**Unstable feature**

You can start a VM interactively

```sh
# v vm start --id <vm_id>
v vm start --id 1 --attach
```

{% end %}

### delete

Delete a virtual machine.

Every machine resources are suppressed or deleted from disk.

```sh
v vm delete --id <vm_id>
```

{% container(type="danger") %}

**Soft deletion**

For now the deletion is the default Linux metadata deletion.
A disk shredding will be added in next releases for **increased privacy**.

{% end %}

### help

You can get more details and discover undocumented commands with:

```sh
v vm --help
```

## Ssh first access.

{% container(type="success") %}

**No network configuration needed.**

{% end %}

You can access your local machines through **ssh**, without any network configuration,
thanks to **ssh over unix socket (vsock)**.

```sh
ssh vm/<vm_name>
```

The pattern `vm/*` is recognized by your ssh client (thanks to systemd-ssh-proxy)
and tells it to connect to the VM via a socket exposed by Virshle
at `/var/lib/virshle/vm/<vm_uuid>/ch.vsock`.

## Update VM configuration.

For faster VM configuration build time,
build your generations locally with all the CPU and RAM you need
and send them to your VM.

{% sbs() %}

```sh
# with the default cli
nixos-rebuild switch \
 --flake ".#<flake_name>" \
 --target-host "vm/<vm_name>" \
 --use-remote-sudo
```

```sh
# with deploy-rs (magicRollback)
deploy ".#vps"
```

{% end %}
