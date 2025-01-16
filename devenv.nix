{ pkgs, lib, config, inputs, ... }:

let
  pkg_swift = pkgs.swift.overrideAttrs {
      installPhase = ''
        exit 1
        ln -s $out/lib/swift/linux/Cxx.swiftmodule/x86_64-unknown-linux-gnu.swiftdoc $out/lib/swift/linux/Cxx.swiftmodule/x86_64-pc-linux-gnu.swiftdoc
      '';
    };
in
{
  packages = [
    pkgs.protobuf
    pkgs.wasm-pack
#    pkgs.swift
    pkgs.swiftpm
  ];

  cachix.enable = false;
  dotenv.enable = true;

  languages.rust = {
    enable = true;
    channel = "stable";
    # Extra targets other than the native
    targets = ["wasm32-unknown-unknown"];
  };

  languages.javascript = {
    enable = true;
    npm.enable = true;
  };

  languages.swift = {
    enable = true;
    package = pkg_swift;
  };

  languages.kotlin.enable = true;
}
