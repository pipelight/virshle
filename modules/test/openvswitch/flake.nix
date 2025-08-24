{
  description = "A flake that uses nixos-tidy home-merger";
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-24.11";
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

          # Install openvswitch custom package

          ({
            lib,
            config,
            inputs,
            pkgs,
            modulesPath,
            ...
          }: let
            openvswitch-afxdp = pkgs.callPackage ../../openvswitch/package.nix {inherit modulesPath;};
          in {
            environment.systemPackages = [
              openvswitch-afxdp
            ];
          })
        ];
      };
    };
  };
}
