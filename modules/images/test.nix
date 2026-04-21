{
  lib,
  pkgs,
  ...
}: {
  imports = [
    ./base
  ];
  ###################################
  # Default user
  users.users."anon" = {
    isNormalUser = true;
    # Set default password for testing vm
    initialPassword = "anon";
  };
  users.groups = {
    wheel.members = ["anon"];
  };

  ###################################
  # Admin users
  # loosen security for fast sudoing
  security.sudo.extraRules = [
    {
      groups = ["wheel"];
      commands = [
        {
          command = "ALL";
          options = ["NOPASSWD"];
        }
      ];
    }
  ];
  environment.systemPackages = [
    pkgs.iproute2
  ];
}
