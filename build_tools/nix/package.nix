{ rustPlatform
, ffmpeg_7-headless
, openssl
, pkg-config
, llvmPackages_16
, libGL
, cmake
, libopus
, lib
, vulkan-loader
, mesa
, darwin
, stdenv
, makeWrapper
}:
let
  buildInputs = [
    ffmpeg_7-headless
    openssl
    libopus
    libGL
    vulkan-loader
  ] ++ lib.optionals stdenv.isDarwin [
    darwin.apple_sdk.frameworks.Metal
    darwin.apple_sdk.frameworks.Foundation
    darwin.apple_sdk.frameworks.QuartzCore
    darwin.libobjc
  ] ++ lib.optionals stdenv.isLinux [
    mesa.drivers
  ];
  rpath = lib.makeLibraryPath buildInputs;
in
rustPlatform.buildRustPackage {
  pname = "live_compositor";
  version = "0.3.0";
  src = ../..;
  cargoLock = {
    lockFile = ../../Cargo.lock;
    allowBuiltinFetchGit = true;
  };

  buildNoDefaultFeatures = true;
  doCheck = false;

  inherit buildInputs;
  nativeBuildInputs = [ pkg-config llvmPackages_16.clang cmake makeWrapper ];

  env.LIBCLANG_PATH = "${llvmPackages_16.libclang.lib}/lib";

  postFixup =
    ''
      rm -f $out/bin/live_compositor
      rm -f $out/bin/package_for_release
      rm -f $out/bin/update_snapshots

      mv $out/bin/main_process $out/bin/live_compositor
    '' + (
      lib.optionalString stdenv.isLinux ''
        patchelf --set-rpath ${rpath} $out/bin/live_compositor
        wrapProgram $out/bin/live_compositor \
        --prefix XDG_DATA_DIRS : "${mesa.drivers}/share" \
        --prefix LD_LIBRARY_PATH : "${mesa.drivers}/lib"
      ''
    );
}

