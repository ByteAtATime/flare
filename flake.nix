{
  description = "Flare: a Raycast-compatible launcher on Linux";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};

        pname = "flare";
        version = "0.1.0";
        src = pkgs.fetchurl {
          url = "https://github.com/ByteAtATime/flare/releases/download/v0.1.0/flare_0.1.0_amd64.AppImage";
          hash = "sha256-uASP1JoHD+gUFUxfsEYUd1EdpDfBUO458ict6MRdyDw=";
        };
        appimageContents = pkgs.appimageTools.extract { inherit pname version src; };

        flare = pkgs.appimageTools.wrapType2 {
          inherit pname version src;
          extraInstallCommands = ''
            install -m 444 -D ${appimageContents}/${pname}.desktop -t $out/share/applications
            substituteInPlace $out/share/applications/${pname}.desktop \
              --replace 'Exec=AppRun' 'Exec=${pname}'
            cp -r ${appimageContents}/usr/share/icons $out/share
          '';
          extraBwrapArgs = [
            "--bind-try /etc/nixos/ /etc/nixos/"
          ];
          dieWithParent = false;
        };
      in
      {
        packages.default = flare;
        packages.flare = flare;

        apps.default = flake-utils.lib.mkApp {
          drv = flare;
        };

        apps.flare = flake-utils.lib.mkApp {
          drv = flare;
        };
      });
}
