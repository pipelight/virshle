{pkgs ? import <nixpkgs> {}}:
pkgs.mkShell {
  buildInputs = with pkgs.buildPackages; [
    openssl
    pkg-config
    libvirt
    libvirt-glib
  ];

  RUSTC_VERSION = pkgs.lib.readFile ./rust-toolchain.toml;
}
