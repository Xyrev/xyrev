#!/usr/bin/env bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
REPO_OWNER="szwathub"
REPO_NAME="xyrev"
BINARY_NAME="xyrev"
DEFAULT_INSTALL_DIR="${PREFIX:-$HOME/.local}/bin"

# Print colored messages
info() { echo -e "${GREEN}ℹ${NC} $*"; }
warn() { echo -e "${YELLOW}⚠${NC} $*"; }
error() { echo -e "${RED}✗${NC} $*" >&2; }
success() { echo -e "${GREEN}✓${NC} $*"; }

# Detect OS
detect_os() {
    case "$(uname -s)" in
        Darwin*)
            echo "macos"
            ;;
        Linux*)
            echo "linux"
            ;;
        MINGW*|MSYS*|CYGWIN*)
            echo "windows"
            ;;
        *)
            error "Unsupported operating system: $(uname -s)"
            exit 1
            ;;
    esac
}

# Detect architecture
detect_arch() {
    local arch="$(uname -m)"
    case "$arch" in
        x86_64|amd64)
            echo "x86_64"
            ;;
        aarch64|arm64)
            echo "aarch64"
            ;;
        i686|i386)
            echo "i686"
            ;;
        armv7*|armhf)
            echo "arm"
            ;;
        *)
            error "Unsupported architecture: $arch"
            exit 1
            ;;
    esac
}

# Detect libc type (for Linux)
detect_libc() {
    if [ "$(detect_os)" != "linux" ]; then
        echo ""
        return
    fi

    # Check if musl is available
    if ldd --version 2>&1 | grep -q musl; then
        echo "musl"
    else
        echo "gnu"
    fi
}

# Build target triple based on release.yml matrix
build_target() {
    local os="$1"
    local arch="$2"
    local libc="$3"

    case "$os" in
        macos)
            case "$arch" in
                x86_64) echo "x86_64-apple-darwin" ;;
                aarch64) echo "aarch64-apple-darwin" ;;
                *) error "Unsupported macOS architecture: $arch"; exit 1 ;;
            esac
            ;;
        linux)
            local libc_suffix=""
            if [ "$libc" = "musl" ]; then
                libc_suffix="-musl"
            else
                libc_suffix="-gnu"
            fi

            case "$arch" in
                x86_64) echo "x86_64-unknown-linux${libc_suffix}" ;;
                aarch64) echo "aarch64-unknown-linux${libc_suffix}" ;;
                i686) echo "i686-unknown-linux${libc_suffix}" ;;
                arm) echo "arm-unknown-linux-${libc_suffix}eabihf" ;;
                *) error "Unsupported Linux architecture: $arch"; exit 1 ;;
            esac
            ;;
        windows)
            case "$arch" in
                x86_64) echo "x86_64-pc-windows-msvc" ;;
                i686) echo "i686-pc-windows-msvc" ;;
                *) error "Unsupported Windows architecture: $arch"; exit 1 ;;
            esac
            ;;
        *)
            error "Unsupported OS: $os"
            exit 1
            ;;
    esac
}

# Get latest release version from GitHub
get_latest_version() {
    local api_url="https://api.github.com/repos/${REPO_OWNER}/${REPO_NAME}/releases/latest"

    if command -v curl >/dev/null 2>&1; then
        curl -fsSL "$api_url" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/'
    elif command -v wget >/dev/null 2>&1; then
        wget -qO- "$api_url" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/'
    else
        error "Neither curl nor wget found. Please install one of them."
        exit 1
    fi
}

# Download file
download_file() {
    local url="$1"
    local output="$2"

    info "Downloading from: $url"

    if command -v curl >/dev/null 2>&1; then
        if ! curl -fsSL "$url" -o "$output"; then
            error "Download failed"
            return 1
        fi
    elif command -v wget >/dev/null 2>&1; then
        if ! wget -qO "$output" "$url"; then
            error "Download failed"
            return 1
        fi
    else
        error "Neither curl nor wget found"
        return 1
    fi

    return 0
}

