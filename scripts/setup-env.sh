#!/usr/bin/env bash
set -euo pipefail

PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

install_apt_deps() {
  echo "[INSTALL] 安装系统依赖 (apt) ..."
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
  echo "[CHECK] Node.js ..."
  if command -v node >/dev/null 2>&1; then
    local version major
    version="$(node -v | sed 's/^v//')"
    major="${version%%.*}"
    if [[ "$major" == "24" ]]; then
      echo "[OK] Node.js $version installed."
      return
    fi
  fi

  echo "[INSTALL] Node.js 24 ..."
  curl -fsSL https://deb.nodesource.com/setup_24.x | sudo -E bash -
  sudo apt-get install -y nodejs
}

install_yarn() {
  echo "[CHECK] Yarn ..."
  if command -v yarn >/dev/null 2>&1; then
    echo "[OK] Yarn $(yarn -v) installed."
    return
  fi

  echo "[INSTALL] Yarn (corepack) ..."
  corepack enable
  corepack prepare yarn@1.22.22 --activate
}

install_rust() {
  echo "[CHECK] Rust ..."
  if command -v cargo >/dev/null 2>&1; then
    echo "[OK] Rust $(rustc -v 2>/dev/null | head -n1) installed."
    return
  fi

  echo "[INSTALL] Rust toolchain ..."
  curl https://sh.rustup.rs -sSf | sh -s -- -y
  # shellcheck disable=SC1090
  . "$HOME/.cargo/env"
}

# Ensure Rust linker (gcc) for building baidu_verify on Linux (build-essential from apt_deps)
ensure_rust_linker() {
  echo "[CHECK] Rust linker (gcc) for Baidu plugin ..."
  if command -v gcc >/dev/null 2>&1 || command -v cc >/dev/null 2>&1; then
    echo "[OK] Linker (gcc/cc) found."
    return
  fi
  echo "[ERROR] gcc/cc not found. Baidu translate plugin will not build. Install build-essential and run this script again." >&2
  echo "  sudo apt-get install -y build-essential" >&2
  echo "Setup will continue; other steps are not affected." >&2
}

install_ffmpeg_tools() {
  echo "[CHECK] FFmpeg tools (ffmpeg/ffprobe) ..."
  local bin_dir="$PROJECT_ROOT/src-tauri/bin"
  mkdir -p "$bin_dir"

  if [[ -x "$bin_dir/ffmpeg" && -x "$bin_dir/ffprobe" ]]; then
    echo "[OK] FFmpeg already exists."
    return
  fi

  echo "[INSTALL] Download FFmpeg ..."

  local temp_dir
  temp_dir="$(mktemp -d)"
  local url="https://johnvansickle.com/ffmpeg/releases/ffmpeg-release-amd64-static.tar.xz"
  echo "[INFO] Download: $url"
  curl -fsSL "$url" -o "$temp_dir/ffmpeg.tar.xz"
  tar -xf "$temp_dir/ffmpeg.tar.xz" -C "$temp_dir"
  local extracted
  extracted="$(find "$temp_dir" -maxdepth 1 -type d -name "ffmpeg-*" | head -n 1)"
  if [[ -z "$extracted" ]]; then
    echo "[ERROR] 解压 FFmpeg 失败" >&2
    rm -rf "$temp_dir"
    exit 1
  fi
  cp "$extracted/ffmpeg" "$bin_dir/ffmpeg"
  cp "$extracted/ffprobe" "$bin_dir/ffprobe"
  chmod +x "$bin_dir/ffmpeg" "$bin_dir/ffprobe"
  rm -rf "$temp_dir"
  echo "[OK] FFmpeg download done."
}

install_mkvtoolnix_tools() {
  echo "[CHECK] MKVToolNix tools (mkvmerge/mkvinfo) ..."
  local bin_dir="$PROJECT_ROOT/src-tauri/bin"
  mkdir -p "$bin_dir"

  if [[ -x "$bin_dir/mkvmerge" && -x "$bin_dir/mkvinfo" ]]; then
    echo "[OK] MKVToolNix already exists."
    return
  fi

  echo "[INSTALL] Download MKVToolNix ..."

  local temp_dir
  temp_dir="$(mktemp -d)"
  local appimage="$temp_dir/mkvtoolnix.appimage"
  local index_url="https://mkvtoolnix.download/linux/releases/"
  local index latest base_url url

  index="$(curl -fsSL "$index_url")"
  latest="$(echo "$index" | grep -oE '/linux/releases/[0-9]+(\.[0-9]+)*/' | sed -E 's#.*/([0-9.]+)/#\1#' | sort -V | tail -n 1)"

  if [[ -z "$latest" ]]; then
    echo "[ERROR] 无法解析 MKVToolNix 版本列表" >&2
    rm -rf "$temp_dir"
    exit 1
  fi

  base_url="https://mkvtoolnix.download/linux/releases/${latest}/"
  for name in "mkvtoolnix-${latest}-x86_64.AppImage" "mkvtoolnix-${latest}.0-x86_64.AppImage" "mkvtoolnix-${latest}.0.0-x86_64.AppImage"; do
    if curl -fsSL "${base_url}${name}" -o "$appimage"; then
      url="${base_url}${name}"
      break
    fi
  done

  if [[ -z "${url:-}" ]]; then
    echo "[ERROR] 未找到可用的 MKVToolNix AppImage" >&2
    echo "下载列表：${index_url}" >&2
    rm -rf "$temp_dir"
    exit 1
  fi

  echo "[INFO] Download: $url"
  chmod +x "$appimage"

  "$appimage" --appimage-extract >/dev/null
  if [[ ! -x "squashfs-root/usr/bin/mkvmerge" || ! -x "squashfs-root/usr/bin/mkvinfo" ]]; then
    echo "[ERROR] 解压 MKVToolNix 失败" >&2
    rm -rf "$temp_dir" "squashfs-root"
    exit 1
  fi

  cp "squashfs-root/usr/bin/mkvmerge" "$bin_dir/mkvmerge"
  cp "squashfs-root/usr/bin/mkvinfo" "$bin_dir/mkvinfo"
  chmod +x "$bin_dir/mkvmerge" "$bin_dir/mkvinfo"
  rm -rf "$temp_dir" "squashfs-root"
  echo "[OK] MKVToolNix download done."
}

main() {
  echo ""
  echo "=== HanamiRIP-CN Linux Environment Setup ==="
  echo ""
  install_apt_deps
  install_node
  install_yarn
  install_rust
  ensure_rust_linker
  install_ffmpeg_tools
  install_mkvtoolnix_tools

  echo "[INSTALL] Project dependencies ..."
  cd "$PROJECT_ROOT"
  yarn install
  echo ""
  echo "=== Environment Setup Complete! ==="
  echo ""
  echo "Available commands:"
  echo "  yarn tauri dev    # Start Tauri desktop app in dev mode"
  echo "  yarn dev          # Start Vite frontend server only"
  echo "  yarn build        # Build desktop application"
  echo ""
  echo "To enable Baidu Translation: set BAIDU_TRANSLATE_APP_ID and BAIDU_TRANSLATE_API_KEY, then: yarn run build:baidu-so:linux"
  echo ""
}

main "$@"
