{ crane }:

final: prev: {
  roland = final.callPackage ./packages/roland.nix {
    inherit crane;
  };
}
