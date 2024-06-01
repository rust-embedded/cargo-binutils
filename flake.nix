{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    flake-utils.url = "github:numtide/flake-utils";

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };
  };

  outputs = { self, nixpkgs, crane, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        rust = pkgs.rust-bin.beta.latest.default;
        craneLib = (crane.mkLib pkgs).overrideToolchain rust;

        my-crate = craneLib.buildPackage {
          src = craneLib.cleanCargoSource (craneLib.path ./.);

          buildInputs = [
            
            # Add additional build inputs here
          ];

          # Additional environment variables can be set directly
          # MY_CUSTOM_VAR = "some value";
        };
      in
      {
        packages.default = my-crate;

        devShells.default = craneLib.devShell {
          # Additional dev-shell environment variables can be set directly
          # MY_CUSTOM_DEV_URL = "http://localhost:3000";

          # Automatically inherit any build inputs from `my-crate`
          inputsFrom = [ my-crate ];

          # Extra inputs (only used for interactive development)
          # can be added here; cargo and rustc are provided by default.
          packages = [
            pkgs.cargo-audit
            pkgs.cargo-watch
          ];
        };

        devShells.lock = with pkgs; mkShell {
          buildInputs = [
            openssl
            pkg-config
            #eza
            #fd
          ]++[ rust ];

          shellHook = ''
          
          '';
        };
      });
}
