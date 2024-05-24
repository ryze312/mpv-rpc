{
  inputs = {
    nixpkgs.url = "nixpkgs/nixpkgs-unstable";

    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs = { self, nixpkgs, fenix }:
  let
    overlays = [ fenix.overlays.default ];

    getPkgsFor = (system: import nixpkgs {
      inherit system overlays;
    });

    forEachSystem = func: nixpkgs.lib.genAttrs [
      "x86_64-linux"
      "aarch64-linux"

      "aarch64-darwin"
      "x86_64-darwin"
    ] (system: func (getPkgsFor system));

  in {
    devShells = forEachSystem (pkgs: {
      default = pkgs.callPackage ./shell.nix {};
    });
  };
}
