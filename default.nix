{ lib
, fetchFromGitHub
, rustPlatform
, writeText
, pkg-config
, libinput
, systemd
}:

rustPlatform.buildRustPackage (finalAttrs: {
  pname = "roland";
  version = "01-02-2026-unstable";

  src = ./.;

  nativeBuildInputs = [
    pkg-config
  ];

  buildInputs = [
    libinput
    systemd
  ];

  cargoHash = "sha256-3r80j3UXQIIxhTIKozWcqxQSSRzDH8K1ND6vFTivXD8=";

  meta = {
    homepage = "https://github.com/oknozor/roland";
    description = "A simple touch gesture recognizer for Linux";
    license = lib.licenses.mit;
    platforms = lib.platforms.linux;
  };
})
