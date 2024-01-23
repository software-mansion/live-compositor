{ rustPlatform
, openssl
, pkg-config
, ffmpeg_6-headless
, llvmPackages_16
, libGL
, cmake
, libopus
, lib
, vulkan-loader

}:
let
  buildInputs = [
    openssl
    pkg-config
    ffmpeg_6-headless
    libopus
    libGL
    vulkan-loader
  ];
  rpath = lib.makeLibraryPath buildInputs;
in
rustPlatform.buildRustPackage {
  pname = "video_compositor";
  version = "0.2.0-rc.1";
  src = ../..;
  cargoSha256 = "sha256-KwwfdE6o3drmhzClwoBjwjXLVKYuDgq/SDG5ZGsqzCU=";
  buildNoDefaultFeatures = true;
  doCheck = false;
  inherit buildInputs;
  nativeBuildInputs = [ pkg-config llvmPackages_16.clang cmake ];
  env.LIBCLANG_PATH = "${llvmPackages_16.libclang.lib}/lib";
  postFixup = ''
    patchelf --set-rpath ${rpath} $out/bin/video_compositor
  '';
}
        
