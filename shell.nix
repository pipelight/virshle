{pkgs ? import <nixpkgs> {}}:
pkgs.mkShell {
  buildInputs = with pkgs.buildPackages; [
    openssl
    pkg-config
    (rust-bin.fromRustupToolchainFile ./rust-toolchain.toml)
    libvirt
    libvirt-glib
  ];
  # SeaOrm Sqlite database
  DATABASE_URL = "sqlite:////var/lib/virshle/virshle.sqlite?mode=rwc";
  DBEE_CONNECTIONS = "[
    {
      \"name\": \"virshle_db\",
      \"type\": \"sqlite\",
      \"url\": \"/var/lib/virshle/virshle.sqlite?mode=rwc\"
    }
  ]";
}
