{lib, ...}: {
  ###################################
  # Default user
  users.users."anon" = {
    isNormalUser = true;
  };
}
