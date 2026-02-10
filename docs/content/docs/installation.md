+++
date = 2025-09-11
updated = 2026-02-13

weight = 10

title = "Installation (NixOs)"

description = """

"""

draft=false
+++

# Installation

## NixOs (with flakes)

Enable the module by adding the repository url to your flake input.

```nix
# flake.nix
inputs = {
  virshle = {
      url = "github:pipelight/virshle";
  };
};
```

Add the module to your host configuration.

```nix
nixosConfiguration = {
    default = pkgs.lib.nixosSystem {
        modules = [
            inputs.virshle.nixosModules.default
        ];
    };
}
```

Enable the service.

```nix
# default.nix
services.virshle = {
    enable = true;
    logLevel = "info"; # error | warn | info | debug | trace
    user = "anon"; # The user to run the node as (default to root).
};
```

{% container(type="warning") %}

You'll face a substantial compilation time due to **virshle**
not being precompiled in any official repository yet.

{% end %}

You may want to create an alias to ease command line usage.

```sh
alias v='virshle'
```

## Run your node.

Enabling the module is enough to have a node up and running
as a systemd-unit (_virshle.service_).

Then get the node health check with the following command:

```sh
v node ls # (-vvv)
# or
v node ls --all # (-vvv)
```

Which should output:

![node_list](/images/v_node_ls_vvv_default.png)

You can troubleshoot the node by either:

- Increasing the daemon verbosity
  and skimming through logs.

  ```nix
  logLevel = "info"; # error | warn | info | debug | trace
  ```

  ```sh
  sudo systemctl status virshle
  # or
  sudo journalctl -xeu virshle.service
  ```

- Or stop the daemon and run it interactively in a terminal
  with a higher verbosity.

  ```sh
  sudo systemctl stop virshle
  v node serve -vvvv
  ```

{% container(type="info") %}

A node is an instance of Virshle that can communicate
with other peer instances.

The notion of `node` has been introduced very early in development
and is greatly inspired by [radicle](https://radicle.xyz/) work on the subject.
The aim is to further create a peer-to-peer hosting network (decentralized hosting!).

{% end %}

## Network configuration

{% container(type="tip") %}

Mandatory for external connectivity

{% end %}

{% container(type="danger") %}

The module to ease host network configuration is still a work in progress.

{% end %}

Virshle is able to add seamless connectivity between host, VMs and the outside
from a single physical port.

However, the module is not ready yet,
so you'll need a heavy ass configuration to enable this (500 lines).

For now, you can fine tune your host network configuration based on the following template:
[`/virshle/modules/networking.nix`](https://github.com/pipelight/virshle/modules/network.nix).

## Set a custom storage (Optional).

You can store VMs resources in another device like an encrypted RAID.
Just symlink `/var/lib/virshle` to the desired path, and set required permissions.

```nix
systemd.tmpfiles.rules = [
  "L+ /var/lib/virshle - - - - /run/media/RAID/storage/virshle"
  "Z '/run/media/RAID/storage/virshle' 2774 ${config.services.virshle.user} users - -"
];
```
