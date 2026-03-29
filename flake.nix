{
  description = "sbvh.nl";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in
      {
        packages.default = pkgs.stdenv.mkDerivation {
          pname = "sbvh-nl";
          version = self.rev or "dev";
          src = ./.;

          nativeBuildInputs = [ pkgs.bun ];

          buildPhase = ''
            export HOME=$(mktemp -d)
            bun install --frozen-lockfile
            bun run build.ts
          '';

          installPhase = ''
            cp -r dist $out
          '';
        };

        devShells.default = pkgs.mkShell {
          buildInputs = [ pkgs.bun ];
        };
      });
}
