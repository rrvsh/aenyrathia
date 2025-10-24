{
  inputs.nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
  outputs = inputs: let
    pkgs = inputs.nixpkgs.legacyPackages."aarch64-darwin";
  in {
    devShells."aarch64-darwin".default = pkgs.mkShell {
      buildInputs = with pkgs; [
        python3
        python313Packages.pyphen
        just
      ];
    };
  };
}
