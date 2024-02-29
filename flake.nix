{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    crane = {
      url = "github:ipetkov/crane";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        rust-overlay.follows = "rust-overlay";
      };
    };
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
      };
    };
  };
  outputs = { self, nixpkgs, crane, rust-overlay }:
    let
      overlays = [ (import rust-overlay) ];
      system = "x86_64-linux";
      pkgs = import nixpkgs {
        inherit system overlays;
      };
      craneLib = (crane.mkLib pkgs).overrideToolchain pkgs.pkgsBuildHost.rust-bin.stable.latest.default;
      src = craneLib.cleanCargoSource ./.;
      nativeBuildInputs = with pkgs; [ pkg-config rust-bin.stable.latest.default ];
      buildInputs = with pkgs; [ openssl ];
      commonArgs = {
        inherit src buildInputs nativeBuildInputs;
      };
      cargoArtifacts = craneLib.buildDepsOnly commonArgs;
      bin = craneLib.buildPackage (commonArgs // {
        doCheck = false;
        inherit cargoArtifacts;
      });
    in
    with pkgs;
    {
      packages.${system} = {
        inherit bin;
        default = bin;
      };
      devShells.${system}.default = mkShell {
        inputsFrom = [ bin ];
      };
    };
}
