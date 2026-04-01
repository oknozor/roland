{
  description = "Roland: Gesture daemon for Linux";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-25.11";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, rust-overlay }:
  let
    inherit (nixpkgs) lib;

    systems = lib.filter (lib.hasSuffix "-linux") lib.systems.flakeExposed;

    overlays = [ rust-overlay.overlays.default ];

    pkgsFor = system: import nixpkgs {
      inherit system overlays;
      config.allowUnfree = true;
    };

    forAllSystems = f: builtins.listToAttrs (map (system: {
      name = system;
      value = f system (pkgsFor system);
    }) systems);

    shell = { mkShell, pkgsBuildHost, pkgsHostHost }:
      mkShell {
        nativeBuildInputs = with pkgsBuildHost; [
          rustfmt
          clippy
          rustc
          rust-analyzer
          cargo-machete
          libnotify
          pkg-config
          gdb
        ] ++ (
          let
            rustToolchain = pkgsBuildHost.rust-bin.stable.latest.default.override {
              extensions = [
                "rust-src"
                "rust-analyzer-preview"
              ];
              targets = [
                "x86_64-unknown-linux-gnu"
                "aarch64-unknown-linux-gnu"
              ];
            };
          in [ rustToolchain ]
        );

        buildInputs = with pkgsHostHost; [
          libinput
          systemd
        ];
      };

  in {
    devShells = forAllSystems (system: pkgs: {
      default = pkgs.callPackage shell {};
    });

    packages = forAllSystems (system: pkgs: {
      default = pkgs.callPackage ./default.nix {};
    });
  };
}
