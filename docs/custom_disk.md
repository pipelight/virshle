# Custom disk image.

## With Nixos.

You can create a default vm disk,
with your favorite configuration already builtin.

Simply use [nixos-generators](https://github.com/nix-community/nixos-generators)

```nix
#flake.nix
vm = inputs.nixos-generators.nixosGenerate {
  format = "raw-efi";
  inherit system;
  inherit specialArgs;
  modules = [
    {virtualisation.diskSize = 10 * 1024;}
      ./vm.nix
  ];
};
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
