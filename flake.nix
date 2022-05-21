{
  description = "A very basic flake";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    flake-compat = {
      url = "github:edolstra/flake-compat";
      flake = false;
    };
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, flake-utils, flake-compat, rust-overlay }:
  flake-utils.lib.eachDefaultSystem(system:
    let
      overlays = [ rust-overlay.overlay ];
      pkgs = import nixpkgs {  inherit system overlays; };

      rust = pkgs.rust-bin.fromRustupToolchainFile ./toolchain.toml;

      rustPlatform = pkgs.makeRustPlatform {
        cargo = rust;
        rustc = rust;
      };

      dependencies = with pkgs; 
        [ rust rust-analyzer rustfmt pkg-config wasm-bindgen-cli ];

    in {

      defaultPackage = rustPlatform.buildRustPackage rec {
        pname = "wgpu_dunes";
        version = "0.0.1";

        nativeBuildInputs = dependencies;

        src = ./.;

        buildPhase = ''
          cargo build --release --target=wasm32-unknown-unknown
        '';
        
        installPhase = ''
          echo 'Creating out dir...'
          mkdir -p $out/lib;

          echo 'Installing wasm module to $out/lib/'
          cp target/wasm32-unknown-unknown/release/${pname}.wasm $out/lib/;
        '';

        cargoLock = {
          lockFile = ./Cargo.lock;
        };

        verifyCargoDeps = true;
      };
      devShell = pkgs.mkShell { packages = dependencies; };
    }
  );
}
