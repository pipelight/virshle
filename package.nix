{
  pkgs ? import <nixpkgs> {},
  lib,
  ...
}:
pkgs.rustPlatform.buildRustPackage rec {
  pname = "virshle";
  version = (builtins.fromTOML (lib.readFile ./${pname}/Cargo.toml)).package.version;

  src = ./.;
  cargoLock = {
    lockFile = ./Cargo.lock;
  };

  # disable tests
  checkType = "debug";
  doCheck = false;

  nativeBuildInputs = with pkgs; [
    installShellFiles
    pkg-config
  ];
  buildInputs = with pkgs; [
    openssl
    pkg-config

    # rust vmm uses latest stable and oxalica tend to lag behind.break
    # so we temporary force use of beta.
    (rust-bin.fromRustupToolchainFile ./rust-toolchain.toml)
  ];

  postInstall = with lib; ''
    installShellCompletion --cmd ${pname}\
      --bash ./autocompletion/${pname}.bash \
      --fish ./autocompletion/${pname}.fish \
      --zsh  ./autocompletion/_${pname}
  '';
}
