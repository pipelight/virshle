{
  description = "Virshle - a virsh wrapper for markup language support";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = inputs:
    with inputs;
      flake-utils.lib.eachDefaultSystem (
        system: let
          pkgs = nixpkgs.legacyPackages.${system};
        in rec {
          packages.default = pkgs.callPackage ./default.nix {};
          devShells.default = pkgs.callPackage ./shell.nix {};
        }
      );
}
