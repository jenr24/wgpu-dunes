{
  description = "A very basic flake";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    flake-compat = {
      url = "github:edolstra/flake-compat";
      flake = false;
    };
    mozilla-rust.url = "github:mozilla/nixpkgs-mozilla/master";
  };

  outputs = { self, nixpkgs, flake-utils, flake-compat, mozilla-rust }:
  flake-utils.lib.eachDefaultSystem(system:
    let
      pkgs = import nixpkgs { 
        inherit system;
        overlays = [ mozilla-rust.overlay ]; 
      };

      rust = (pkgs.rustChannelOf { rustToolchain = ./toolchain.toml; }).rust;
      rustPlatform = pkgs.makeRustPlatform {
        cargo = rust;
        rustc = rust;
      };

      dependencies = with pkgs; [ rust rls rustfmt pkg-config ];

    in pkgs.stdenv.mkDerivation rec {
      name = "wgpu-dunes";
      nativeBuildInputs = dependencies;
      buildInputs = dependencies;

      defaultPackage = rustPlatform.buildRustPackage rec {
        pname = name;
        version = "0.0.1";

        inherit buildInputs;
        inherit nativeBuildInputs;

        src = pkgs.fetchFromGitHub {
          owner = "jenr24";
          repo = pname;
        };

        cargoLock = {
          lockFile = ./Cargo.lock;
        };

        verifyCargoDeps = true;
    };

    }
  );
}
