#!/usr/bin/env bash
set -e

echo "=== BitChord Rust - Arch Linux Installer ==="

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIN_PATH="$SCRIPT_DIR/target/release/bitchord"

if [ ! -f "$BIN_PATH" ]; then
    echo "[*] Building release binary..."
    cargo build --release
fi

echo "[*] Installing to ~/.local/bin..."
mkdir -p "$HOME/.local/bin"
cp "$BIN_PATH" "$HOME/.local/bin/bitchord"
chmod +x "$HOME/.local/bin/bitchord"

echo "[*] Installing desktop shortcut..."
mkdir -p "$HOME/.local/share/applications"
sed "s|Exec=bitchord|Exec=$HOME/.local/bin/bitchord|g; s|Icon=bitchord|Icon=$HOME/.local/share/icons/hicolor/256x256/apps/bitchord.png|g" "$SCRIPT_DIR/bitchord.desktop" > "$HOME/.local/share/applications/bitchord.desktop"

mkdir -p "$HOME/.local/share/icons/hicolor/256x256/apps"
cp "$SCRIPT_DIR/assets/logo.png" "$HOME/.local/share/icons/hicolor/256x256/apps/bitchord.png"

echo "=== Installation Complete! ==="
echo "You can now run 'bitchord' from terminal, your app launcher, or rofi/wofi."
