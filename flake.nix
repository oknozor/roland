{
  description = "Roland -- Standalone touchscreen daemon";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-26.05";
    crane.url = "github:ipetkov/crane";
  };

  outputs = { self, nixpkgs, crane }:
  let
    overlays = [
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

    shell = { mkShell, pkgsBuildHost, pkgsHostHost }:
      mkShell {
        nativeBuildInputs = with pkgsBuildHost; [
          rustfmt
          clippy
          rustc
          cargo
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
          export LLVM_COV="${pkgsBuildHost.llvmPackages.bintools}/bin/llvm-cov"
          export LLVM_PROFDATA="${pkgsBuildHost.llvmPackages.bintools}/bin/llvm-profdata"
        '';
      };

  in {
    overlays.default = nixpkgs.lib.composeManyExtensions [
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
