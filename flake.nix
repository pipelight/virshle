{
  description = "Virshle - Manage VM with TOML";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
    flake-parts.url = "github:hercules-ci/flake-parts";
    pipelight.url = "github:pipelight/pipelight";
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
      flake = {
        nixosModules = rec {
          default = virshle;
          virshle = ./modules/default.nix;
          nixos-generator = ./modules/nixos-generator;
        };
      };
      systems =
        flake-utils.lib.allSystems;
      perSystem = {
        config,
        self,
        inputs,
        pkgs,
        system,
        ...
      }: let
        overlays = [(import rust-overlay)];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
      in {
        devShells.default = pkgs.callPackage ./shell.nix {};
        packages = {
          default = pkgs.callPackage ./package.nix {};
          disk-images = rec {
            default = xxs;
            xxs = inputs.nixos-generators.nixosGenerate {
              format = "raw-efi";
              modules = [
                inputs.virshle.nixosModule.nixos-generators
                {virtualisation.diskSize = 20 * 1024;}
              ];
            };
            xs = inputs.nixos-generators.nixosGenerate {
              format = "raw-efi";
              modules = [
                inputs.virshle.nixosModule.nixos-generators
                {virtualisation.diskSize = 50 * 1024;}
              ];
            };
            s = inputs.nixos-generators.nixosGenerate {
              format = "raw-efi";
              modules = [
                inputs.virshle.nixosModule.nixos-generators
                {virtualisation.diskSize = 80 * 1024;}
              ];
            };
          };
        };
      };
    };
}
