{
  pkgs ? import <nixpkgs> {},
  stdenv,
}:
stdenv.mkDerivation rec {
  __noChroot = true;

  name = "virshle";
  version = "0.1.3";
  src = ./.;

  nativeBuildInputs = with pkgs; [
    deno
  ];

  installPhase = ''
    mkdir -p $out/bin

    echo -e "#!/usr/bin/env bash
    deno run -A $out/bin/mod.ts" > ${name}

    install -t $out/bin mod.ts
    install -t $out/bin ${name}
  '';
}
