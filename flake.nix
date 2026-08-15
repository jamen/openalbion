{
  description = "OpenAlbion — a recreation of Fable: The Lost Chapter's game engine";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    { nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
        };

        # winit/wgpu pick their backend per target OS at compile time (Metal + AppKit
        # on Darwin, Vulkan/GL + Wayland/X11 on Linux) — no Cargo feature wiring needed
        # here. The Linux windowing/graphics libs below are dlopen'd at runtime, not
        # linked, so they only need to be on LD_LIBRARY_PATH, and only on Linux.
        linuxRuntimeLibs = with pkgs; [
          wayland
          libxkbcommon
          vulkan-loader
          mesa
          libx11
          libxcursor
          libxrandr
          libxi
          libxcb
        ];

        # Windows cross-compilation via mingw-w64, not cargo-xwin: winit and wgpu's
        # DX12 backend both go through `windows-rs`, which ships prebuilt import libs
        # for the `-gnu` ABI as well as `-msvc` (see AGENTS.md discussion), and wgpu's
        # Vulkan backend dlopens vulkan-1.dll at runtime rather than link-time, so
        # nothing here needs the proprietary MSVC SDK/CRT.
        mingw = pkgs.pkgsCross.mingwW64;
        mingwTarget = "x86_64-pc-windows-gnu";
        mingwBin = "${mingw.stdenv.cc}/bin/${mingw.stdenv.cc.targetPrefix}";
        # rustc's windows-gnu target spec unconditionally requests `-l:libpthread.a`
        # (a leftover from std's old pthread-based TLS destructors); rust-overlay's
        # target component doesn't vendor rustup's "self-contained" mingw import libs
        # that normally supply it, so it has to come from the cross toolchain instead.
        mingwPthreads = mingw.windows.pthreads;
      in
      {
        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            (rust-bin.stable.latest.default.override {
              extensions = [
                "rust-src"
                "rust-analyzer"
              ];
              targets = [ mingwTarget ];
            })
            # `mingw.stdenv.cc` deliberately does NOT go in `packages`. Its cc-wrapper setup
            # hook exports a bare `CC`/`AR` for its own target, which the `cc` crate then uses
            # to build `packages/lzo`'s minilzo — producing a *Windows COFF* object that the
            # Linux host link cannot resolve (`undefined symbol: lzo1x_decompress_safe`).
            # `cargo check` never links, so the breakage hides. Everything the cross build
            # needs is exported below by absolute store path, so it need not be on PATH.
          ];

          shellHook = ''
            ${pkgs.lib.optionalString pkgs.stdenv.isLinux ''
              export LD_LIBRARY_PATH=${
                pkgs.lib.makeLibraryPath linuxRuntimeLibs
              }''${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH}
            ''}
            export CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER=${mingwBin}gcc
            export CC_x86_64_pc_windows_gnu=${mingwBin}gcc
            export AR_x86_64_pc_windows_gnu=${mingwBin}ar
            export CARGO_TARGET_X86_64_PC_WINDOWS_GNU_RUSTFLAGS="-L ${mingwPthreads}/lib"
          '';
        };
      }
    );
}
