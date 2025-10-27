{ config, ... }:
let
  inherit (config.flake) dependencies devDependencies;
in
{
  perSystem =
    { pkgs, ... }:
    let
      inherit (pkgs) mkShell;
    in
    {
      devShells.default = mkShell {
        buildInputs = (dependencies pkgs) ++ (devDependencies pkgs);
      };
    };
}
