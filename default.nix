{
  pkgs ? import <nixpkgs> {},
  stdenv,
}:
stdenv.mkDerivation {
  __noChroot = true;

  name = "virshle";
  version = "0.1.1";
  src = ./.;

  buildInputs = with pkgs; [
    deno
  ];
  buildPhase = ''
    export HOME=$(pwd)
    deno compile -A --output virshle ./mod.ts
  '';
  installPhase = ''
    export HOME=$(pwd)
    mkdir -p $out/bin
    install -t $out/bin virshle
  '';
}
