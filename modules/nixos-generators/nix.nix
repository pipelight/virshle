{
  lib,
  pkgs,
  ...
}: {
  ##########################
  ## Lix
  # nix.package = pkgs.lixPackageSets.stable.lix;
  ## Nix
  # Enable Flakes
  nix.settings = {
    experimental-features = ["nix-command" "flakes"];
    # auto-optimise-store = true;
    # sandbox = "relaxed";
  };
  system.stateVersion = "25.11";
  system.autoUpgrade.channel = "https://nixos.org/channels/nixos-25.11/";

  ##########################
  # Nix substituters
  # and Binary caches
  nix.settings = {
    trusted-users = ["root" "@wheel"];
    substituters = [
      "https://cache.nixos.org/"
      "https://nix-community.cachix.org"
    ];
    trusted-public-keys = [
      "nix-community.cachix.org-1:mB9FSh9qf2dCimDSUo8Zy7bkq5CX+/rkCWyvRCYg3Fs="
    ];
  };
}
