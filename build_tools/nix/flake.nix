{
  description = "Dev shell for LiveCompositor";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/8db50d6f207f6e6bea072986fe5abfc955f04bfc";
    flake-parts.url = "github:hercules-ci/flake-parts/b253292d9c0a5ead9bc98c4e9a26c6312e27d69f";
  };

  outputs = inputs@{ flake-parts, ... }:
    flake-parts.lib.mkFlake { inherit inputs; } {
      systems = [ "x86_64-linux" "aarch64-linux" "aarch64-darwin" "x86_64-darwin" ];
      perSystem = { config, self', inputs', pkgs, system, lib, ... }:
        let
          packageWithoutChromium = (pkgs.callPackage ./package.nix { });
          ffmpeg =
            (if pkgs.stdenv.isDarwin then
              (pkgs.ffmpeg_6-full.override {
                x264 = pkgs.x264.overrideAttrs (old: {
                  postPatch = old.postPatch + ''
                    substituteInPlace Makefile --replace '$(if $(STRIP), $(STRIP) -x $@)' '$(if $(STRIP), $(STRIP) -S $@)'
                  '';
                });
              })
            else
              pkgs.ffmpeg_6-full
            );
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
          devDependencies = with pkgs; [
            ffmpeg # to add ffplay

            gst_all_1.gstreamer
            gst_all_1.gst-plugins-base
            gst_all_1.gst-plugins-good
            gst_all_1.gst-plugins-bad
            gst_all_1.gst-libav

            nodejs_18
            rustfmt
            clippy
            cargo-watch
            rust-analyzer
            pkgs.mesa.drivers
          ];
        in
        {
          devShells = {
            default = pkgs.mkShell {
              packages = devDependencies;

              # Fixes "ffplay" used in examples on Linux (not needed on NixOS)
              env.LIBGL_DRIVERS_PATH = "${pkgs.mesa.drivers}/lib/dri";

              env.LIBCLANG_PATH = "${pkgs.llvmPackages_16.libclang.lib}/lib";
              env.LD_LIBRARY_PATH = lib.makeLibraryPath (libcefDependencies ++ [ pkgs.mesa.drivers pkgs.libGL ]);

              inputsFrom = [ packageWithoutChromium ];
            };
          };
          packages = {
            default = packageWithoutChromium;
          };
        };
    };
}
