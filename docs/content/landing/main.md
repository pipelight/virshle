+++
date = 2025-02-10
updated = 2026-02-13

title = "Landing"
description = ""

render = false
+++

{% container(type="null") %}
**tl;dr**

Virshle is **toml** configuration files and fancy cli
on top of a modern hypervisor.

{% end %}

{% container(type="danger") %}

**Alpha releases.**

Although virshle is stable enough to be the engine propelling [Crocuda_vps](https://crocuda.com),
You shouldn't use it in production as you may encounter unnoticed breaking changes.

{% end %}

{% sbs() %}

{% container(type="info") %}

**Built for Nixos**
{% icon(type="nixos pin") %}\_{% end %}

Create machines based on your personal nixos configurations.

{% end %}

{% container(type="info") %}

~$ **ssh://addicts**

Access your local virtual machine via a shared ssh socket.

```sh
ssh vm/<name>`
```

{% end %}

{% end %}

{% container(type="info") %}

~$ **Command line first**█

List your VMs.
And get details on resources, storage and network.

```sh
v vm ls -v`
```

![cli_preview](/images/v_vm_ls_v.png)

{% end %}

{% container(type="info") %}

**\[\[ Template centric \]\]**

In the vein of NixOs,
you get a declarative/reproducible approach of VM creation.

A **single template** can spin up **multiple identical** VMs.

{% sbs() %}

{% container(type="null") %}

Define custom VM templates...

```toml
# /etc/virshle/config.toml

#########################
## Templates:
# Vm standard sizes with decents presets.
[[template.vm]]
name = "xxs"
vcpu = 1
vram = "1GiB"
[[template.vm.disk]]
name = "os"
path = "/var/lib/virshle/cache/nixos.xxs.efi.img"
[[template.vm.net]]
name = "main"
[template.vm.net.type.tap]

```

{% end %}

{% container(type="null") %}

...add some user defined data...

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
{% end %}

{% container(type="null") %}

...and twist multiple copy of a VM.

```sh
v vm create --template xxs --user-data ~./user-data.toml
```

{% end %}
{% end %}
