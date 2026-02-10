+++
date = 2025-02-10
updated = 2026-02-10

title = "Landing"
description = ""

render = false
+++

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

**Template centric**

Define a standard VM template that you can twist multiple copy of.

```toml
#########################
# Templates:
# vm standard sizes with decents presets.

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
