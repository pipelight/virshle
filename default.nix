{
  pkgs ? import <nixpkgs> {},
  stdenv,
}:
stdenv.mkDerivation {
  __noChroot = true;

  name = "virshle";
  version = "0.1.0";
  src = ./.;

  buildInputs = with pkgs; [
    deno
  ];
  buildPhase = ''
    export HOME=$(pwd)
    deno compile --output virshle ./mod.ts
  '';
  installPhase = ''
    mkdir -p $out/bin
    install -t $out/bin virshle
  '';
}
