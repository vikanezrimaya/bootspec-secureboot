{
  description = "bootloader-secureboot";

  inputs.nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";

  outputs =
    { self
    , nixpkgs
    , ...
    } @ inputs:
    let
      nameValuePair = name: value: { inherit name value; };
      genAttrs = names: f: builtins.listToAttrs (map (n: nameValuePair n (f n)) names);
      allSystems = [ "x86_64-linux" "aarch64-linux" "i686-linux" "x86_64-darwin" ];

      forAllSystems = f: genAttrs allSystems (system: f {
        inherit system;
        pkgs = import nixpkgs { inherit system; };
      });
    in
    {
      devShell = forAllSystems ({ system, pkgs, ... }:
        pkgs.mkShell {
          name = "bootspec-secureboot";

          buildInputs = with pkgs; [
            cargo
            rustc
            clippy
            codespell
            nixpkgs-fmt
            rustfmt
            rust-analyzer
          ];
        });

      packages = forAllSystems
        ({ system, pkgs, ... }:
          rec {
            package = pkgs.rustPlatform.buildRustPackage {
              pname = "bootspec-secureboot";
              version = "unreleased";

              src = self;

              cargoLock = {
                lockFile = ./Cargo.lock;
                outputHashes = {
                  "bootspec-1.0.1" = "sha256-DLJ0JR4WfBCG24UFW9qiJle1D4avxTzP5uyx0+AeAYw=";
                };
              };

              postPatch = ''
                substituteInPlace installer/build.rs \
                  --replace "@patched_sbattach@" "${sbattach}"
              '';
            };

            sbattach = import ./installer/patched-sbattach.nix { inherit pkgs; };
          });

      defaultPackage = forAllSystems ({ system, ... }: self.packages.${system}.package);

      nixosModules.bootspec-secureboot = {
        imports = [ ./nixos-module.nix ];
        nixpkgs.overlays = [
          (final: prev: {
            bootspec-secureboot = self.defaultPackage."${final.stdenv.targetPlatform.system}";
          })
        ];
      };
    };
}
