+++
date = 2025-09-11
updated = 2026-02-10

weight = 10

title = "Custom disk"

description = """

"""

draft=false
+++

# Custom disk image.

## With Nixos.

You can create a default disk image (`.img`),
with your favourite configuration already built-in.

Simply use [nixos-generators](https://github.com/nix-community/nixos-generators)

```nix
#flake.nix
{
  description = "Virshle virtual machine base config.";
  inputs = {
    nixos-generators = {
      url = "github:nix-community/nixos-generators";
      inputs.nixpkgs.follows = "nixpkgs";
      # inputs.nixpkgs.follows = "nixpkgs-unstable";
    };
    pipelight = {
      url = "github:pipelight/pipelight?ref=dev";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };
  outputs = {
    self,
    nixpkgs,
  } @ inputs:
    vm = inputs.nixos-generators.nixosGenerate {
      format = "raw-efi";
      inherit system;
      inherit specialArgs;
      modules = [
        {virtualisation.diskSize = 10 * 1024;}
          ./vm.nix
      ];
    };
}
```

Enable `pipelight-init` in the vm configuration.

```nix
crocuda = {
  virtualization = {
    pipelight-init.enable = true;
  };
};
```

This will allow virshle to load additional config to the vm
via a mounted disk.
See it as a [cloud-init](https://cloudinit.readthedocs.io/en/latest/)
replacement.

Finally, add the init disk to the vm configuration.

```nix
#vm.nix
fileSystems."/pipelight-init" = {
  device = "/dev/disk/by-label/INIT";
  fsType = "vfat";
  options = [
    "nofail"
  ];
};
```

## Without Nixos.

Good luck.
