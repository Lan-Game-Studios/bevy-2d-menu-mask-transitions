let
  pkgs = import <nixpkgs> { };
in
pkgs.mkShell {
  packages = with pkgs; [
    cargo
    cargo-tarpaulin
    cargo-nextest
    rustc
    rust-analyzer
    rustfmt
    clippy
    gcc
    alsa-lib
    dbus
    pkg-config
    udev
    wayland
    libxkbcommon
    mold
  ];

  LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [
    # stdenv.cc.cc
    pkgs.libxkbcommon
    pkgs.vulkan-loader
    pkgs.wayland
    pkgs.udev
    pkgs.alsaLib
  ];

  env = {
    RUST_BACKTRACE = "full";
    WINIT_UNIX_BACKEND = "wayland";
  };
}
