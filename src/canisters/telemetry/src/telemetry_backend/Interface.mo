import Command "Command";
import CommandResults "CommandResults";
import DfxVersion "DfxVersion";
import Network "Network";
import Parameters "Parameters";
import Platform "Platform";

module Interface {
  type Command = Command.Command;
  type DfxVersion = DfxVersion.DfxVersion;
  type Network = Network.Network;
  type Parameters = Parameters.Parameters;
  type Platform = Platform.Platform;

  public type ReportCommandArgs = {
    dfxVersion : Text;
    platform : Platform;
    network : Network;
    command : Command;
    parameters : ?Parameters;
    success : Bool;
  };

  public type CommandSuccessRatesEntry = {
    command : Command;
    parameters : ?Parameters;
    successRate : Nat;
  };

  public type CommandResultsEntry = {
    command : Command;
    parameters : ?Parameters;
    successes : Nat;
    failures : Nat;
  };

  public type ReportPeriodicUseArgs = {
    dfxVersion: DfxVersion;
    platform : Platform;
  };

  public type ActiveUsersEntry = {
    platform : Platform;
    users : Nat;
  };
}
