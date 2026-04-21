{
  description = "Virshle - Manage VM with TOML";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
    # flake-parts.url = "github:hercules-ci/flake-parts";
    disko = {
      url = "github:nix-community/disko/latest";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    pipelight.url = "github:pipelight/pipelight";
    # nixos-generators = {
    #   url = "github:nix-community/nixos-generators";
    #   inputs.nixpkgs.follows = "nixpkgs";
    # };
  };

  outputs = {
    self,
    nixpkgs,
    rust-overlay,
    flake-utils,
    # flake-parts,
    disko,
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
    {
      nixosModules = rec {
        default = virshle;
        virshle = ./modules/default.nix;
        vm = ./modules/images/default.nix;
        vm-test = ./modules/images/test.nix;
      };
    }
    // flake-utils.lib.eachDefaultSystem (system: let
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
      nixosConfigurations = let
        make_vm_configuration = size: swap:
          nixpkgs.lib.nixosSystem {
            inherit pkgs;
            inherit specialArgs;
            modules = [
              ./modules/images/default.nix
              {
                disko.devices.disk.main.imageSize = size;
                swapDevices = [
                  {
                    device = "/var/lib/swapfile";
                    size = swap * 1024;
                  }
                ];
              }
            ];
          };
        make_vm_test_configuration = size: swap:
          nixpkgs.lib.nixosSystem {
            inherit pkgs;
            inherit specialArgs;
            modules = [
              ./modules/images/test.nix
              {
                disko.devices.disk.main.imageSize = size;
                swapDevices = [
                  {
                    device = "/var/lib/swapfile";
                    size = swap * 1024;
                  }
                ];
              }
            ];
          };
      in {
        xxs-test = make_vm_test_configuration "20G" 1;
        xxs = make_vm_configuration "20G" 1;
        xs = make_vm_configuration "50G" 1;
        s = make_vm_configuration "80G" 2;
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

      devShells.default = pkgs.callPackage ./shell.nix {};
      packages = rec {
        default = pkgs.callPackage ./package.nix {};

        ###################################
        ## Btrfs VMs
        # Output all vm disk sizes:
        # - nixos.xxs.efi.raw 20GiB
        # - nixos.xs.efi.raw 50GiB
        # - nixos.s.efi.raw 80GiB
        vm-xxs-test = nixosConfigurations.xxs-test.config.system.build.diskoImages;
        vm-xxs = nixosConfigurations.xxs.config.system.build.diskoImages;
        vm-xs = nixosConfigurations.xs.config.system.build.diskoImages;
        vm-s = nixosConfigurations.s.config.system.build.diskoImages;

        ## Build all VMs images.
        vm = let
          ## Get Vm image store path.
          vm-derivation-store-path = name:
            nixosConfigurations.${name}.config.system.build.diskoImages;
          ## Create commands to copy derivations result to same directory.
          copy-images-to-output = with pkgs;
            names:
            # Join commands
              lib.strings.concatStringsSep "\n"
              # Yield command list/array
              (lib.lists.forEach names (
                name: "cp ${vm-derivation-store-path name}/* $out/nixos.${name}.efi.img"
              ));
        in
          pkgs.stdenv.mkDerivation {
            name = "vms";
            installPhase = ''
              mkdir $out
              ${copy-images-to-output ["xxs-test" "xxs" "xs" "s"]}
            '';
            fixupPhase = "";
            buildPhase = "";
            outputs = ["out"];
            src = ./.;
          };
      };
    });
}
