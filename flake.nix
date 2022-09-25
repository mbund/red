{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    utils.url = "github:numtide/flake-utils";
    naersk.url = "github:nix-community/naersk";
    flake-compat = {
      url = "github:edolstra/flake-compat";
      flake = false;
    };
    pre-commit-hooks = {
      url = "github:cachix/pre-commit-hooks.nix";
      inputs.nixpkgs.follows = "nixpkgs";
      inputs.flake-utils.follows = "utils";
    };
  };

  outputs = {
    nixpkgs,
    utils,
    naersk,
    pre-commit-hooks,
    ...
  }:
    utils.lib.eachDefaultSystem (
      system: let
        name = "red";
        pkgs = import nixpkgs {
          inherit system;
        };
        naersk-lib = naersk.lib."${system}";
        deps = with pkgs; [];
      in rec {
        packages.${name} = naersk-lib.buildPackage {
          pname = "${name}";
          root = ./.;
          doCheck = true;
          copyLibs = true;
          buildInputs = deps;
        };
        packages.default = packages.${name};

        apps.${name} = utils.lib.mkApp {
          inherit name;
          drv = packages.${name};
        };
        apps.default = apps.${name};

        checks.pre-commit-check = pre-commit-hooks.lib.${system}.run {
          src = nixpkgs.lib.cleanSource ../.;
          hooks = {
            alejandra.enable = true;
            statix.enable = true;

            rustfmt.enable = true;
            clippy.enable = true;
          };
        };

        devShells.default = pkgs.mkShell {
          name = "${name}-devshell";
          packages = with pkgs;
            [
              rustc
              cargo
              clippy
              rustfmt
              rust-analyzer
              alejandra
            ]
            ++ deps;
          shellHook = ''
            ${checks.pre-commit-check.shellHook}
          '';
        };
      }
    );
}
