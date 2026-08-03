#!/usr/bin/env bash
# ─────────────────────────────────────────────────────────────────────────────
# jobsense-parker installer — macOS / Linux
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/arpanpathak/jobsense-parker/master/install.sh | bash
#   curl -fsSL .../install.sh | bash -s -- v0.3.0   # pin a specific version
#
# Downloads the prebuilt binary for your OS/arch from the GitHub Release,
# installs it to ~/.local/bin (or $JOBSENSE_INSTALL_DIR), and prints PATH help.
# ─────────────────────────────────────────────────────────────────────────────
set -euo pipefail

REPO="arpanpathak/jobsense-parker"
VERSION="${1:-latest}"
INSTALL_DIR="${JOBSENSE_INSTALL_DIR:-$HOME/.local/bin}"

# ── Detect OS + arch ────────────────────────────────────────────────────────
case "$(uname -s)" in
  Darwin) TARGET_OS="apple-darwin" ;;
  Linux)  TARGET_OS="unknown-linux-gnu" ;;
  *)
    echo "✗ Unsupported OS: $(uname -s). Windows users: run install.ps1 instead."
    exit 1
    ;;
esac

case "$(uname -m)" in
  x86_64 | amd64) TARGET_ARCH="x86_64" ;;
  arm64 | aarch64) TARGET_ARCH="aarch64" ;;
  *)
    echo "✗ Unsupported architecture: $(uname -m)"
    exit 1
    ;;
esac

# ── Resolve latest version if not pinned ────────────────────────────────────
# ── Resolve download URL ─────────────────────────────────────────────────────
# GitHub's `/releases/latest/download/<asset>` redirect serves the newest
# release's asset without any API call (the API is rate-limited when
# unauthenticated and can return 403).
ASSET="jobsense-parker-$TARGET_ARCH-$TARGET_OS.tar.gz"
if [ "$VERSION" = "latest" ]; then
  URL="https://github.com/$REPO/releases/latest/download/$ASSET"
else
  URL="https://github.com/$REPO/releases/download/$VERSION/$ASSET"
fi
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

# ── Download + extract ──────────────────────────────────────────────────────
echo "→ Downloading $ASSET ($VERSION)"
curl -fsSL "$URL" -o "$TMP_DIR/$ASSET"
tar -xzf "$TMP_DIR/$ASSET" -C "$TMP_DIR"

mkdir -p "$INSTALL_DIR"
install -m 755 "$TMP_DIR/jobsense-parker" "$INSTALL_DIR/jobsense-parker"

# ── Done ────────────────────────────────────────────────────────────────────
echo "✓ Installed jobsense-parker ($VERSION) to $INSTALL_DIR"

case ":$PATH:" in
  *":$INSTALL_DIR:"*) ;;
  *)
    echo
    echo "  Add it to your PATH, e.g. for your shell:"
    case "${SHELL##*/}" in
      zsh) echo '  echo '\''export PATH="$HOME/.local/bin:$PATH"'\'' >> ~/.zshrc' ;;
      bash) echo '  echo '\''export PATH="$HOME/.local/bin:$PATH"'\'' >> ~/.bashrc' ;;
      *) echo "  export PATH=\"$INSTALL_DIR:\$PATH\"" ;;
    esac
    ;;
esac

echo
echo "  Run 'jobsense-parker' to start hunting."
