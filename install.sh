#!/usr/bin/env bash
set -e

echo "=== BitChord Rust - Arch Linux Installer ==="

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIN_PATH="$SCRIPT_DIR/target/release/bitchord"

if [ ! -f "$BIN_PATH" ]; then
    echo "[*] Building release binary..."
    cargo build --release
fi

echo "[*] Installing to ~/.local/bin/bitchord-rust..."
mkdir -p "$HOME/.local/bin"
cp "$BIN_PATH" "$HOME/.local/bin/bitchord-rust"
chmod +x "$HOME/.local/bin/bitchord-rust"

echo "[*] Installing desktop shortcut..."
mkdir -p "$HOME/.local/share/applications"
sed "s|Exec=bitchord-rust|Exec=$HOME/.local/bin/bitchord-rust|g; s|Icon=bitchord-rust|Icon=$HOME/.local/share/icons/hicolor/256x256/apps/bitchord-rust.png|g" "$SCRIPT_DIR/bitchord-rust.desktop" > "$HOME/.local/share/applications/bitchord-rust.desktop"

mkdir -p "$HOME/.local/share/icons/hicolor/256x256/apps"
cp "$SCRIPT_DIR/assets/logo.png" "$HOME/.local/share/icons/hicolor/256x256/apps/bitchord-rust.png"

update-desktop-database "$HOME/.local/share/applications" 2>/dev/null || true

echo "=== Installation Complete! ==="
echo "You can now run 'bitchord-rust' from your terminal or application launcher (rofi/wofi)."
