#!/usr/bin/env bash
set -e

# build_portable.sh
# Builds TheBestMLAWriter in Portable / Testing mode on macOS and Linux
echo "=== Building TheBestMLAWriter Portable / Testing Build ==="

cargo build --release

DIST_DIR="dist/TheBestMLAWriter-Portable"
rm -rf "${DIST_DIR}"
mkdir -p "${DIST_DIR}"

cp "target/release/the_best_mla_writer" "${DIST_DIR}/"
cp "README.md" "${DIST_DIR}/"
cp "LICENSE" "${DIST_DIR}/"
if [ -f "sample_mla_paper.mladoc" ]; then
    cp "sample_mla_paper.mladoc" "${DIST_DIR}/"
fi

echo "Portable binary ready at ${DIST_DIR}/the_best_mla_writer"
