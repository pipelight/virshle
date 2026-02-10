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
  } @ inputs:
    flake-parts.lib.mkFlake {
      inherit inputs;
    } {
      flake = rec {
        nixosModules = rec {
          default = virshle;
          virshle = ./modules/default.nix;
          nixos-generators = ./modules/nixos-generators;
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
      };
      systems = flake-utils.lib.allSystems;
      perSystem = {
        config,
        self,
        pkgs,
        system,
        ...
      }: let
        overlays = [(import rust-overlay)];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        virshle_lib =
          {}
          // (import ./lib/network {
            inherit (nixpkgs) lib;
          });
        specialArgs = {
          inherit inputs;
          inherit virshle_lib;
        };
      in {
        devShells.default = pkgs.callPackage ./shell.nix {};
        packages = {
          default = pkgs.callPackage ./package.nix {};
          vm_base = inputs.nixos-generators.nixosGenerate {
            inherit pkgs;
            inherit specialArgs;
            format = "raw-efi";
            modules = [
              ./modules/nixos-generators
            ];
          };
          vm_all_sizes = inputs.nixos-generators.nixosGenerate {
            inherit pkgs;
            inherit specialArgs;
            format = "raw-efi";
            modules = [
              ./modules/make-disk-images.nix
              ./modules/nixos-generators
            ];
          };
        };
        ## Unit tests
        tests = import ./lib/network/test.nix {
          inherit virshle_lib;
          inherit (nixpkgs) lib;
        };
      };
    };
}
