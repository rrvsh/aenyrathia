{ config, ... }:
let
  inherit (config.flake) dependencies;
in
{
  perSystem =
    { pkgs, ... }:
    let
      inherit (pkgs.stdenv) mkDerivation;
    in
    rec {
      packages.default = packages.export-md;
      packages.export-md = mkDerivation {
        name = "aenyrathia-writings-md";
        src = ../writings;
        buildInputs = dependencies pkgs;
        dontBuild = true; # just copy files
        installPhase = ''
          mkdir -p $out
          cp -r $src/* $out
        '';
      };
    };
}
