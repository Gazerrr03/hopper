#!/usr/bin/env sh
set -e

REPO="qizhidong/hopper"

# Get latest release tag
TAG=$(curl -s https://api.github.com/repos/$REPO/releases/latest | grep '"tag_name"' | sed 's/.*"v\?\([^"]*\)".*/\1/')

if [ -z "$TAG" ]; then
  echo "Error: Could not fetch latest release"
  exit 1
fi

# Detect OS
case "$(uname -s)" in
  Darwin*)  OS="apple-darwin" ;;
  MINGW*|CYGWIN*|MSYS*)  OS="pc-windows" ;;
  *)        echo "Unsupported OS: $(uname -s)"; exit 1 ;;
esac

# Detect architecture
case "$(uname -m)" in
  x86_64|amd64)  ARCH="x86_64" ;;
  aarch64|arm64) ARCH="aarch64" ;;
  *)             echo "Unsupported architecture: $(uname -m)"; exit 1 ;;
esac

BINARY="hopper-${OS}-${ARCH}.tar.gz"
URL="https://github.com/${REPO}/releases/download/${TAG}/${BINARY}"

echo "Installing hopper ${TAG} for ${OS}-${ARCH}..."

# Detect install prefix
if [ -w /usr/local/bin ]; then
  PREFIX="/usr/local/bin"
elif [ -w "$HOME/.local/bin" ] || [ -d "$HOME/.local/bin" ]; then
  PREFIX="$HOME/.local/bin"
else
  PREFIX="$HOME/.local/bin"
  mkdir -p "$PREFIX"
fi

# Download and extract
curl -fsL "$URL" | tar -xz -C "$PREFIX" hopper

echo "Installed to ${PREFIX}/hopper"
echo "Make sure ${PREFIX} is in your PATH"
