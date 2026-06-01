{ lib
, pkgs
, crane
}:

let
  rustToolchain = pkgs.rust-bin.stable.latest.default;
  craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

  src = craneLib.cleanCargoSource (craneLib.path ../..);

  commonArgs = {
    inherit src;

    nativeBuildInputs = [ pkgs.pkg-config ];
    buildInputs = [ pkgs.libinput pkgs.udev ];
  };

  cargoArtifacts = craneLib.buildDepsOnly commonArgs;

in craneLib.buildPackage (commonArgs // {
  inherit cargoArtifacts;
  doCheck = false;

  meta = {
    description = "Compositor-agnostic touchscreen gesture daemon";
    license = lib.licenses.mit;
    mainProgram = "roland";
    platforms = lib.platforms.linux;
  };
})
