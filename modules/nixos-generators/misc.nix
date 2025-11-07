{lib, ...}: {
  ###################################
  # Default user
  users.users."anon" = {
    isNormalUser = true;
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
}
