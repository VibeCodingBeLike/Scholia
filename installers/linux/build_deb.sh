#!/usr/bin/env bash
set -e

echo "=== Building Scholia Debian (.deb) Package ==="

PACKAGE_NAME="scholia"
VERSION="0.1.0"
ARCH="amd64"
DEB_DIR="target/debian/${PACKAGE_NAME}_${VERSION}_${ARCH}"

cargo build --release

rm -rf "${DEB_DIR}"
mkdir -p "${DEB_DIR}/DEBIAN"
mkdir -p "${DEB_DIR}/usr/bin"
mkdir -p "${DEB_DIR}/usr/share/applications"
mkdir -p "${DEB_DIR}/usr/share/icons/hicolor/scalable/apps"
mkdir -p "${DEB_DIR}/usr/share/doc/${PACKAGE_NAME}"

cp "target/release/scholia" "${DEB_DIR}/usr/bin/"
chmod 755 "${DEB_DIR}/usr/bin/scholia"

cp "installers/linux/scholia.desktop" "${DEB_DIR}/usr/share/applications/"
cp "assets/scholia-mla.svg" "${DEB_DIR}/usr/share/icons/hicolor/scalable/apps/scholia.svg"
cp "README.md" "${DEB_DIR}/usr/share/doc/${PACKAGE_NAME}/"
cp "LICENSE" "${DEB_DIR}/usr/share/doc/${PACKAGE_NAME}/copyright"

cat <<EOF > "${DEB_DIR}/DEBIAN/control"
Package: ${PACKAGE_NAME}
Version: ${VERSION}
Section: editors
Priority: optional
Architecture: ${ARCH}
Maintainer: Scholia Authors <info@scholia.editor>
Description: Modern, distraction-free MLA 9th edition document editor
 Scholia enforces strict adherence to MLA 9 guidelines while providing
 a customizable frosted glass interface, custom keybindings, and high-fidelity
 export to DOCX, HTML/PDF, and Markdown.
EOF

dpkg-deb --build "${DEB_DIR}" "dist/${PACKAGE_NAME}_${VERSION}_${ARCH}.deb"
echo "=== Created dist/${PACKAGE_NAME}_${VERSION}_${ARCH}.deb ==="
