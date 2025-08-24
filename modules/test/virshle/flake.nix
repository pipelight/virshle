{
  description = "A flake to test virshle";
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-25.05";
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

          ../../default.nix

          ({
            lib,
            config,
            inputs,
            pkgs,
            ...
          }: {
            services.virshle = {
              enable = true;
              afxdp.enable = true;
              logLevel = "debug";
              user = "anon";
            };
          })
        ];
      };
    };
  };
}
