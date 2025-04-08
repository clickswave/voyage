#!/bin/sh
# POSIX compatible script for installing Voyage

set -e  # Exit immediately if a command exits with a non-zero status

REPO_URL="https://github.com/clickswave/voyage.git"
DEFAULT_INSTALL_DIR="/opt/clickswave"
BUILD_DIR="/tmp/voyage_build"

# Parse arguments
INSTALL_DIR="$DEFAULT_INSTALL_DIR"
while [ "$#" -gt 0 ]; do
    case "$1" in
        --install-dir)
            INSTALL_DIR="$2"
            shift 2
            ;;
        *)
            printf "Unknown option: %s\n" "$1" >&2
            exit 1
            ;;
    esac
done

# Ensure INSTALL_DIR is an absolute path
resolve_path() {
    case "$1" in
        /*)
            printf "%s\n" "$1"
            ;;
        *)
            pwd_val=$(pwd)
            printf "%s/%s\n" "$pwd_val" "$1"
            ;;
    esac
}

INSTALL_DIR=$(resolve_path "$INSTALL_DIR")

VOYAGE_DIR="$INSTALL_DIR/voyage"
BIN_DIR="$INSTALL_DIR/bin"

# Ensure dependencies are installed
dep_check() {
    if ! command -v git >/dev/null 2>&1; then
        printf "Error: git is not installed. Please install git and try again.\n" >&2
        exit 1
    fi
    if ! command -v cargo >/dev/null 2>&1; then
        printf "Error: Rust (cargo) is not installed. Please install Rust and try again.\n" >&2
        exit 1
    fi
    if ! rustup show active-toolchain >/dev/null 2>&1; then
        printf "Error: Rust toolchain is not set. Please run the following command to set it up:\n" >&2
        printf "  rustup default stable\n" >&2
        exit 1
    fi
}

dep_check

# Create installation directories
sudo mkdir -p "$VOYAGE_DIR" "$BIN_DIR"
sudo chown -R "$USER:$(id -g -n)" "$INSTALL_DIR"

# Create build directory in /tmp
rm -rf "$BUILD_DIR"
mkdir -p "$BUILD_DIR"
printf "Using build directory: %s\n" "$BUILD_DIR"
cd "$BUILD_DIR"

printf "Cloning Voyage repository...\n"
git clone "$REPO_URL" .

# Build the project
printf "Building Voyage...\n"
cargo build --release

# Move binary to voyage directory
mkdir -p "$VOYAGE_DIR"
printf "Installing voyage in %s\n" "$VOYAGE_DIR"
mv "target/release/voyage" "$VOYAGE_DIR/voyage"
chmod +x "$VOYAGE_DIR/voyage"

# Create symlink in bin directory
ln -sf "$VOYAGE_DIR/voyage" "$BIN_DIR/voyage"

printf "Creating PATH entry...\n"
CONFIG_FILES="$HOME/.bashrc:$HOME/.zshrc:$HOME/.config/fish/config.fish"
IFS=':'
SHELL_FOUND=false
for CONFIG_FILE in $CONFIG_FILES; do
    if [ -f "$CONFIG_FILE" ]; then
        if grep -q "$BIN_DIR" "$CONFIG_FILE" 2>/dev/null; then
            printf "%s is already in PATH in %s\n" "$BIN_DIR" "$CONFIG_FILE"
        else
            printf '%s\n' "export PATH=\"$BIN_DIR:\$PATH\"" >> "$CONFIG_FILE"
            printf "Added %s to PATH in %s\n" "$BIN_DIR" "$CONFIG_FILE"
        fi
        SHELL_FOUND=true
    fi
done
unset IFS

if [ ! "$SHELL_FOUND" = "true" ]; then
    printf "No valid shell configuration files found. Please add the following line manually:\n"
    printf "  export PATH=\"%s:\$PATH\"\n" "$BIN_DIR"
fi

# Clean up
printf "Cleaning up...\n"
rm -rf "$BUILD_DIR"

printf -- "--------------------------------------------------------------------\n"
printf -- "|      Voyage has been installed. Thank you for choosing us.       |\n"
printf -- "|                                                                  |\n"
printf -- "|        Please reopen your terminal or source your rc file        |\n"
printf -- "|                   For more info: voyage --help                   |\n"
printf -- "--------------------------------------------------------------------\n"