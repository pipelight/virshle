{lib, ...}:
with lib; let
  _rand_uuid = builtins.readFile /proc/sys/kernel/random/uuid;
  random_mac = str_to_mac _rand_uuid;

  ## Functions
  # Generate a 128bits hash from a secret
  _str_to_hash = string:
    builtins.substring 0 32 (builtins.hashString "sha256" string);

  # Add a ":" each n(step) characters
  _hash_to_address = string: step:
    lib.concatStringsSep ":" (
      lib.forEach (lib.range 0 (lib.stringLength string / step - 1)) (
        i: builtins.substring (i * step) step string
      )
    );
  # Generate a mac address from a sting
  str_to_mac = string: let
    hash = builtins.substring 0 12 (_str_to_hash string);
    vec_hash = stringToCharacters hash;

    sanitized_hash = concatStrings (
      imap0 (i: v:
        if i == 1
        then "2" # SLAP quadrant (AAI)
        else v) (stringToCharacters hash)
    );
    step = 2;
    mac = _hash_to_address sanitized_hash step;
  in
    mac;

  # Generate an ipv6 from a string
  str_to_ipv6 = string: let
    hash = _str_to_hash string;
    step = 4;
    ipv6 = _hash_to_address hash step;
  in
    ipv6;

  # Generate an ipv6 iid from a string
  str_to_iid = string: let
    # Take only the iid part of the ipv6
    hash = builtins.substring 16 16 (_str_to_hash string);
    step = 4;
    iid = _hash_to_address hash step;
  in
    iid;
in rec {
  inherit _str_to_hash;

  inherit random_mac;
  inherit str_to_mac;

  inherit str_to_ipv6;
  inherit str_to_iid;
}
