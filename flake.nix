{
  outputs =
    inputs:
    inputs.flake-parts.lib.mkFlake { inherit inputs; } (
      (inputs.import-tree ./nix)
      // {
        systems = import inputs.systems;
        flake = {
          dependencies =
            _: [ ];
          devDependencies =
            pkgs: with pkgs; [
              python3
              python313Packages.pyphen
              just
              ruff
            ];
        };
      }
    );
  inputs = {
    systems.url = "github:nix-systems/default";
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-parts = {
      url = "github:hercules-ci/flake-parts";
      inputs.nixpkgs-lib.follows = "nixpkgs";
    };
    import-tree.url = "github:vic/import-tree";
  };
}
