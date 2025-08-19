{ pkgs ? import <nixpkgs> { } }:

pkgs.mkShell rec {
  buildInputs = with pkgs; [
    # necessary for building wgpu in 3rd party packages (in most cases)
    libxkbcommon
    xorg.libX11
    xorg.libXcursor
    xorg.libXrandr
    xorg.libXi
    alsa-lib
    fontconfig
    freetype
    shaderc
    directx-shader-compiler
    pkg-config
    cmake
    mold # could use any linker, needed for rustix (but mold is fast)

    libGL
    vulkan-headers
    vulkan-loader
    vulkan-tools
    vulkan-tools-lunarg
    vulkan-extension-layer
    vulkan-validation-layers # don't need them *strictly* but immensely helpful

    # necessary for developing (all of) wgpu itself
    cargo-nextest
    cargo-fuzz

    # nice for developing wgpu itself
    typos

    # nice tools
    gdb
    rr
    evcxr
    valgrind
  ];

  shellHook = ''
    export RUSTC_VERSION="$(tomlq -r .toolchain.channel rust-toolchain.toml)"
    export PATH="$PATH:''${CARGO_HOME:-~/.cargo}/bin"
    export PATH="$PATH:''${RUSTUP_HOME:-~/.rustup/toolchains/$RUSTC_VERSION-x86_64-unknown-linux/bin}"
    export LD_LIBRARY_PATH="$LD_LIBRARY_PATH:${builtins.toString (pkgs.lib.makeLibraryPath buildInputs)}";

    rustup default $RUSTC_VERSION
    rustup component add rust-src rust-analyzer
  '';
}
