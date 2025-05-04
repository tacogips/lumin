{
  description = "lumin";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
      fenix,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = nixpkgs.legacyPackages.${system};

        # Get Rust toolchain from fenix - with updated hash
        rust-toolchain = fenix.packages.${system}.fromToolchainFile {
          file = ./rust-toolchain.toml;
          sha256 = "sha256-AJ6LX/Q/Er9kS15bn9iflkUwcgYqRQxiOIL2ToVAXaU=";
        };

        # Create a modified buildRustPackage that skips the problematic steps
        buildRustPackageCustom =
          args:
          pkgs.rustPlatform.buildRustPackage (
            args
            // {
              # These options help bypass the workspace inheritance issues
              dontFixCargo = true;
              cargoLockCheck = false;
              doCheck = false;
            }
          );
          
        # Build cargo-machete with the same rust-toolchain version
        cargo-machete = pkgs.fetchFromGitHub {
          owner = "bnjbvr";
          repo = "cargo-machete";
          rev = "v0.8.0";
          sha256 = "sha256-0vlau3leAAonV5E9NAtSqw45eKoZBzHx0BmoEY86Eq8=";
        };
        
        # Shell script to build and run cargo-machete with the specific toolchain
        cargo-machete-wrapper = pkgs.writeShellScriptBin "cargo-machete" ''
          # Use the specific rust-toolchain version
          export PATH=${rust-toolchain}/bin:$PATH
          
          # Create a temporary build directory
          TEMP_DIR=$(mktemp -d)
          trap "rm -rf $TEMP_DIR" EXIT
          
          # Copy source to temporary directory
          cp -r ${cargo-machete}/* $TEMP_DIR/
          cd $TEMP_DIR
          
          # Build with the specific toolchain
          if [ ! -f ~/.cargo/.cargo-machete-built ]; then
            echo "Building cargo-machete with specific Rust toolchain..."
            cargo build --release
            cp target/release/cargo-machete ~/.cargo/bin/
            touch ~/.cargo/.cargo-machete-built
          fi
          
          # Run the installed binary
          exec ~/.cargo/bin/cargo-machete "$@"
        '';
      in
      {
        # Development shell with Rust toolchain
        devShells.default = pkgs.mkShell {
          packages = [
            rust-toolchain
            pkgs.nixpkgs-fmt
            pkgs.openssl
            pkgs.pkg-config
            pkgs.nodejs
            pkgs.nodePackages.npm
            pkgs.go-task
            cargo-machete-wrapper
          ];

          # Add OpenSSL configuration
          shellHook = ''
            export OPENSSL_DIR=${pkgs.openssl.dev}
            export OPENSSL_LIB_DIR=${pkgs.openssl.out}/lib
            export OPENSSL_INCLUDE_DIR=${pkgs.openssl.dev}/include
            echo "Shell loaded successfully with OpenSSL configuration"
          '';
        };

        # Simple package definition
        packages.default = buildRustPackageCustom {
          pname = "lumin";
          version = "0.1.3";
          src = ./.;

          # Basic cargo lock configuration
          cargoLock = {
            lockFile = ./Cargo.lock;
            allowBuiltinFetchGit = true;
          };

          # Enable Git fetching with CLI
          CARGO_NET_GIT_FETCH_WITH_CLI = "true";
          CARGO_TERM_VERBOSE = "true";

          nativeBuildInputs = [
            rust-toolchain
            pkgs.pkg-config
          ];

          buildInputs =
            [
              pkgs.openssl
            ]
            ++ pkgs.lib.optionals pkgs.stdenv.isDarwin [
              pkgs.libiconv
            ];

          # OpenSSL environment variables
          OPENSSL_DIR = "${pkgs.openssl.dev}";
          OPENSSL_LIB_DIR = "${pkgs.openssl.out}/lib";
          OPENSSL_INCLUDE_DIR = "${pkgs.openssl.dev}/include";
        };
      }
    );
}
