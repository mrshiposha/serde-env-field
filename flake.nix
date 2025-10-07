{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/release-25.05";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-parts.url = "github:hercules-ci/flake-parts";
  };

  outputs = inputs@{ self, nixpkgs, flake-parts, rust-overlay }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [ "x86_64-linux" ];

      perSystem = { system, ... }:
        let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [ (import rust-overlay) ];
          };
        in {
          _module.args.pkgs = pkgs;

          devShells.default = with pkgs;
            mkShell rec {
              packages = [
                (rust-bin.stable.latest.default.override {
                  extensions = [ "rust-src" "rust-analyzer" ];
                })
              ];
              RUST_BACKTRACE = 1;
              LD_LIBRARY_PATH = "${pkgs.lib.makeLibraryPath packages}";
            };
        };
    };
}
