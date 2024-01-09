{
  description = "Dev shell for LiveCompositor";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
  };

  outputs = inputs@{ flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [ "x86_64-linux" "aarch64-linux" "aarch64-darwin" "x86_64-darwin" ];
      perSystem = { config, self', inputs', pkgs, system, lib, ... }:
        let
          maybeDarwinPkgs = with pkgs; lib.optionals stdenv.isDarwin [
            darwin.apple_sdk.frameworks.Metal
            darwin.apple_sdk.frameworks.Foundation
            darwin.apple_sdk.frameworks.QuartzCore
            darwin.libobjc

            libiconv
          ];
          maybeLinuxPkgs = with pkgs; lib.optionals stdenv.isLinux [
            vulkan-tools
            vulkan-headers
            vulkan-loader
            vulkan-validation-layers
            wayland
            mesa.drivers
          ];
          # https://github.com/NixOS/nixpkgs/blob/master/pkgs/development/libraries/libcef/default.nix#L33
          libcefDependencies = with pkgs;  [
            glib
            nss
            nspr
            atk
            at-spi2-atk
            expat
            xorg.libxcb
            libxkbcommon
            xorg.libX11
            xorg.libXcomposite
            xorg.libXdamage
            xorg.libXext
            xorg.libXfixes
            xorg.libXrandr
            mesa
            gtk3
            pango
            cairo
            dbus
            at-spi2-core
            cups
            xorg.libxshmfence
          ] ++ (
            pkgs.lib.optionals pkgs.stdenv.isLinux [
              libdrm
              alsa-lib
            ]
          );
          compositorBuildDependencies = with pkgs; [
            cargo
            rustc
            libGL
            libopus

            pkg-config
            openssl
            ffmpeg_6-full

            llvmPackages_16.clang
          ] ++ maybeDarwinPkgs ++ maybeLinuxPkgs;
          devDependencies = with pkgs; [
            nodejs_18
            rustfmt
            clippy
            rust-analyzer
          ];
        in
        {
          devShells = {
            default = pkgs.mkShell {
              packages = compositorBuildDependencies ++ devDependencies;

              # Fixes "ffplay" used in examples on Linux (not needed on NixOS)
              env.LIBGL_DRIVERS_PATH = "${pkgs.mesa.drivers}/lib/dri";

              env.LIBCLANG_PATH = "${pkgs.llvmPackages_16.libclang.lib}/lib";
              env.LD_LIBRARY_PATH = lib.makeLibraryPath (maybeDarwinPkgs ++ maybeLinuxPkgs ++ libcefDependencies ++ [ pkgs.libGL ]);
            };
          };
        };
    };
}
