#!/usr/bin/env bash
set -euo pipefail

# Installs libvips for server-side image processing (thumbnails, resizing, format conversion).
# The tinyboards backend uses the `image` crate which can leverage libvips for performance.
#
# Usage: ./setup_image_processing.sh
#
# After installing, rebuild the backend with the image processing feature:
#   cd backend && cargo build --release
#
# In the Docker deployment, libvips is not needed on the host — the backend
# Dockerfile handles its own dependencies.

echo "Installing libvips and dependencies..."

# Detect package manager
if command -v apt-get &>/dev/null; then
    # Debian / Ubuntu
    sudo apt-get update
    sudo apt-get install -y --no-install-recommends \
        libvips-dev \
        libvips-tools \
        libvips42
    echo ""
    echo "Installed libvips (Debian/Ubuntu)."

elif command -v dnf &>/dev/null; then
    # Fedora / RHEL 8+
    sudo dnf install -y vips-devel vips-tools
    echo ""
    echo "Installed libvips (Fedora/RHEL)."

elif command -v yum &>/dev/null; then
    # CentOS / RHEL 7
    sudo yum install -y vips-devel vips-tools
    echo ""
    echo "Installed libvips (CentOS/RHEL)."

elif command -v pacman &>/dev/null; then
    # Arch Linux
    sudo pacman -S --noconfirm libvips
    echo ""
    echo "Installed libvips (Arch Linux)."

elif command -v apk &>/dev/null; then
    # Alpine
    sudo apk add --no-cache vips-dev vips-tools
    echo ""
    echo "Installed libvips (Alpine)."

else
    echo "Error: Could not detect package manager."
    echo "Please install libvips manually for your distribution."
    echo "See: https://www.libvips.org/install.html"
    exit 1
fi

# Verify installation
if command -v vips &>/dev/null; then
    echo ""
    echo "libvips version: $(vips --version)"
    echo ""
    echo "Image processing is ready. Rebuild the backend to use it:"
    echo "  cd backend && cargo build --release"
else
    echo ""
    echo "Warning: 'vips' command not found after installation."
    echo "The library may still be available for linking. Try building the backend."
fi
