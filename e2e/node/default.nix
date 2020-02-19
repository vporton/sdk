{ pkgs ? import ../../nix { inherit system; }
, system ? builtins.currentSystem
, dfx
, userlib-js
}:
let
  e2e = pkgs.lib.noNixFiles (pkgs.lib.gitOnlySource ../. "node");
  sources = pkgs.sources;

  inputs = with pkgs; [
    coreutils
    nodejs-12_x
    dfx.standalone
  ];
in

pkgs.napalm.buildPackage e2e {
  root = ./.;
  name = "node-e2e-tests";
  PATH = pkgs.lib.makeSearchPath "bin" inputs;

  # ci script now does everything CI should do. Bundle is needed because it's the output
  # of the nix derivation.
  npmCommands = [
    "npm install"
    # Monkey-patch the userlib source into our install dir. napalm is unable
    # to include dependencies from package-locks in places other than the build
    # root.
    (
      pkgs.writeScript "include-userlib.sh" ''
        #!${pkgs.stdenv.shell}
        userlib="node_modules/@internet-computer/js-user-library"
        mkdir -p $userlib
        cp -R ${userlib-js.out}/* $userlib
        cp -R ${userlib-js.lib}/node_modules $userlib
      ''
    )
    "npm run ci"
  ];

  # Nothing to do in the install phase here.
  installPhase = ''
    touch $out
  '';
}
