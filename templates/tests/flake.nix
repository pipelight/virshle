{
  description = "A flake that uses virshle module";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    virshle = {
      url = "github:pipelight/virshle?ref=dev";
      # url = "path:../../";
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
      # Default module
      default = pkgs.lib.nixosSystem {
        specialArgs = {inherit inputs;};
        modules = [
          ../commons/configuration.nix
          ../commons/hardware-configuration.nix

          inputs.virshle.nixosModules.default

          ###################################
          # You may move this module into its own file.
          ({
            lib,
            inpus,
            config,
            ...
          }: {
            services.virshle = {
              enable = true;
              dhcp.defaultConfig = true;
            };
          })
          ###################################
        ];
      };
    };
    packages."${system}" = {
      default = nixosConfigurations.default.config.system.build.toplevel;
    };
  };
}
