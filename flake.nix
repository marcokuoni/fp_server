{
  description = "Rust dev env with nightly (edition2024 ready)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.11";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
      ...
    }:
    let
      system = "x86_64-linux";

      pkgs = import nixpkgs {
        inherit system;
        overlays = [ (import rust-overlay) ];
      };

      # Nightly toolchain for edition2024
      rustToolchain = pkgs.rust-bin.nightly.latest.default;

      # Use that toolchain for buildRustPackage as well
      rustPlatform = pkgs.makeRustPlatform {
        cargo = rustToolchain;
        rustc = rustToolchain;
      };

      # Nix build of the Rust app
      rustApp = rustPlatform.buildRustPackage {
        pname = "my-rust-app"; # adapt to your crate name
        version = "0.1.0";

        src = ./.;

        # first time: dummy, then replace with suggested hash from nix
        cargoHash = "sha256-wJ0YbizE98zSDhR8LOMuz3DDhh6sGIOI/Td64a9tfbc=";

        nativeBuildInputs = [
          pkgs.pkg-config
        ];

        buildInputs = [
          pkgs.openssl
        ];
      };

      # nix run wrapper using the same nightly toolchain
      runner = pkgs.writeShellApplication {
        name = "rust-app";

        runtimeInputs = [ rustToolchain ];

        text = ''
          export RUST_BACKTRACE="''${RUST_BACKTRACE-1}"
          export RUST_LOG="''${RUST_LOG-debug}"

          cargo run "$@"
        '';
      };
    in
    {
      # nix develop
      devShells.${system}.default = pkgs.mkShell {
        packages = [
          rustToolchain
          pkgs.pkg-config
          pkgs.openssl
        ];

        RUST_BACKTRACE = "1";
        RUST_LOG = "debug";
      };

      # nix build + extra runner package
      packages.${system} = {
        default = rustApp;
        runner = runner;
      };

      # nix run .#
      apps.${system}.default = {
        type = "app";
        program = "${runner}/bin/rust-app";
      };
    };
}
