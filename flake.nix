{
  description = "bevy_2d_menu_mask_transition dev shell";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs =
    {
      nixpkgs,
      ...
    }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs { inherit system; };

      runtimeDeps = with pkgs; [
        alsa-lib
        udev
        vulkan-loader
        wayland
        libxkbcommon
        libx11
        libxcursor
        libxi
        libxrandr
        libxcb
        libGL
      ];
    in
    {
      devShells.${system}.default = pkgs.mkShell {
        packages =
          with pkgs;
          [
            cargo
            cargo-nextest
            cargo-tarpaulin
            rustc
            rust-analyzer
            rustfmt
            clippy
            gcc
            pkg-config
            mold
            just
          ]
          ++ runtimeDeps;

        LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath runtimeDeps;

        env = {
          RUST_BACKTRACE = "full";
          WINIT_UNIX_BACKEND = "wayland";
        };
      };
    };
}
