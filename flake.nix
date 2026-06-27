{
  description = "OpenAlbion — a recreation of Fable: The Lost Chapter's game engine";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs =
    { nixpkgs, rust-overlay, ... }:
    let
      system = "x86_64-linux";
      overlays = [ (import rust-overlay) ];
      pkgs = import nixpkgs {
        inherit system overlays;
      };
    in
    {
      devShells.${system}.default = pkgs.mkShell {
        buildInputs = with pkgs; [
          (rust-bin.stable.latest.default.override {
            extensions = [
              "rust-src"
              "rust-analyzer"
            ];
          })
          wayland
          libxkbcommon
          vulkan-loader
          mesa
        ];
        shellHook = ''
          export LD_LIBRARY_PATH=${
            pkgs.lib.makeLibraryPath [ pkgs.wayland pkgs.libxkbcommon pkgs.vulkan-loader pkgs.mesa ]
          }''${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}
        '';
      };
    };
}
