{
  description = "Rust dev env with rustc >= 1.78";

  inputs = {
    # reasonably recent nixpkgs
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-24.11";

    # oxalica rust overlay for pinned Rust toolchains
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
      system = "x86_64-linux"; # adjust if needed

      pkgs = import nixpkgs {
        inherit system;
        overlays = [ (import rust-overlay) ];
      };

      # 🔧 choose your Rust version here
      # this guarantees at least 1.78
      # rustToolchain = pkgs.rust-bin.stable."1.78.0".default;
      # for a higher version later, just change to e.g.:
      # rustToolchain = pkgs.rust-bin.stable."1.79.0".default;
      # or always-latest stable:
      rustToolchain = pkgs.rust-bin.stable.latest.default;
    in
    {
      devShells.${system}.default = pkgs.mkShell {
        packages = [
          rustToolchain
          pkgs.pkg-config
          pkgs.openssl # common dependency for many crates
        ];

        # optional nice defaults
        RUST_BACKTRACE = "1";
      };
    };
}
