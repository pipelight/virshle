{pkgs ? import <nixpkgs> {}}:
pkgs.mkShell {
  buildInputs = with pkgs.buildPackages; [
    openssl
    pkg-config
    libvirt
    libvirt-glib
    rust-bin.nightly.latest.default
    llvmPackages_latest.bintools
  ];

  RUSTFLAGS = builtins.map (a: ''-L ${a}/lib'') [
    # add libraries here (e.g. pkgs.libvmi)
    pkgs.llvmPackages_latest.bintools
    pkgs.libvirt-glib
  ];
  RUSTC_VERSION = pkgs.lib.readFile ./rust-toolchain.toml;
}
