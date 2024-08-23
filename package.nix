{
  pkgs ? import <nixpkgs> {},
  lib,
  ...
}:
pkgs.rustPlatform.buildRustPackage rec {
  pname = "virshle";
  version = "0.3.0";

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
    libvirt
    libvirt-glib
    rust-bin.nightly.latest.default
    llvmPackages_latest.bintools
  ];

  postInstall = with lib; ''
    installShellCompletion --cmd ${pname}\
      --bash ./autocompletion/${pname}.bash \
      --fish ./autocompletion/${pname}.fish \
      --zsh  ./autocompletion/_${pname}
  '';
}
