{
  description = "Virshle - Manage VM with TOML";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
    flake-parts.url = "github:hercules-ci/flake-parts";
    pipelight.url = "github:pipelight/pipelight";
    nixos-generators = {
      url = "github:nix-community/nixos-generators";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = {
    self,
    nixpkgs,
    rust-overlay,
    flake-utils,
    flake-parts,
    ...
  } @ inputs: let
    virshle_lib =
      {}
      // (import ./modules/lib/network {
        inherit (nixpkgs) lib;
      });
    specialArgs = {
      inherit inputs;
      inherit virshle_lib;
    };
  in
    flake-parts.lib.mkFlake {
      inherit inputs;
    } {
      flake = rec {
        nixosModules = rec {
          default = virshle;
          virshle = ./modules/default.nix;
          nixos-generators = ./modules/nixos-generators/default_vm;
          nixos-generators-test-vm = ./modules/nixos-generators/test_vm;
        };
        nixosConfigurations = {
          vm_disko_base = nixpkgs.lib.nixosSystem {
            inherit specialArgs;
            modules = [
              ./modules/nixos-generators/default_vm
            ];
          };
          vm_disko_test = nixpkgs.lib.nixosSystem {
            inherit specialArgs;
            modules = [
              ./modules/nixos-generators/test_vm
            ];
          };
        };
        defaultTemplate = templates.default;
        templates = {
          default = {
            path = ./templates/default;
            description = ''
              A minimal nixos configuration flake for virshle VMs.
            '';
          };
        };
        ## Unit tests
        tests = import ./modules/lib/network/test.nix {
          inherit virshle_lib;
          inherit (nixpkgs) lib;
        };
      };
      systems = flake-utils.lib.allSystems;
      perSystem = {
        config,
        self,
        pkgs,
        system,
        ...
      }: let
        # Fix big disk image creation stuck at 'crng init done'.
        #
        # https://github.com/nix-community/nixos-generators/issues/443#issuecomment-3697547318
        #
        # Overlay to increase LKL (Linux Kernel Library) memory from 100M to 1024M
        # The cptofs tool uses LKL to run a kernel as a library for filesystem operations
        # during disk image creation. The default 100M causes OOM for large disk images.
        lklMemoryOverlay = final: prev: {
          lkl = prev.lkl.overrideAttrs (old: {
            postPatch =
              (old.postPatch or "")
              + ''
                # Increase LKL kernel memory for large disk image builds
                substituteInPlace tools/lkl/cptofs.c \
                  --replace-fail 'lkl_start_kernel("mem=100M")' 'lkl_start_kernel("mem=1024M")'
              '';
          });
        };
        overlays = [
          (import rust-overlay)
          # lklMemoryOverlay
        ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
      in rec {
        devShells.default = pkgs.callPackage ./shell.nix {};
        packages = {
          default = pkgs.callPackage ./package.nix {};

          # vm_disko = nixosConfigurations.installer.config.system.build.isoImage;
          vm_base = inputs.nixos-generators.nixosGenerate {
            inherit pkgs;
            inherit specialArgs;
            format = "raw-efi";
            modules = [
              ./modules/nixos-generators/default_vm
            ];
          };
          # Output all vm disk sizes:
          # - nixos.xxs.efi.raw
          # - nixos.xs.efi.raw
          # - nixos.s.efi.raw
          vm_all_sizes = inputs.nixos-generators.nixosGenerate {
            inherit pkgs;
            inherit specialArgs;
            format = "raw-efi";
            modules = [
              ./modules/nixos-generators/make-disk-images.nix
              ./modules/nixos-generators/default_vm
              {
                nix.settings = {
                  trusted-users = ["root" "@wheel"];
                };
              }
            ];
          };
          # Output vm disk for easy testing (with default passwords).
          # - nixos.test.xxs.iso
          vm_test = inputs.nixos-generators.nixosGenerate {
            inherit pkgs;
            inherit specialArgs;
            format = "raw-efi";
            modules = [
              ./modules/nixos-generators/make-test-disk-images.nix
              ./modules/nixos-generators/test_vm
            ];
          };
        };
      };
    };
}
