#!/bin/sh
set -eu

# ==============================================================================
# 설정 영역 (프로젝트에 맞게 수정하세요)
# ==============================================================================
OWNER="ppzxc"
REPO="backup"
BIN_NAME="backup"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"

# ==============================================================================
# 1. OS 및 Rust Target Triple 감지
# ==============================================================================
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
  Linux*)
    OS_TARGET="unknown-linux-musl"
    ;;
  Darwin*)
    OS_TARGET="apple-darwin"
    ;;
  MINGW*|MSYS*|CYGWIN*)
    OS_TARGET="pc-windows-msvc"
    ;;
  *)
    echo "Error: 지원하지 않는 OS입니다: $OS" >&2
    exit 1
    ;;
esac

case "$ARCH" in
  x86_64|amd64)
    ARCH_TARGET="x86_64"
    ;;
  arm64|aarch64)
    ARCH_TARGET="aarch64"
    ;;
  *)
    echo "Error: 지원하지 않는 아키텍처입니다: $ARCH" >&2
    exit 1
    ;;
esac

# Rust 타깃 트립플 조합 (예: x86_64-unknown-linux-gnu, aarch64-apple-darwin)
TARGET="${ARCH_TARGET}-${OS_TARGET}"

# ==============================================================================
# 2. 최신 버전을 GitHub API로 가져오기
# ==============================================================================
echo "Checking latest release for ${OWNER}/${REPO}..."

LATEST_RELEASE_URL="https://api.github.com/repos/${OWNER}/${REPO}/releases/latest"
TAG=$(curl -fsSL "$LATEST_RELEASE_URL" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')

if [ -z "$TAG" ]; then
  echo "Error: 최신 태그 정보를 가져올 수 없습니다." >&2
  exit 1
fi

# 압축 파일 이름 규칙 (예: my-app-v1.0.0-x86_64-unknown-linux-gnu.tar.gz)
FILE_NAME="${BIN_NAME}-${TAG}-${TARGET}.tar.gz"
DOWNLOAD_URL="https://github.com/${OWNER}/${REPO}/releases/download/${TAG}/${FILE_NAME}"

# ==============================================================================
# 3. 임시 디렉터리에 다운로드 및 압축 해제
# ==============================================================================
TMP_DIR=$(mktemp -d)
trap 'rm -rf "$TMP_DIR"' EXIT INT TERM

echo "Downloading ${FILE_NAME}..."
if ! curl -fsSL "$DOWNLOAD_URL" -o "$TMP_DIR/release.tar.gz"; then
  echo "Error: 파일 다운로드 실패. URL을 확인해 주세요: $DOWNLOAD_URL" >&2
  exit 1
fi

tar -xzf "$TMP_DIR/release.tar.gz" -C "$TMP_DIR"

# ==============================================================================
# 4. 바이너리 설치
# ==============================================================================
mkdir -p "$INSTALL_DIR"
chmod +x "$TMP_DIR/$BIN_NAME"
mv "$TMP_DIR/$BIN_NAME" "$INSTALL_DIR/$BIN_NAME"

echo " Successfully installed ${BIN_NAME} (${TAG}) to ${INSTALL_DIR}/${BIN_NAME}"

# ==============================================================================
# 5. PATH 환경변수 체크 및 안내
# ==============================================================================
case ":$PATH:" in
  *":$INSTALL_DIR:"*) ;;
  *)
    echo ""
    echo " [안내] $INSTALL_DIR 경로가 PATH에 포함되어 있지 않은 것 같습니다."
    echo "셸 설정 파일(~/.bashrc, ~/.zshrc 등)에 아래 줄을 추가해 주세요:"
    echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
    ;;
esac
