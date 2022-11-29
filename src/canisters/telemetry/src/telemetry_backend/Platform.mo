import Hash "mo:base/Hash";

module Platform
{
  public type Platform = { #linux; #darwin; #windows };

  public func equal(a : Platform, b : Platform) : Bool {
    a == b
  };

  public func hash(p : Platform) : Hash.Hash {
    switch (p) {
      case (#linux) 10_338_022;      // Hash.hash(1)
      case (#darwin) 3_469_318_313;  // Hash.hash(2)
      case (#windows) 899_316_653;   // Hash.hash(3)
    }
  };
}
