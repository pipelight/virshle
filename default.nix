{
  pkgs ? import <nixpkgs> {},
  stdenv,
}:
stdenv.mkDerivation {
  name = "virshle";
  version = "0.1.0";
  src = ./.;
  buildInputs = [
  deno
  ];
}
