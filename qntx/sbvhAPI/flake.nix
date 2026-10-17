{
  description = "QNTX plugin — booking a call with SBVH";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };

      in
      {
        # There is deliberately no package output. qntx-proto's build script
        # reads its protos from ../../plugin/grpc/protocol — a path that climbs
        # out of the crate and into the QNTX repo. Cargo checks that whole repo
        # out, so the path resolves; nix vendors each crate into a directory of
        # its own, so it does not, and protoc fails with "Could not make proto
        # path relative".
        #
        # Nix would buy us nothing here to make up for that: the only system
        # dependency is protoc, and the binary links nothing the target lacks.
        # So nix pins the toolchain and cargo does the build, in the shell
        # below and identically in CI.
        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            cargo
            rustc
            clippy
            rustfmt
            protobuf
            jq
          ];
          PROTOC = "${pkgs.protobuf}/bin/protoc";
        };
      });
}
