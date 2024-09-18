{pkgs ? import <nixpkgs> {}}:
pkgs.mkShell {
  buildInputs = with pkgs.buildPackages; [
    openssl
    pkg-config
    (rust-bin.fromRustupToolchainFile ./rust-toolchain.toml)
    libvirt
    libvirt-glib
  ];
}
