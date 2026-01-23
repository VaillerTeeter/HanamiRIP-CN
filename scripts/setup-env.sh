#!/usr/bin/env bash
set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

install_apt_deps() {
  echo "[STEP] 安装系统依赖 (apt) ..."
  sudo apt-get update
  sudo apt-get install -y \
    build-essential \
    pkg-config \
    libgtk-3-dev \
    libwebkit2gtk-4.1-dev \
    librsvg2-dev \
    libssl-dev \
    curl \
    ca-certificates
}

install_node() {
  if command -v node >/dev/null 2>&1; then
    local version major
    version="$(node -v | sed 's/^v//')"
    major="${version%%.*}"
    if [[ "$major" == "24" ]]; then
      echo "[OK] Node $version 已安装。"
      return
    fi
  fi

  echo "[STEP] 安装 Node.js 24 ..."
  curl -fsSL https://deb.nodesource.com/setup_24.x | sudo -E bash -
  sudo apt-get install -y nodejs
}

install_yarn() {
  if command -v yarn >/dev/null 2>&1; then
    echo "[OK] yarn 已安装。"
    return
  fi

  echo "[STEP] 安装 yarn (corepack) ..."
  corepack enable
  corepack prepare yarn@1.22.22 --activate
}

install_rust() {
  if command -v cargo >/dev/null 2>&1; then
    echo "[OK] Cargo 已安装。"
    return
  fi

  echo "[STEP] 安装 Rust 工具链 ..."
  curl https://sh.rustup.rs -sSf | sh -s -- -y
  # shellcheck disable=SC1090
  . "$HOME/.cargo/env"
}

main() {
  install_apt_deps
  install_node
  install_yarn
  install_rust

  echo "[STEP] 安装项目依赖 ..."
  cd "$PROJECT_ROOT"
  yarn install
  echo "[OK] 环境与依赖安装完成。"
}

main "$@"
