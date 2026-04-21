<span>
<h1>
<img width="125px" alt="virshle_logo" src="https://github.com/pipelight/virshle/blob/master/docs/static/images/virshle.png"/>
<p>Virshle - A cli for multiple relentless virtual machines.</p>

</h1>
</span>

> [!DANGER]
> **Alpha releases.**
>
> Although Virshle is stable enough to be the engine propelling [Crocuda_vps](https://crocuda.com),
> You shouldn't use it in production as you may encounter unnoticed breaking changes.

Virshle is a single cli to manage multiple virtual machines.

It works on top of:

- [cloud-hypervisor](https://github.com/cloud-hypervisor/cloud-hypervisor)
  and
  [linux-kvm](https://linux-kvm.org/page/Main_Page)
  for machines virtualization,
- [pipelight-init](https://github.com/pipelight/pipelight)
  for fast vm provisioning on boot,
- [openvswitch](https://github.com/openvswitch/ovs)
  for network configuration.

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

## Usage

In the vein of NixOs,
you get a declarative/reproducible approach of VM creation.

A **single template** can spin up **multiple identical** VMs.

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

...and twist multiple copy of a VM.

```sh
v vm create --template xxs --user-data ~./user-data.toml
```

Check out the `docs` directory or on the documentation website at
[virshle.crocuda.com](https://virshle.crocuda.com).

# Developers

## Fancy tests.

Set the `CARGO_TEST_TRACING_LEVEL` environment variable
to run tests and print pretty logs when needed.

```sh
CARGO_TEST_TRACING_LEVEL='error' cargo test
```