# Main installation function
main() {
    info "Installing ${BINARY_NAME}..."
    echo ""

    # Detect system
    local os="$(detect_os)"
    local arch="$(detect_arch)"
    local libc="$(detect_libc)"
    local target="$(build_target "$os" "$arch" "$libc")"

    info "Detected system:"
    echo "  OS: $os"
    echo "  Architecture: $arch"
    [ -n "$libc" ] && echo "  Libc: $libc"
    echo "  Target: $target"
    echo ""

    # Get version
    local version="${VERSION:-}"
    if [ -z "$version" ]; then
        info "Fetching latest version..."
        version="$(get_latest_version)"
        if [ -z "$version" ]; then
            error "Failed to fetch latest version"
            exit 1
        fi
    fi

    # Ensure version starts with 'v'
    case "$version" in
        v*) ;;
        *) version="v$version" ;;
    esac

    info "Version: $version"
    echo ""

    # Build download URL
    local pkg_suffix=".tar.gz"
    if [ "$os" = "windows" ]; then
        pkg_suffix=".zip"
    fi

    local pkg_name="${BINARY_NAME}-${version}-${target}${pkg_suffix}"
    local download_url="https://github.com/${REPO_OWNER}/${REPO_NAME}/releases/download/${version}/${pkg_name}"

    # Create temporary directory
    local tmp_dir="$(mktemp -d)"
    trap "rm -rf '$tmp_dir'" EXIT

    local tmp_archive="$tmp_dir/${pkg_name}"

    # Download archive
    if ! download_file "$download_url" "$tmp_archive"; then
        error "Failed to download ${pkg_name}"
        error "URL: $download_url"
        exit 1
    fi

    success "Downloaded successfully"
    echo ""

    # Download and verify SHA256 checksum
    info "Verifying SHA256 checksum..."
    local checksums_url="https://github.com/${REPO_OWNER}/${REPO_NAME}/releases/download/${version}/SHA256SUMS.txt"
    local tmp_checksums="$tmp_dir/SHA256SUMS.txt"

    if download_file "$checksums_url" "$tmp_checksums" 2>/dev/null; then
        # Extract expected checksum for our package
        local expected_checksum=""
        expected_checksum="$(grep "  ${pkg_name}$" "$tmp_checksums" | awk '{print $1}')"

        if [ -z "$expected_checksum" ]; then
            error "Checksum for ${pkg_name} not found in SHA256SUMS.txt"
            exit 1
        fi

        # Calculate actual checksum
        local actual_checksum=""
        if command -v sha256sum >/dev/null 2>&1; then
            actual_checksum="$(sha256sum "$tmp_archive" | awk '{print $1}')"
        elif command -v shasum >/dev/null 2>&1; then
            actual_checksum="$(shasum -a 256 "$tmp_archive" | awk '{print $1}')"
        else
            warn "No sha256sum or shasum found, skipping checksum validation"
        fi

        if [ -n "$actual_checksum" ]; then
            if [ "$actual_checksum" = "$expected_checksum" ]; then
                success "SHA256 checksum verified"
            else
                error "SHA256 checksum mismatch!"
                error "  Expected: $expected_checksum"
                error "  Actual:   $actual_checksum"
                exit 1
            fi
        fi
    else
        warn "Could not download SHA256SUMS.txt, skipping checksum validation"
    fi
    echo ""

    # Verify archive
    info "Verifying archive..."
    if [ "$os" = "windows" ]; then
        if ! command -v unzip >/dev/null 2>&1; then
            error "unzip command not found"
            exit 1
        fi
        if ! unzip -t "$tmp_archive" >/dev/null 2>&1; then
            error "Downloaded file is not a valid zip archive"
            exit 1
        fi
    else
        if ! tar -tzf "$tmp_archive" >/dev/null 2>&1; then
            error "Downloaded file is not a valid tarball"
            exit 1
        fi
    fi
    success "Archive verified"
    echo ""

    # Create installation directory
    local install_dir="$DEFAULT_INSTALL_DIR"
    info "Installing to: $install_dir"

    if ! mkdir -p "$install_dir"; then
        error "Could not create directory $install_dir"
        error "Try running with sudo or set PREFIX to a directory you own:"
        error "  export PREFIX=\$HOME/.local"
        exit 1
    fi

    # Extract and install
    local binary_suffix=""
    if [ "$os" = "windows" ]; then
        binary_suffix=".exe"
    fi

    local extract_dir="$tmp_dir/extract"
    mkdir -p "$extract_dir"

    if [ "$os" = "windows" ]; then
        unzip -q "$tmp_archive" -d "$extract_dir"
    else
        tar -xzf "$tmp_archive" -C "$extract_dir"
    fi

    # Find the binary (it might be in a subdirectory)
    local binary_path=""
    if [ -f "$extract_dir/${BINARY_NAME}${binary_suffix}" ]; then
        binary_path="$extract_dir/${BINARY_NAME}${binary_suffix}"
    else
        binary_path="$(find "$extract_dir" -name "${BINARY_NAME}${binary_suffix}" -type f | head -n 1)"
    fi

    if [ -z "$binary_path" ] || [ ! -f "$binary_path" ]; then
        error "Binary not found in archive"
        exit 1
    fi

    # Install binary
    if [ -f "$install_dir/${BINARY_NAME}${binary_suffix}" ]; then
        warn "Replacing existing binary at $install_dir/${BINARY_NAME}${binary_suffix}"
    fi

    cp "$binary_path" "$install_dir/${BINARY_NAME}${binary_suffix}"
    chmod +x "$install_dir/${BINARY_NAME}${binary_suffix}"

    success "Installed to $install_dir/${BINARY_NAME}${binary_suffix}"
    echo ""

    # Check if binary is in PATH
    if ! command -v "$BINARY_NAME" >/dev/null 2>&1; then
        warn "$install_dir is not in your PATH"
        echo ""

        # Detect shell
        local shell_name="$(basename "${SHELL:-/bin/sh}")"
        local rc_file=""

        case "$shell_name" in
            zsh)  rc_file="$HOME/.zshrc" ;;
            bash) rc_file="$HOME/.bashrc" ;;
            fish) rc_file="$HOME/.config/fish/config.fish" ;;
            *)    rc_file="$HOME/.profile" ;;
        esac

        echo "To add $install_dir to your PATH, add this to $rc_file:"
        echo ""
        if [ "$shell_name" = "fish" ]; then
            echo "  set -gx PATH $install_dir \$PATH"
        else
            echo "  export PATH=\"$install_dir:\$PATH\""
        fi
        echo ""
        echo "Then reload your shell:"
        echo "  source $rc_file"
    else
        success "${BINARY_NAME} is ready to use!"
    fi

    echo ""
    success "Installation complete!"
    echo ""
    echo "Run '${BINARY_NAME} --help' to get started."
}

# Run main function
main "$@"
