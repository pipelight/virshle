{
  description = "A minimal nixos configuration flake for virshle VMs";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    virshle = {
      url = "github:pipelight/virshle";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    pipelight.url = "github:pipelight/pipelight";
  };

  outputs = {
    self,
    nixpkgs,
    ...
  } @ inputs: let
    system = "x86_64-linux";
    pkgs = nixpkgs;
  in rec {
    nixosConfigurations = {
      default = pkgs.lib.nixosSystem {
        specialArgs = {inherit inputs;};
        modules = [
          ../commons/configuration.nix
          ../commons/hardware-configuration.nix

          inputs.virshle.nixosModules.nixos-generators
        ];
      };
    };
    packages."${system}" = {
      default = nixosConfigurations.default.config.system.build.toplevel;
    };
  };
}
