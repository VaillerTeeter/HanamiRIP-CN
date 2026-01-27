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

install_ffmpeg_tools() {
  echo "[STEP] 安装内置 FFmpeg (ffmpeg/ffprobe) ..."
  local bin_dir="$PROJECT_ROOT/src-tauri/bin"
  mkdir -p "$bin_dir"

  if [[ -x "$bin_dir/ffmpeg" && -x "$bin_dir/ffprobe" ]]; then
    echo "[OK] 内置 FFmpeg 已存在。"
    return
  fi

  local temp_dir
  temp_dir="$(mktemp -d)"
  local url="https://johnvansickle.com/ffmpeg/releases/ffmpeg-release-amd64-static.tar.xz"
  echo "[INFO] 下载: $url"
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
  echo "[OK] 内置 FFmpeg 下载完成。"
}

install_mkvtoolnix_tools() {
  echo "[STEP] 安装内置 MKVToolNix (mkvmerge/mkvinfo) ..."
  local bin_dir="$PROJECT_ROOT/src-tauri/bin"
  mkdir -p "$bin_dir"

  if [[ -x "$bin_dir/mkvmerge" && -x "$bin_dir/mkvinfo" ]]; then
    echo "[OK] 内置 MKVToolNix 已存在。"
    return
  fi

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

  echo "[INFO] 下载: $url"
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
  echo "[OK] 内置 MKVToolNix 下载完成。"
}

main() {
  install_apt_deps
  install_node
  install_yarn
  install_rust
  install_ffmpeg_tools
  install_mkvtoolnix_tools

  echo "[STEP] 安装项目依赖 ..."
  cd "$PROJECT_ROOT"
  yarn install
  echo "[OK] 环境与依赖安装完成。"
}

main "$@"
