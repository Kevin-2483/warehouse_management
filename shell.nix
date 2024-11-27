{ pkgs ? import <nixpkgs> {} }:

let
  mozillaOverlay = import (builtins.fetchTarball https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz);
  nixpkgs = import <nixpkgs> { overlays = [ mozillaOverlay ]; };
in
nixpkgs.mkShell {
  buildInputs = with nixpkgs; [
    # C++ 编译工具链
    gcc
    clang
    clang-tools
    cmake
    gnumake

    # Rust 工具链 (nightly)
    latest.rustChannels.nightly.rust
    latest.rustChannels.nightly.rust-src
    rust-analyzer

    # 库依赖
    sqlite
    openssl

    # 可选的额外开发工具
    pkg-config
    autoconf
    automake
  ];

  # 设置环境变量，方便编译和链接
  shellHook = ''
    echo "C++ rust 开发环境已就绪 🚀"
    echo "可用工具: cmake, gcc, clang, sqlite, openssl"
    echo "Rust toolchain: $(rustc --version)"
  '';

  # 配置 pkg-config 路径，确保库可被正确发现
  PKG_CONFIG_PATH = "${nixpkgs.sqlite.dev}/lib/pkgconfig:${nixpkgs.openssl.dev}/lib/pkgconfig";

  # 设置 Rust 相关环境变量
  RUST_BACKTRACE = 1;
  RUST_SRC_PATH = "${nixpkgs.latest.rustChannels.nightly.rust-src}/lib/rustlib/src/rust/library";
}