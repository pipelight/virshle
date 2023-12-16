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
  buildPhase = ''
    deno compile --output virshle ./mod.ts
  '';
  installPhase = ''
    mkdir -p $out/bin
    install -t $out/bin virshle
  '';
}
