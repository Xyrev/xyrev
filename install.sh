#!/usr/bin/env bash
set -e

echo "Installing Xyrev CLI..."

# Detect platform
case "$(uname -s || echo "")" in
  Darwin*) PLATFORM="darwin" ;;
  Linux*) PLATFORM="linux" ;;
  *)
    if command -v winget >/dev/null 2>&1; then
      echo "Windows detected. Installing via winget..."
      winget install Xyrev.xyrev
      exit $?
    else
      echo "Error: Windows detected but winget not found." >&2
      exit 1
    fi
    ;;
esac

# Detect architecture
case "$(uname -m)" in
  x86_64|amd64) ARCH="x64" ;;
  aarch64|arm64) ARCH="arm64" ;;
  *) echo "Error: Unsupported architecture $(uname -m)" >&2 ; exit 1 ;;
esac

# Determine download URL based on VERSION
# if [ "${VERSION}" = "latest" ] || [ -z "$VERSION" ]; then
#   DOWNLOAD_URL="https://github.com/xyrev/xyrev/releases/latest/download/xyrev-${PLATFORM}-${ARCH}.tar.gz"
#   CHECKSUMS_URL="https://github.com/xyrev/xyrev/releases/latest/download/SHA256SUMS.txt"
# elif [ "${VERSION}" = "prerelease" ]; then
#   # Get the latest prerelease tag
#   if ! command -v git >/dev/null 2>&1; then
#     echo "Error: git is required to install prerelease versions" >&2
#     exit 1
#   fi
#   VERSION="$(git ls-remote --tags https://github.com/xyrev/xyrev | tail -1 | awk -F/ '{print $NF}')"
#   if [ -z "$VERSION" ]; then
#     echo "Error: Could not determine prerelease version" >&2
#     exit 1
#   fi
#   echo "Latest prerelease version: $VERSION"
#   DOWNLOAD_URL="https://github.com/xyrev/xyrev/releases/download/${VERSION}/xyrev-${PLATFORM}-${ARCH}.tar.gz"
#   CHECKSUMS_URL="https://github.com/xyrev/xyrev/releases/download/${VERSION}/SHA256SUMS.txt"
# else
#   # Prefix version with 'v' if not already present
#   case "$VERSION" in
#     v*) ;;
#     *) VERSION="v$VERSION" ;;
#   esac
#   DOWNLOAD_URL="https://github.com/xyrev/xyrev/releases/download/${VERSION}/xyrev-${PLATFORM}-${ARCH}.tar.gz"
#   CHECKSUMS_URL="https://github.com/xyrev/xyrev/releases/download/${VERSION}/SHA256SUMS.txt"
# fi
DOWNLOAD_URL="https://github.com/xyrev/xyrev/releases/download/v0.2.3/optimise-v0.2.3-aarch64-apple-darwin.tar.gz"
echo "Downloading from: $DOWNLOAD_URL"

# Download and extract with error handling
TMP_DIR="$(mktemp -d)"
TMP_TARBALL="$TMP_DIR/xyrev-${PLATFORM}-${ARCH}.tar.gz"
echo "$TMP_DIR"

if command -v curl >/dev/null 2>&1; then
  curl -fsSL "$DOWNLOAD_URL" -o "$TMP_TARBALL"
elif command -v wget >/dev/null 2>&1; then
  wget -qO "$TMP_TARBALL" "$DOWNLOAD_URL"
else
  echo "Error: Neither curl nor wget found. Please install one of them."
  rm -rf "$TMP_DIR"
  exit 1
fi

# # Attempt to download checksums file and validate
# TMP_CHECKSUMS="$TMP_DIR/SHA256SUMS.txt"
# CHECKSUMS_AVAILABLE=false
# if command -v curl >/dev/null 2>&1; then
#   curl -fsSL "$CHECKSUMS_URL" -o "$TMP_CHECKSUMS" 2>/dev/null && CHECKSUMS_AVAILABLE=true
# elif command -v wget >/dev/null 2>&1; then
#   wget -qO "$TMP_CHECKSUMS" "$CHECKSUMS_URL" 2>/dev/null && CHECKSUMS_AVAILABLE=true
# fi

# if [ "$CHECKSUMS_AVAILABLE" = true ]; then
#   if command -v sha256sum >/dev/null 2>&1; then
#     if (cd "$TMP_DIR" && sha256sum -c --ignore-missing SHA256SUMS.txt >/dev/null 2>&1); then
#       echo "✓ Checksum validated"
#     else
#       echo "Error: Checksum validation failed." >&2
#       rm -rf "$TMP_DIR"
#       exit 1
#     fi
#   elif command -v shasum >/dev/null 2>&1; then
#     if (cd "$TMP_DIR" && shasum -a 256 -c --ignore-missing SHA256SUMS.txt >/dev/null 2>&1); then
#       echo "✓ Checksum validated"
#     else
#       echo "Error: Checksum validation failed." >&2
#       rm -rf "$TMP_DIR"
#       exit 1
#     fi
#   else
#     echo "Warning: No sha256sum or shasum found, skipping checksum validation."
#   fi
# fi

# # Check that the file is a valid tarball
# if ! tar -tzf "$TMP_TARBALL" >/dev/null 2>&1; then
#   echo "Error: Downloaded file is not a valid tarball or is corrupted." >&2
#   rm -rf "$TMP_DIR"
#   exit 1
# fi

# INSTALL_DIR="$PREFIX/bin"
# if ! mkdir -p "$INSTALL_DIR"; then
#   echo "Error: Could not create directory $INSTALL_DIR. You may not have write permissions." >&2
#   echo "Try running this script with sudo or set PREFIX to a directory you own (e.g., export PREFIX=\$HOME/.local)." >&2
#   exit 1
# fi

# # Install binary
# if [ -f "$INSTALL_DIR/xyrev" ]; then
#   echo "Notice: Replacing xyrev binary found at $INSTALL_DIR/xyrev."
# fi
# tar -xz -C "$INSTALL_DIR" -f "$TMP_TARBALL"
# chmod +x "$INSTALL_DIR/xyrev"
# echo "✓ Xyrev CLI installed to $INSTALL_DIR/xyrev"
# rm -rf "$TMP_DIR"

# # Check if installed binary is accessible
# if ! command -v xyrev >/dev/null 2>&1; then
#   echo ""
#   echo "Notice: $INSTALL_DIR is not in your PATH"

#   # Detect shell rc file
#   case "$(basename "${SHELL:-/bin/sh}")" in
#     zsh)  RC_FILE="$HOME/.zshrc" ;;
#     bash) RC_FILE="$HOME/.bashrc" ;;
#     *)    RC_FILE="$HOME/.profile" ;;
#   esac

#   # Prompt user to add to shell rc file (only if interactive)
#   if [ -t 0 ] || [ -e /dev/tty ]; then
#     echo ""
#     printf "Would you like to add it to %s? [y/N] " "$RC_FILE"
#     if read -r REPLY </dev/tty 2>/dev/null; then
#       if [ "$REPLY" = "y" ] || [ "$REPLY" = "Y" ]; then
#         echo "export PATH=\"$INSTALL_DIR:\$PATH\"" >> "$RC_FILE"
#         echo "✓ Added PATH export to $RC_FILE"
#       fi
#     fi
#   else
#     echo ""
#     echo "To add $INSTALL_DIR to your PATH permanently, add this to $RC_FILE:"
#     echo "  export PATH=\"$INSTALL_DIR:\$PATH\""
#   fi
# fi

# echo ""
# echo "Installation complete! Run 'xyrev help' to get started."
