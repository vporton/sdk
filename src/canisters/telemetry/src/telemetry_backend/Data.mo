import Time "mo:base/Time";
import Trie "mo:base/Trie";

import ActiveUsers "ActiveUsers";
import CommandResults "CommandResults";

module Data {
  public type Time = Time.Time;
  public type Trie<K, V> = Trie.Trie<K, V>;
  type ActiveUsers = ActiveUsers.ActiveUsers;
  type ActiveUsersByVersion = ActiveUsers.ActiveUsersByVersion;
  type CommandResults = CommandResults.CommandResults;

  public type V0 = {
    var dailyAggregationPeriodStart : Time;
    var dailyAggregationPeriodEnd : Time;

    var thirtyDayAggregationPeriodStart : Time;
    var thirtyDayAggregationPeriodEnd : Time;

    var commandResults : CommandResults;

    var overrideTime : ?Time;
  };

  public type V1 = {
    var dailyAggregationPeriodStart : Time;
    var dailyAggregationPeriodEnd : Time;

    var thirtyDayAggregationPeriodStart : Time;
    var thirtyDayAggregationPeriodEnd : Time;

    var commandResults : CommandResults;

    var dailyActiveUsers : ActiveUsers;
    var dailyActiveUsersByVersion : ActiveUsersByVersion;

    var overrideTime : ?Time;
  };

  public type V2 = {
    var dailyAggregationPeriodStart : Time;
    var dailyAggregationPeriodEnd : Time;

    var thirtyDayAggregationPeriodStart : Time;
    var thirtyDayAggregationPeriodEnd : Time;

    var commandResults : CommandResults;

    var dailyActiveUsers : ActiveUsers;
    var dailyActiveUsersByVersion : ActiveUsersByVersion;
    var monthlyActiveUsers : ActiveUsers;

    var overrideTime : ?Time;
  };

  public type Data = V2;
  public type Versioned = {
    #v0 : V0;
    #v1 : V1;
    #v2 : V2;
  };

  public func new() : Data {
    {
      var dailyAggregationPeriodStart = 0;
      var dailyAggregationPeriodEnd = 0;

      var thirtyDayAggregationPeriodStart = 0;
      var thirtyDayAggregationPeriodEnd = 0;

      var commandResults = Trie.empty();

      var dailyActiveUsers = Trie.empty();
      var dailyActiveUsersByVersion = Trie.empty();

      var monthlyActiveUsers = Trie.empty();

      var overrideTime = null;
    }
  };

  public func fromV0(prev: V0) : Data {
    {
      var dailyAggregationPeriodStart = prev.dailyAggregationPeriodStart;
      var dailyAggregationPeriodEnd = prev.dailyAggregationPeriodEnd;

      var thirtyDayAggregationPeriodStart = prev.thirtyDayAggregationPeriodStart;
      var thirtyDayAggregationPeriodEnd = prev.thirtyDayAggregationPeriodEnd;

      var commandResults = prev.commandResults;

      var dailyActiveUsers = Trie.empty();
      var dailyActiveUsersByVersion = Trie.empty();

      var monthlyActiveUsers = Trie.empty();

      var overrideTime = prev.overrideTime;
    }
  };

  public func fromV1(prev: V1) : Data {
    {
      var dailyAggregationPeriodStart = prev.dailyAggregationPeriodStart;
      var dailyAggregationPeriodEnd = prev.dailyAggregationPeriodEnd;

      var thirtyDayAggregationPeriodStart = prev.thirtyDayAggregationPeriodStart;
      var thirtyDayAggregationPeriodEnd = prev.thirtyDayAggregationPeriodEnd;

      var commandResults = prev.commandResults;

      var dailyActiveUsers = prev.dailyActiveUsers;
      var dailyActiveUsersByVersion = prev.dailyActiveUsersByVersion;

      var monthlyActiveUsers = Trie.empty();

      var overrideTime = prev.overrideTime;
    }
  };

}
