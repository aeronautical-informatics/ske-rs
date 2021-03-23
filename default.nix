{ pkgs ? import <nixpkgs> { }}:

with pkgs;

let 
  ske-server = import ../default.nix { inherit pkgs; };
  generatedBuild = callPackage ./Cargo.nix {
    defaultCrateOverrides = pkgs.defaultCrateOverrides // {
      ske-rs = attrs: {
        preConfigure = "cp ${ske-server}/bin/libskeserver.so libskeserver.so";
        #buildInputs = [ ske-server ];
      };
    };
  };
in generatedBuild.rootCrate.build
