+++
date = 2025-09-11
updated = 2026-02-10

weight = 10

title = "Installation"

description = """

"""

draft=false
+++

# Installation

## NixOs (with flakes)

Enable the module by adding the repository url to your configuration.

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
  user = "anon"; #default to root.
};
```

{% container(type="warning") %}

You'll face a substential compilation time due to **virshle** and **pipelight**
not being precompiled in any official repository yet.

{% end %}

You may want to create an alias to ease command line usage.

```sh
alias v='virshle'
```

## Set a custom storage.

You can store VMs resources in another device like an encrypted RAID.
Just symlink `/var/lib/virshle` to the desired path, and set required permissions.

```nix
systemd.tmpfiles.rules = [
  "L+ /var/lib/virshle - - - - /run/media/RAID/storage/virshle"
  "Z '/run/media/RAID/storage/virshle' 2774 ${config.services.virshle.user} users - -"
];
```

## Tweak the network.

Virshle is able to add seamless connectivity between host, VMs and the outside
from a single physical port.
But you'll need a heavy ass configuration to enable this (500 lines).

I deliberately chose not to expose a module for this

I like to create a per hardware `networking.nix` file that contains
firewall, dhcp, and routing rules.

For fine vm network control, you can add a host network configuration like the following
[`modules/networking.nix`](https://github.com/pipelight/virshle/modules/config.nix).
