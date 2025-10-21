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
    system = "x86_64-linux";
    overlays = [(import rust-overlay)];
    pkgs = import nixpkgs {
      inherit system overlays;
    };
  in rec {
    nixosModules = rec {
      default = virshle;
      virshle = ./modules/default.nix;
      nixos-generators = ./modules/nixos-generators;
    };
    devShells.${system}.default = pkgs.callPackage ./shell.nix {};
    packages.${system} = {
      default = pkgs.callPackage ./package.nix {};
      vm_xxs = inputs.nixos-generators.nixosGenerate {
        inherit pkgs;
        format = "raw-efi";
        modules = [
          ./modules/nixos-generators
          inputs.pipelight.nixosModules.pipelight-init
          {
            virtualisation.diskSize = 20 * 1024;
            services.pipelight-init.enable = true;
          }
        ];
      };
      vm_xs = inputs.nixos-generators.nixosGenerate {
        inherit pkgs;
        format = "raw-efi";
        modules = [
          ./modules/nixos-generators
          inputs.pipelight.nixosModules.pipelight-init
          {
            virtualisation.diskSize = 50 * 1024;
            services.pipelight-init.enable = true;
          }
        ];
      };
      vm_s = inputs.nixos-generators.nixosGenerate {
        inherit pkgs;
        format = "raw-efi";
        modules = [
          ./modules/nixos-generators
          inputs.pipelight.nixosModules.pipelight-init
          {
            virtualisation.diskSize = 80 * 1024;
            services.pipelight-init.enable = true;
          }
        ];
      };
    };
  };
}
