{pkgs ? import <nixpkgs> {}}:
pkgs.mkShell {
  buildInputs = with pkgs.buildPackages; [
    openssl
    pkg-config

    # rust vmm uses latest stable and oxalica tend to lag behind.break
    # so we temporary force use of beta.

    (rust-bin.fromRustupToolchainFile ./rust-toolchain.toml)
    rust-analyzer
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
