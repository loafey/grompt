{
    description = "A very basic flake";

    inputs = {
        nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
        flake-utils.url = "github:numtide/flake-utils";
        naersk.url = "github:nix-community/naersk";
        fenix = {
            url = "github:nix-community/fenix";
            inputs.nixpkgs.follows = "nixpkgs";
        };
    };

    outputs = { flake-utils, nixpkgs, naersk, fenix, ... }:
        flake-utils.lib.eachDefaultSystem (system:
        let 
            overlays = [ fenix.overlays.default ] ;
            pkgs = import nixpkgs {
                inherit system overlays;
            };
            toolchain = with fenix.packages.${system};  combine [
                complete.cargo
                complete.rustc
            ];
        in
        {
            defaultPackage = (naersk.lib.${system}.override {
                cargo = toolchain;
                rustc = toolchain;
            }).buildPackage {
                src = ./.;
                nativeBuildInputs = with pkgs; [
                    pkg-config
                    openssl
                ];
            };

            devShell = pkgs.mkShell {
                nativeBuildInputs = with pkgs; [
                    rustc
                    cargo
                    rust-analyzer
                    pkg-config
                    openssl
                ];
            };
        });
}
