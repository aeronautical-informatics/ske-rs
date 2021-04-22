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
      ske-lib = ske.defaultPackage."${system}";
    in rec {
      # `nix build`
      packages.ske-rs = naersk-lib.buildPackage {
        pname = "ske-rs";
        src = ./.;
        doCheck = true;
        doDoc = true;
        overrideMain = _: { SKE_PATH = "${ske-lib}"; };
      };
      defaultPackage = packages.ske-rs;

      # `nix run`
      apps.ske-rs = utils.lib.mkApp {
        drv = packages.ske-rs;
      };
      defaultApp = apps.ske-rs;

      # `nix develop`
      devShell = pkgs.mkShell {
        nativeBuildInputs = with pkgs; [ rustc cargo ];
      };

      hydraJobs = packages.ske-rs;
    });
}
