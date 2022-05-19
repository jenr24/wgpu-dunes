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

  outputs = { self, nixpkgs }: flake-utils.lib.eachDefaultSystem(system:
    let
      pkgs = import nixpkgs { 
        inherit system;
        overlays = [ mozilla-rust.overlay ]; 
      };

      rust = (pkgs.rustChannelOf { rustToolchain = ./toolchain.toml; }).rust;
      rustPlatform = makeRustPlatform {
        cargo = rust;
        rustc = rust;
      };

    in rustPlatform.buildRustPackage rec {
      pname = "wgpu-dunes";
      version = "0.0.1";

      src = fetchFromGitHub {
        owner = "jenr24";
        repo = pname;
      };

      cargoLock = {
        lockFile = ./Cargo.lock;
      };

      verifyCargoDeps = true;
    }
  );
}
