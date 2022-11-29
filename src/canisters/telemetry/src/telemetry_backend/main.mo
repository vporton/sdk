import Debug "mo:base/Debug";
import Hash "mo:base/Hash";
import Int "mo:base/Int";
import Nat "mo:base/Nat";
import Option "mo:base/Option";
import Prelude "mo:base/Prelude";
import Text "mo:base/Text";
import Time "mo:base/Time";
import Trie "mo:base/Trie";

import ActiveUsers "ActiveUsers";
import Aggregation "Aggregation";
import CommandDetails "CommandDetails";
import CommandResults "CommandResults";
import Data "Data";
import DfxVersion "DfxVersion";
import InvocationDetails "InvocationDetails";
import Interface "Interface";
import Platform "Platform";

actor {
  type Time = Time.Time;
  type Trie<K, V> = Trie.Trie<K, V>;
  type Trie2D<K1, K2, V> = Trie.Trie2D<K1, K2, V>;

  type ActiveUsers = ActiveUsers.ActiveUsers;
  type ActiveUsersByVersion = ActiveUsers.ActiveUsersByVersion;
  type ActiveUsersEntry = Interface.ActiveUsersEntry;
  type AggregationPeriodStart = Aggregation.AggregationPeriodStart;
  type CommandDetails = CommandDetails.CommandDetails;
  type DfxVersion = DfxVersion.DfxVersion;
  type InvocationDetails = InvocationDetails.InvocationDetails;
  type CommandResultsEntry = Interface.CommandResultsEntry;
  type CommandResultCounts = CommandResults.CommandResultCounts;
  type CommandSuccessRatesEntry = Interface.CommandSuccessRatesEntry;
  type Platform = Platform.Platform;
  type ReportPeriodicUseArgs = Interface.ReportPeriodicUseArgs;
  type ReportCommandArgs = Interface.ReportCommandArgs;
  type V0 = Data.V0;
  type V1 = Data.V1;

  let nsPerDay = 86_400 * 1000_000_000;
  let nsPer30Days = 30 * nsPerDay;

  stable var versioned : Data.Versioned = #v1(Data.new());
  var data : Data.Data = switch (versioned) {
    case (#v1 v1) v1;
    case (_) Data.new();
  };

  system func postupgrade() {
    switch (versioned) {
      case (#v0 v0) {
        data := Data.fromV0(v0);
        versioned := #v1(data);
      };
      case (#v1 v1) {};
    };
  };

  func currentTime() : Time {
    switch (data.overrideTime) {
      case null Time.now();
      case (?override) override;
    }
  };

  func updateAggregationPeriods() {
    let now = currentTime();

    if (now > data.dailyAggregationPeriodEnd) {
      let day : Time = now / nsPerDay;
      data.dailyAggregationPeriodStart := day * nsPerDay;
      data.dailyAggregationPeriodEnd :=
        data.dailyAggregationPeriodStart + nsPerDay - 1;
    };

    if (now > data.thirtyDayAggregationPeriodEnd) {
      if (data.thirtyDayAggregationPeriodStart == 0) {
        data.thirtyDayAggregationPeriodStart :=
          data.dailyAggregationPeriodStart;
      } else {
        let periodsToAdd =
          (now - data.thirtyDayAggregationPeriodStart) / nsPer30Days;
        data.thirtyDayAggregationPeriodStart += periodsToAdd * nsPer30Days;
      };
      data.thirtyDayAggregationPeriodEnd :=
        data.thirtyDayAggregationPeriodStart + nsPer30Days - 1;
    };
  };

  public func reportCommandResult(args: ReportCommandArgs) : async () {
    updateAggregationPeriods();
    let aggregationPeriod = data.thirtyDayAggregationPeriodStart;
    let invocationDetails : InvocationDetails = {
      dfxVersion = args.dfxVersion;
      platform = args.platform;
      network = args.network;
    };
    let commandDetails : CommandDetails = {
      command = args.command;
      parameters = args.parameters;
    };

    let k1 = Aggregation.trieKey(aggregationPeriod);
    let k2 = InvocationDetails.trieKey(invocationDetails);
    let k3 = CommandDetails.trieKey(commandDetails);

    let (t,v) = Trie.remove3D(
      data.commandResults,
      k1, Int.equal,
      k2, InvocationDetails.equal,
      k3, CommandDetails.equal);
    let counts : CommandResultCounts = switch(v) {
      case null { { var successes = 0; var failures = 0 } };
      case (?x) x;
    };
    if (args.success) {
      counts.successes += 1;
    } else {
      counts.failures += 1;
    };

    data.commandResults := Trie.put3D(
      t,
      k1, Int.equal,
      k2, InvocationDetails.equal,
      k3, CommandDetails.equal,
      counts);
  };

  public query func getCommandResultReportingPeriods(
    ) : async [AggregationPeriodStart] {
    Trie.toArray(
      data.commandResults,
      func (
        k: AggregationPeriodStart,
        v: Trie2D<InvocationDetails, CommandDetails, CommandResultCounts>
      ) : AggregationPeriodStart {
        k
      }
    )
  };

  public query func getInvocationDetailsForReportingPeriod(
    aggregationPeriod : AggregationPeriodStart
  ): async [InvocationDetails] {
    let x = Trie.find(data.commandResults,
      Aggregation.trieKey(aggregationPeriod),
      Int.equal);
    switch (x) {
      case null [];
      case (?t) Trie.toArray(t,
        func (
          k: InvocationDetails,
          v: Trie<CommandDetails, CommandResultCounts>
        ) : InvocationDetails {
          k
        });
    }
  };

  public query func getCommandSuccessRates(
    aggregationPeriod: AggregationPeriodStart,
    invocationDetails: InvocationDetails
  ): async [CommandSuccessRatesEntry] {
    let x1 = Trie.find(data.commandResults,
      Aggregation.trieKey(aggregationPeriod),
      Int.equal);
    let t1 = switch (x1) {
      case null Trie.empty();
      case (?t) t;
    };
    let k2 = InvocationDetails.trieKey(invocationDetails);
    let x2 = Trie.find(t1, k2, InvocationDetails.equal);

    switch (x2) {
      case null [];
      case (?t) Trie.toArray(t,
        func(
          k : CommandDetails,
          v : CommandResultCounts
        ) : CommandSuccessRatesEntry {
          {
            command = k.command;
            parameters = k.parameters;
            successRate = v.successes * 100 / (v.successes + v.failures);
          }
        });
    }
  };

  // Active Users (by platform)

  func reportActiveUser(
    activeUsers : ActiveUsers,
    aggregationPeriod : AggregationPeriodStart,
    platform : Platform
  ) : ActiveUsers {
    let k1 = { key = aggregationPeriod; hash = Int.hash aggregationPeriod };
    let k2 = { key = platform; hash = Platform.hash(platform) };

    let (t,v) = Trie.remove2D(
      activeUsers,
      k1, Int.equal,
      k2, Platform.equal);
    let users = Option.get(v, 0) + 1;

    Trie.put2D(
      t,
      k1, Int.equal,
      k2, Platform.equal,
      users);
  };

  func getActiveUsersReportingPeriods(
    activeUsers : ActiveUsers
  ) : [AggregationPeriodStart] {
    Trie.toArray(
      activeUsers,
      func (
        k: AggregationPeriodStart,
        v: Trie<Platform, Nat>
      ) : AggregationPeriodStart {
        k
      }
    )
  };

  func getActiveUsers(
    activeUsers : ActiveUsers,
    aggregationPeriod: AggregationPeriodStart
  ) : [ActiveUsersEntry] {
    let x1 = Trie.find(activeUsers,
      { key = aggregationPeriod; hash = Int.hash aggregationPeriod },
      Int.equal);

    switch (x1) {
      case null [];
      case (?t) Trie.toArray(t,
        func(
          k : Platform,
          v : Nat
        ) : ActiveUsersEntry {
          {
            platform = k;
            users = v;
          }
        });
    }
  };


  // Active Users (by dfx version)

  func reportActiveUserByVersion(
    activeUsers : ActiveUsersByVersion,
    aggregationPeriod : AggregationPeriodStart,
    dfxVersion : DfxVersion,
    platform : Platform
  ) : ActiveUsersByVersion {
    let k1 = { key = aggregationPeriod; hash = Int.hash aggregationPeriod };
    let k2 = { key = dfxVersion; hash = Text.hash dfxVersion };
    let k3 = { key = platform; hash = Platform.hash(platform) };

    let (t,v) = Trie.remove3D(
      activeUsers,
      k1, Int.equal,
      k2, Text.equal,
      k3, Platform.equal);
    let users = Option.get(v, 0) + 1;

    Trie.put3D(
      t,
      k1, Int.equal,
      k2, Text.equal,
      k3, Platform.equal,
      users);
  };

  func getActiveUsersByVersionReportingPeriods(
    activeUsers : ActiveUsers
  ) : [AggregationPeriodStart] {
    Trie.toArray(
      activeUsers,
      func (
        k: AggregationPeriodStart,
        v: Trie<Platform, Nat>
      ) : AggregationPeriodStart {
        k
      }
    )
  };

  func getActiveUsersByVersion(
    activeUsers : ActiveUsers,
    aggregationPeriod: AggregationPeriodStart
  ) : [ActiveUsersEntry] {
    let x1 = Trie.find(activeUsers,
      { key = aggregationPeriod; hash = Int.hash aggregationPeriod },
      Int.equal);

    switch (x1) {
      case null [];
      case (?t) Trie.toArray(t,
        func(
          k : Platform,
          v : Nat
        ) : ActiveUsersEntry {
          {
            platform = k;
            users = v;
          }
        });
    }
  };

  // Daily Active Users
  public func reportDailyUse(args : ReportPeriodicUseArgs) : async () {
    updateAggregationPeriods();
    data.dailyActiveUsers := reportActiveUser(
      data.dailyActiveUsers,
      data.dailyAggregationPeriodStart,
      args.platform);
    data.dailyActiveUsersByVersion := reportActiveUserByVersion(
      data.dailyActiveUsersByVersion,
      data.dailyAggregationPeriodStart,
      args.dfxVersion,
      args.platform);
  };

  public query func getDailyActiveUsersReportingPeriods(
  ) : async [AggregationPeriodStart] {
    getActiveUsersReportingPeriods(data.dailyActiveUsers)
  };

  public query func getDailyUsers(
    aggregationPeriod: AggregationPeriodStart
  ) : async [ActiveUsersEntry] {
    getActiveUsers(data.dailyActiveUsers, aggregationPeriod)
  };

  // todo: access control https://dfinity.atlassian.net/browse/SDK-864
  public query func testGetCommandResults(
    aggregationPeriod: AggregationPeriodStart,
    invocationDetails: InvocationDetails
  ): async [CommandResultsEntry] {
    let x1 = Trie.find(data.commandResults,
      Aggregation.trieKey(aggregationPeriod),
      Int.equal);
    let t1 = switch (x1) {
      case null Trie.empty();
      case (?t) t;
    };
    let k2 = InvocationDetails.trieKey(invocationDetails);
    let x2 = Trie.find(t1, k2, InvocationDetails.equal);

    switch (x2) {
      case null [];
      case (?t) Trie.toArray(t,
        func(
          k : CommandDetails,
          v : CommandResultCounts
        ) : CommandResultsEntry {
          {
            command = k.command;
            parameters = k.parameters;
            successes = v.successes;
            failures = v.failures;
          }
        });
    }
  };

  // todo: access control https://dfinity.atlassian.net/browse/SDK-864
  public func testSetTime(v : Time) : async () {
    data.overrideTime := ?v;
  };
};
