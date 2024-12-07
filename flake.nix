{
  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-24.11";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = (import nixpkgs) {
          inherit system;
        };

        rustPlatform = pkgs.makeRustPlatform {
          cargo = pkgs.cargo;
          rustc = pkgs.rustc;
        };

        lintingRustFlags = "-D unused-crate-dependencies";
      in {
        devShell = pkgs.mkShell {
          packages = with pkgs; [
            # Rust toolchain
            cargo
            rustc

            # Code analysis tools
            clippy
            rust-analyzer

            # Code formatting tools
            treefmt
            alejandra
            mdl
            rustfmt

            # Rust dependency linting
            cargo-deny

            # Container image management tool
            skopeo
          ];

          RUSTFLAGS = lintingRustFlags;
        };

        packages = rec {
          default = rustPlatform.buildRustPackage {
            pname = "emfcamp-dial-a-schedule";
            version = "0.1.0";

            src = ./.;

            cargoLock = {
              lockFile = ./Cargo.lock;

              outputHashes = {
                "emfcamp-schedule-api-0.0.1" = "sha256-beyMI00aB9gvFIenNiuXRg8tnnc30zTiMWVIlV/UZGQ=";
              };
            };
          };

          container-image = pkgs.dockerTools.buildImage {
            name = "emfcamp-dial-a-schedule";
            tag = "latest";
            created = "now";

            copyToRoot = pkgs.buildEnv {
              name = "image-root";
              paths = [pkgs.bashInteractive pkgs.coreutils];
              pathsToLink = ["/bin"];
            };

            config = {
              Entrypoint = ["${pkgs.tini}/bin/tini" "--" "${default}/bin/emfcamp-dial-a-schedule"];
              ExposedPorts = {
                "8000/tcp" = {};
                "9090/tcp" = {};
              };
              Env = [
                "SSL_CERT_FILE=${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt"
                "WEBHOOK_ADDRESS=0.0.0.0:8000"
                "OBSERVABILITY_ADDRESS=0.0.0.0:9090"
              ];
            };
          };
        };
      }
    );
}
