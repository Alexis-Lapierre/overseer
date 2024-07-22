{
  description = "devShell with lsp and compiler for overseer";

  inputs = {
    nixpkgs.url      = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url  = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
      in
      {
        devShells.default = with pkgs; mkShell rec {
          buildInputs = [
            rust-bin.beta.latest.default
            rust-analyzer

            # Runtime dependencies of iced, and underlying winit
            libxkbcommon
            libGL
            # TODO: detect when we are running wayland to only add it then
            wayland
          ];

          shellHook = ''
            export RUST_BACKTRACE=1
            export LD_LIBRARY_PATH="${lib.makeLibraryPath buildInputs}"

            echo "Cargo version: $(cargo --version)"
            echo "Rust Analyzer: $(rust-analyzer --version)"
            echo variable RUST_BACKTRACE=$RUST_BACKTRACE
          '';
        };
      }
    );

    
}
