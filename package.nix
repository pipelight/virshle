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
    # outputHashes = {
    #   "tappers-0.4.2" = "sha256-kx/gLngL7+fH5JmJTVTGawyNdRde59dbFdrzermy/CE=";
    # };
  };

  # disable tests
  checkType = "debug";
  doCheck = false;

  nativeBuildInputs = with pkgs; [
    installShellFiles
    pkg-config

    llvmPackages.clang
    clang
  ];
  buildInputs = with pkgs; [
    openssl
    pkg-config

    (rust-bin.fromRustupToolchainFile ./rust-toolchain.toml)
  ];

  LIBCLANG_PATH = lib.makeLibraryPath [pkgs.llvmPackages.libclang.lib];
  # LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
  # LIBCLANG_PATH = lib.getLib pkgs.llvmPackages.libclang.lib;

  postInstall = ''
    installShellCompletion --cmd ${pname} \
      --bash ./autocompletion/${pname}.bash \
      --fish ./autocompletion/${pname}.fish \
      --zsh  ./autocompletion/_${pname}
  '';
}
