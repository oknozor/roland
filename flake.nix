{
  description = "Roland -- Standalone touchscreen daemon";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-26.05";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane.url = "github:ipetkov/crane";
  };

  outputs = { self, nixpkgs, rust-overlay, crane }:
  let
    overlays = [
      rust-overlay.overlays.default
      self.overlays.default
    ];

    pkgs = import nixpkgs {
      inherit overlays;
      config.allowUnfree = true;
      system = "x86_64-linux";
    };

    pkgsCross = import nixpkgs {
      inherit overlays;
      config.allowUnfree = true;
      system = "x86_64-linux";
      crossSystem.config = "aarch64-unknown-linux-gnu";
    };

    mkRustToolchain = pkgs: pkgs.rust-bin.stable.latest.default.override {
      extensions = [
        "rust-src"
        "rust-analyzer-preview"
        "llvm-tools"
      ];
      targets = [
        "x86_64-unknown-linux-gnu"
        "aarch64-unknown-linux-gnu"
      ];
    };

    shell = { mkShell, pkgsBuildHost, pkgsHostHost }:
      let
        rustToolchain = mkRustToolchain pkgsBuildHost;
      in mkShell {
        nativeBuildInputs = with pkgsBuildHost; [
          rustToolchain
          rustfmt
          clippy
          rustc
          rust-analyzer
          cargo-machete
          cargo-llvm-cov
          pkg-config
          libnotify
        ];

        buildInputs = with pkgsHostHost; [
          libinput
          udev
        ];

        shellHook = ''
          export LLVM_COV="${rustToolchain}/lib/rustlib/x86_64-unknown-linux-gnu/bin/llvm-cov"
          export LLVM_PROFDATA="${rustToolchain}/lib/rustlib/x86_64-unknown-linux-gnu/bin/llvm-profdata"
        '';
      };

  in {
    overlays.default = nixpkgs.lib.composeManyExtensions [
      rust-overlay.overlays.default
      (import ./nix/overlay.nix { inherit crane; })
    ];

    packages.x86_64-linux = {
      roland = pkgs.roland;
      default = pkgs.roland;
    };

    homeModules.default = ./nix/modules/roland.nix;

    devShells.x86_64-linux = {
      default = pkgs.callPackage shell {};
      cross = pkgsCross.callPackage shell {};
    };
  };
}
