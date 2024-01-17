{ outputs = { self, nixpkgs }: let
  arch = "x86_64-linux";
  pkgs = import nixpkgs { system = arch; };
  in with pkgs; {
    packages.${arch}.default = cargo;
    devShells.${arch}.default = mkShell {
      env.LIBCLANG_PATH = "${libclang.lib}/lib";
      buildInputs = with pkgs; [ pkg-config openssl ffmpeg libclang.lib ];
      packages = [ cargo ]; };
  };
}
