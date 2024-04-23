import L "lib";
import D "canister:dependency";

actor {
  public shared func greet(name : Text) : async Text {
    return "Hello, " # name # "!";
  };
};
