{
  pkgs ? import <nixpkgs> {},
  stdenv,
}:
stdenv.mkDerivation rec {
  __noChroot = true;

  name = "virshle";
  version = "0.1.4";
  src = ./.;

  nativeBuildInputs = with pkgs; [
    deno
  ];

  installPhase = ''
    mkdir -p $out/bin

    echo -e "#!/usr/bin/env bash
    deno run -A $out/bin/mod.ts \$@" > ${name}

    cp -r ./* $out/bin/
    install -t $out/bin ${name}
  '';
}
