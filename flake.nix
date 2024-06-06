{
  description = "Rust flake";
  inputs =
    {
      nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable"; # or whatever vers
    };
  
  outputs = { self, nixpkgs, ... }@inputs:
    let
     system = "x86_64-linux"; # your version
     pkgs = nixpkgs.legacyPackages.${system};    
    in
    {
      devShells.${system}.default = pkgs.mkShell
      {
        packages = with pkgs; [
            rustc
            cargo
            pkg-config
        ]; # whatever you need
        nativeBuildInputs = with pkgs; [
            openssl
            alsa-lib
            libxkbcommon
            libGL
            wayland
        ];
        LD_LIBRARY_PATH = "$LD_LIBRARY_PATH:${ with pkgs; lib.makeLibraryPath [
           wayland
           libxkbcommon
           libGL
        ] }";
      };
    };
}
