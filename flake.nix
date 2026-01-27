{
  description = "qmpo - directory:// URI handler for opening directories in your file manager";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs }:
    let
      supportedSystems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];

      forAllSystems = nixpkgs.lib.genAttrs supportedSystems;

      nixpkgsFor = forAllSystems (system: import nixpkgs {
        inherit system;
        overlays = [ self.overlays.default ];
      });
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
      devShells = forAllSystems (system:
        let
          pkgs = nixpkgsFor.${system};
        in
        {
          default = pkgs.mkShell {
            buildInputs = with pkgs; [
              rustup
              pkg-config
            ];
          };
        });
    };
}
