{}:
let
  mozillaOverlay = import (builtins.fetchTarball https://github.com/mozilla/nixpkgs-mozilla/archive/master.tar.gz);
  nixpkgs = import (builtins.fetchTarball {
    url = "https://github.com/NixOS/nixpkgs/archive/nixos-24.05.tar.gz";
  }) {
    overlays = [ mozillaOverlay ];
  };
in
{
  # 哪个 nixpkgs 版本
  channel = "unstable";

  # 安装的工具
  packages = with nixpkgs; [

    cmake
    glibc
    # Rust 工具链 (nightly)
    latest.rustChannels.nightly.rust
    latest.rustChannels.nightly.rust-src
    rust-analyzer

    # 库依赖
    sqlite
    openssl
  ];

  # 设置环境变量
  env = {
    # 配置 pkg-config 路径，确保库可被正确发现
    PKG_CONFIG_PATH = "${nixpkgs.sqlite.dev}/lib/pkgconfig:${nixpkgs.openssl.dev}/lib/pkgconfig";
    LIBC_PATH="${nixpkgs.glibc}/lib";
    LD_LIBRARY_PATH="${nixpkgs.glibc}/lib:$LD_LIBRARY_PATH";
    # 设置 Rust 相关环境变量
    RUST_BACKTRACE = 1;
    RUST_SRC_PATH = "${nixpkgs.latest.rustChannels.nightly.rust-src}/lib/rustlib/src/rust/library";
  };

  idx = {
    extensions = [
      # "vscodevim.vim"
    ];

    previews = {
      enable = true;
      previews = {
        # 例子: 启用 web 预览
        # web = {
        #   command = ["npm" "run" "dev"];
        #   manager = "web";
        #   env = { PORT = "$PORT"; };
        # };
      };
    };

    workspace = {
      onCreate = {
        # npm-install = "npm install";
      };
      onStart = {
        output = ''
          echo "C++ rust 开发环境已就绪 🚀"
          echo "可用工具: cmake, gcc, clang, sqlite, openssl"
          echo "Rust toolchain: $(rustc --version)"
        '';
      };
    };
  };
}
