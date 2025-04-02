#!/bin/bash

set -e  # Exit immediately if a command exits with a non-zero status

REPO_URL="https://github.com/clickswave/voyage.git"
DEFAULT_INSTALL_DIR="/opt/clickswave"
BUILD_DIR="/tmp/voyage_build"

# Parse arguments
INSTALL_DIR="$DEFAULT_INSTALL_DIR"
while [[ "$#" -gt 0 ]]; do
    case "$1" in
        --install-dir)
            INSTALL_DIR="$2"
            shift 2
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Ensure INSTALL_DIR is an absolute path
if command -v realpath &>/dev/null; then
    INSTALL_DIR="$(realpath "$INSTALL_DIR")"
elif command -v grealpath &>/dev/null; then
    INSTALL_DIR="$(grealpath -m "$INSTALL_DIR")"
elif command -v python3 &>/dev/null; then
    INSTALL_DIR="$(python3 -c "import os; print(os.path.abspath('$INSTALL_DIR'))")"
else
    echo "Error: realpath, grealpath, or Python 3 is required to resolve paths."
    exit 1
fi

VOYAGE_DIR="$INSTALL_DIR/voyage"
BIN_DIR="$INSTALL_DIR/bin"

# Ensure dependencies are installed
dep_check() {
    if ! command -v git &>/dev/null; then
        echo "Error: git is not installed. Please install git and try again."
        exit 1
    fi
    if ! command -v cargo &>/dev/null; then
        echo "Error: Rust (cargo) is not installed. Please install Rust and try again."
        exit 1
    fi
    if ! rustup show active-toolchain &>/dev/null; then
        echo "Error: Rust toolchain is not set. Please run the following command to set it up:"
        echo "  rustup default stable"
        exit 1
    fi
}

dep_check

# Create installation directories
sudo mkdir -p "$VOYAGE_DIR" "$BIN_DIR"
sudo chown -R "$USER:$(id -gn)" "$INSTALL_DIR"

# Create build directory in /tmp
rm -rf "$BUILD_DIR"
mkdir -p "$BUILD_DIR"
echo "Using build directory: $BUILD_DIR"
cd "$BUILD_DIR"

echo "Cloning Voyage repository..."
git clone "$REPO_URL" .

# Build the project
echo "Building Voyage..."
cargo build --release

# Move binary to voyage directory
mkdir -p "$VOYAGE_DIR"
echo "Installing Voyage..."
mv "target/release/voyage" "$VOYAGE_DIR/voyage"
chmod +x "$VOYAGE_DIR/voyage"

# Create symlink in bin directory
ln -sf "$VOYAGE_DIR/voyage" "$BIN_DIR/voyage"

echo "Creating PATH entry..."
CONFIG_FILES=(
    "$HOME/.bashrc"
    "$HOME/.zshrc"
    "$HOME/.config/fish/config.fish"
)

SHELL_FOUND=false
for CONFIG_FILE in "${CONFIG_FILES[@]}"; do
    if [ -f "$CONFIG_FILE" ]; then
        if grep -q "$BIN_DIR" "$CONFIG_FILE" 2>/dev/null; then
            echo "$BIN_DIR is already in PATH in $CONFIG_FILE"
        else
            echo "export PATH=\"$BIN_DIR:\$PATH\"" >> "$CONFIG_FILE"
            echo "Added $BIN_DIR to PATH in $CONFIG_FILE"
        fi
        SHELL_FOUND=true
    fi
done

if ! $SHELL_FOUND; then
    echo "No valid shell configuration files found. Please add the following line manually:"
    echo "  export PATH=\"$BIN_DIR:\$PATH\""
fi

# Clean up
echo "Cleaning up..."
rm -rf "$BUILD_DIR"

echo "Voyage installed successfully in $INSTALL_DIR! You can now run: voyage --help"
