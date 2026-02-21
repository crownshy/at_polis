{
  description = "Rust Jetstream consumer dev environment";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        rust = pkgs.rust-bin.stable.latest.default;
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = [
            rust
            pkgs.rust-analyzer
            pkgs.pkg-config
            pkgs.openssl
          ];

          shellHook = ''
            export RUST_SRC_PATH=${rust}/lib/rustlib/src/rust/library
            echo "🦀 Rust dev shell ready (with rust-analyzer)"
          '';
        };
      }
    );
}
