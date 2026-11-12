#!/usr/bin/env bash
set -e

# Build script for macOS .app bundle and .dmg installer
echo "=== Building Scholia for macOS ==="

APP_NAME="Scholia"
VERSION=$(grep -m1 '^version' Cargo.toml | sed -E 's/.*"([^"]+)".*/\1/')
BUNDLE_DIR="target/release/bundle/osx/${APP_NAME}.app"
CONTENTS_DIR="${BUNDLE_DIR}/Contents"
MACOS_DIR="${CONTENTS_DIR}/MacOS"
RESOURCES_DIR="${CONTENTS_DIR}/Resources"

cargo build --release

echo "=== Creating macOS App Bundle ==="
rm -rf "${BUNDLE_DIR}"
mkdir -p "${MACOS_DIR}"
mkdir -p "${RESOURCES_DIR}"

cp "target/release/scholia" "${MACOS_DIR}/scholia"
chmod +x "${MACOS_DIR}/scholia"

sed "s/0.1.0/${VERSION}/g" "installers/macos/Info.plist" > "${CONTENTS_DIR}/Info.plist"
cp "assets/scholia-mla.svg" "${RESOURCES_DIR}/scholia.svg"

if command -v codesign &> /dev/null; then
    codesign --force --deep --sign - "${BUNDLE_DIR}" || true
fi

echo "=== App Bundle created at ${BUNDLE_DIR} ==="

if command -v create-dmg &> /dev/null; then
    echo "=== Generating DMG with create-dmg ==="
    mkdir -p "dist"
    create-dmg \
        --volname "${APP_NAME} Installer" \
        --window-pos 200 120 \
        --window-size 600 400 \
        --icon-size 100 \
        --app-drop-link 420 180 \
        "dist/${APP_NAME}-v${VERSION}-macOS.dmg" \
        "${BUNDLE_DIR}"
    echo "=== DMG created at dist/${APP_NAME}-v${VERSION}-macOS.dmg ==="
else
    echo "=== Note: create-dmg not found. App bundle is ready to be used or distributed. ==="
fi
