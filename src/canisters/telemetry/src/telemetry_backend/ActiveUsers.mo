import Trie "mo:base/Trie";

import Aggregation "Aggregation";
import DfxVersion "DfxVersion";
import Platform "Platform";

module ActiveUsers
{
  public type Trie2D<K1, K2, V> = Trie.Trie2D<K1, K2, V>;
  public type Trie3D<K1, K2, K3, V> = Trie.Trie3D<K1, K2, K3, V>;
  type AggregationPeriodStart = Aggregation.AggregationPeriodStart;
  type DfxVersion = DfxVersion.DfxVersion;
  type Platform = Platform.Platform;

  public type ActiveUsers = Trie2D<AggregationPeriodStart, Platform, Nat>;
  public type ActiveUsersByVersion = Trie3D<
    AggregationPeriodStart, DfxVersion, Platform,
    Nat>;
}
