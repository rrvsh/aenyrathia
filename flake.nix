{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-parts.url = "github:hercules-ci/flake-parts";
    flake-parts.inputs.nixpkgs-lib.follows = "nixpkgs";
  };
  outputs =
    inputs:
    inputs.flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [
        "aarch64-darwin"
        "x86_64-linux"
        "aarch64-linux"
      ];
      perSystem =
        { pkgs, ... }:
        let
          aenyrathia = pkgs.rustPlatform.buildRustPackage {
            pname = "aenyrathia";
            version = "0.1.0-alpha";

            src = pkgs.lib.cleanSource ./.;

            cargoLock.lockFile = ./Cargo.lock;

            nativeBuildInputs = with pkgs; [
              makeWrapper
              pkg-config
            ];

            buildInputs = with pkgs; [
              openssl
            ];

            postInstall = ''
              mkdir -p $out/share/aenyrathia
              cp -R static $out/share/aenyrathia/static

              wrapProgram $out/bin/aenyrathia \
                --set-default STATIC_DIR $out/share/aenyrathia/static
            '';

            meta = {
              description = "Browsable/editable git-backed wiki for Aenyrathia";
              homepage = "https://github.com/rrvsh/aenyrathia";
              license = pkgs.lib.licenses.mit;
              mainProgram = "aenyrathia";
            };
          };
        in
        {
          packages = {
            inherit aenyrathia;
            default = aenyrathia;
          };

          devShells.default = pkgs.mkShell {
            buildInputs = with pkgs; [
              bacon
              cargo
              clippy
              just
              openssl
              pkg-config
              rustc
              rustfmt
              sqlx-cli
            ];
          };
        };
    };
}
