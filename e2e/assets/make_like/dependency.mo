import L "lib";

actor {
    public shared func greet(name: Text) : async Text {
        return "Hello, " # name # "!";
    }
}