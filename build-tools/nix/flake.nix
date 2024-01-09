{
  description = "Dev shell for LiveCompositor";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs = inputs@{ flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [ "x86_64-linux" "aarch64-linux" "aarch64-darwin" "x86_64-darwin" ];
      perSystem = { config, self', inputs', pkgs, system, lib, ... }:
        {
          devShells = {
            name = "live_compositor";
            default = pkgs.mkShell {
              packages = with pkgs; [
                cargo
                rustc
                nodejs_18

                pkg-config
                openssl
                ffmpeg_6-full
                libcef
                llvmPackages.bintools
                llvmPackages.clang
              ];
              env.LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";
              env.LD_LIBRARY_PATH = lib.makeLibraryPath (with pkgs; [
                  libGL
                  
                  # https://github.com/NixOS/nixpkgs/blob/master/pkgs/development/libraries/libcef/default.nix#L33
                  glib
                  nss
                  nspr
                  atk
                  at-spi2-atk
                  libdrm
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
                  alsa-lib
                  dbus
                  at-spi2-core
                  cups
                  xorg.libxshmfence
              ]);
              inputsFrom = [ ];
            };
          };
        };
    };
}
