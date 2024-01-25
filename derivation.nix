{ pkgs, stdenv }:
pkgs.rustPlatform.buildRustPackage rec {
    pname = "file-watcher-${version}";
    version = "1.0";

    src = ./.;

    nativeBuildInputs = [ pkgs.pkg-config ];

    cargoLock.lockFile = ./Cargo.lock;

    PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";

    doCheck = false;
}

