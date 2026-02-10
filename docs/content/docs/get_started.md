+++
date = 2025-09-11
updated = 2026-02-11

weight = 40

title = "Run your first VM."

draft=false
+++

# Run your first VM.

## Prerequisite

### Get a default bootable disk...

You can create default bootable disks from the virshle repository flake.

```sh
nix build "github:pipelight/virshle/#vm_all_sizes" \
    --out-link ./result_vms \
    --show-trace
```

This command yields the following disks inside the `./result_vms` directory.

| name              | size   |
| ----------------- | ------ |
| nixos.xxs.efi.img | 20 GiB |
| nixos.xs.efi.img  | 50 GiB |
| nixos.s.efi.img   | 80 GiB |

You are free to move them around (or in the `/var/lib/virshle/cache` directory)
and use them inside your template definitions.

```sh
rsync --progress -azv ./result_vms/* /var/lib/virshle/cache
# or
cp ./result_vms/* /var/lib/virshle/cache
```

```toml
[[template.vm.disk]]
name = "os"
path = "/var/lib/virshle/cache/nixos.xxs.efi.img"
```

### ...or build a custom one.

You can create a default disk image (`.img`),
with your favourite configuration already built-in.

Simply use [nixos-generators](https://github.com/nix-community/nixos-generators).
And add the required dependencies to your flake.

```nix
# flake.nix
{
  description = "Base config for virshle custom disk image ";
  inputs = {
    nixos-generators = {
      url = "github:nix-community/nixos-generators";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    pipelight = {
      url = "github:pipelight/pipelight";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    virshle = {
      url = "github:pipelight/virshle";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };
  outputs = {
    self,
    nixpkgs,
  } @ inputs:
    vm_base = inputs.nixos-generators.nixosGenerate {
      inherit pkgs;
      inherit specialArgs;
      format = "raw-efi";
      modules = [
        inputs.virshle.nixosModules.nixos-generators
        # Set a custom disk size
        {virtualisation.diskSize = 10 * 1024;}

        ./my_configuration.nix
      ];
    };
}
```

Then build the disk image.

```sh
nix build ".#vm_base" \
    --out-link ./result_vms \
    --show-trace
```

And feel free to add it to Virshle cache directory for convenience.

```sh
rsync --progress -azv ./result_vms/* /var/lib/virshle/cache
# or
cp ./result_vms/* /var/lib/virshle/cache
```

### Make your configuration compatible.

For your VM to be `compatible with the hypervisor`, you need the
**nixos-generators** module
from the
[virshle](https://github.com/pipelight/virshle)
flake as a dependency to your configuration:

- Add the **virshle** and the **pipelight** flake to your flake inputs.

```nix
# flake.nix
{
  inputs =  {
    virshle = {
      url = "github:pipelight/virshle";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    pipelight.url = "github:pipelight/pipelight";
  };
}
```

- And import the **nixos-generators** module.

```nix
# vm.nix
{
  inputs,
  ...
}: {
  imports = [
    inputs.virshle.nixosModules.nixos-generators
  ];
}
```

Or, you can just start fresh from the **template flake**:

```sh
nix flake init \
    --template github:pipelight/virshle?ref=master
```

{% container(type="warning") %}

You'll face a substantial compilation time due to **pipelight**
not being precompiled in any official repository yet.

_Pipelight is used inside the machine for runtime configuration after boot._

{% end %}

{% container(type="danger") %}

**FHS Linux (Arch, Debian...)**

The creation of a custom raw-efi image on FHS Linux hasn't been automated and
is therefore beyond the scope of this documentation.

Instructions available in the [FHS_install](../unstable/fhs-installation) section.

{% end %}

## Create VM from template.

Virshle can create VMs by the use of templates.

You add some template definitions into your configuration file at `/etc/virshle/config.toml`.

A functional machine needs at least :

- A bootable OS disk (mandatory),
- Some cpu,
- Some ram,

See the template below that defines a small machine preset named `xxs`.

```toml
# /etc/virshle/config.toml
[[template.vm]]
name = "xxs"
vcpu = 1
vram = "1GiB"
[[template.vm.disk]]
name = "os"
path = "/var/lib/virshle/cache/nixos.xxs.efi.img"
```

Add an ssh key to your user to connect through **ssh**.

```toml
# ./user-data.toml
[[user]]
name = "anon"
[user.ssh]
authorized_keys = ["ssh-ed25519 AAAAC3N..."]
```

```sh
# v vm create --template <template_name> --user-data <user_data_filepath>
v vm create --template xxs --user-data ~./user-data.toml
```

Check that the machine has been created.

```sh
v vm ls # -v
```

## Start the VM!

You are all set.

Start your new virtual machine and connect through `ssh` with the
key you provided in your `user-data.toml`.

```sh
# v vm start --id <vm_id>
v vm start --id 1
```
