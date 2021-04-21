{
  inputs = {
    utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nmattia/naersk";
    ske.url = "git+ssh://git@github.com/aeronautical-informatics/ske?ref=main";
  };

  outputs = { self, nixpkgs, utils, naersk, ske }:
    utils.lib.eachSystem [ "x86_64-linux" ] (system: let
      pkgs = nixpkgs.legacyPackages."${system}";
      naersk-lib = naersk.lib."${system}";
      _ske = ske.defaultPackage."${system}";
    in rec {
      # `nix build`
      packages.my-project = naersk-lib.buildPackage {
        pname = "ske-rs";
        root = ./.;
        doCheck = true;
        doDoc = true;
        overrideMain = _: { postPatch = "cp ${_ske}/bin/libskeserver.so ."; };
      };
      defaultPackage = packages.my-project;

      # `nix run`
      apps.my-project = utils.lib.mkApp {
        drv = packages.my-project;
      };
      defaultApp = apps.my-project;

      # `nix develop`
      devShell = pkgs.mkShell {
        nativeBuildInputs = with pkgs; [ rustc cargo ];
      };

      hydraJobs = packages.my-project;
    });
}
