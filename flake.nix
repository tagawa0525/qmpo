{
  description = "qmpo - directory:// URI handler for opening directories in your file manager";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

    # devShell の Rust ツールチェーンを flake.lock で固定するために使用する。
    # rustc / cargo / rustPlatform は上書きしないため、packages.qmpo のビルドには
    # 影響しない（nixpkgs 側の rustPlatform をそのまま使う）。
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    {
      self,
      nixpkgs,
      rust-overlay,
    }:
    let
      supportedSystems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];

      forAllSystems = nixpkgs.lib.genAttrs supportedSystems;

      nixpkgsFor = forAllSystems (
        system:
        import nixpkgs {
          inherit system;
          overlays = [
            self.overlays.default
            (import rust-overlay)
          ];
        }
      );
    in
    {
      # Overlay for use in other flakes
      overlays.default = final: prev: {
        qmpo = final.callPackage ./nix/package.nix { };
      };

      # Packages
      packages = forAllSystems (system: {
        qmpo = nixpkgsFor.${system}.qmpo;
        default = self.packages.${system}.qmpo;
      });

      # Home Manager module
      homeManagerModules.default = import ./nix/home-manager.nix;

      # Development shell
      devShells = forAllSystems (
        system:
        let
          pkgs = nixpkgsFor.${system};

          # default プロファイルに rustc / cargo / rustfmt / clippy が含まれる。
          # rust-src と rust-analyzer は rust-analyzer の補完・定義ジャンプに必要。
          rustToolchain = pkgs.rust-bin.stable.latest.default.override {
            extensions = [
              "rust-src"
              "rust-analyzer"
            ];
          };
        in
        {
          default = pkgs.mkShell {
            buildInputs = [
              rustToolchain
              pkgs.pkg-config
            ];
          };
        }
      );
    };
}
