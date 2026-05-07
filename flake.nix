{
  description = "DevShell for Rust development with Wayland and Vulkan (vulkano)";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
        };

      in {
        devShells.default = pkgs.mkShell {
          name = "rust-wayland-vulkan-shell";

          buildInputs = with pkgs; [
            rust-bin.stable.latest.default
            pkg-config
            cmake
            cmake
            llvmPackages_21.libclang
            clang
            glibc.dev
          ];

          shellHook = ''
            export BINDGEN_EXTRA_CLANG_ARGS="-I$XKBCOMMON_INCLUDE_DIR"
            export LIBCLANG_PATH="${pkgs.llvmPackages_21.libclang.lib}/lib"
            export C_INCLUDE_PATH=${pkgs.glibc.dev}/include
            export LIBRARY_PATH=${pkgs.glibc.dev}/lib
          '';
        };
      }
    );
}

