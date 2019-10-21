self: super:

let src = builtins.fetchGit {
  url = "ssh://git@github.com/dfinity-lab/dfinity";
  # ref = "v0.2.0"; # TODO
  rev = "3dbb6c63468cc4de15f4984b72a0c15ed3df1ba0";
}; in

{
  dfinity = (import src { inherit (self) system; }).dfinity;
}
