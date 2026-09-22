{
  description = "default rust dev shell";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
      in
      {
        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            git
            ripgrep
            fd

            rustc
            cargo
            rustfmt
            clippy
            rust-analyzer
            pkg-config
            openssl
            gcc
          ];

          env = {
              RUST_SRC_PATH = "${pkgs.rust.packages.stable.rustPlatform.rustLibSrc}";
          };

          shellHook = ''
            echo "-> entered $(basename $PWD) dev shell"
            echo "-> rustc $(rustc --version)"
          '';
        };

        formatter = pkgs.nixpkgs-fmt;
      });
}
