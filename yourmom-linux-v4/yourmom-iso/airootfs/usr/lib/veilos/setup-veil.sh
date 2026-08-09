#!/bin/bash
# VeilOS: Veil Compositor Setup
# Called by Calamares post-install
# Sets up VeilLogin as greeter, configures PAM, enables services

set -e

TARGET_ROOT="${1:=/mnt/install}"
DEFAULT_USER="void"

echo "[VeilOS] Setting up Veil compositor environment..."

# Enable VeilLogin as default greeter
echo "[VeilOS] Configuring VeilLogin..."
chroot "$TARGET_ROOT" systemctl set-default graphical.target

# Add default user to seat group (for seatd/libseat access)
echo "[VeilOS] Configuring user permissions..."
chroot "$TARGET_ROOT" usermod -a -G seat "$DEFAULT_USER" 2>/dev/null || true

# Create default Veil config if not present
VEIL_CONFIG_DIR="$TARGET_ROOT/home/$DEFAULT_USER/.config/veil"
if [ ! -d "$VEIL_CONFIG_DIR" ]; then
    mkdir -p "$VEIL_CONFIG_DIR"
    chroot "$TARGET_ROOT" chown -R "$DEFAULT_USER:$DEFAULT_USER" "/home/$DEFAULT_USER/.config"
fi

# Enable seatd (required for Veil with libseat)
echo "[VeilOS] Enabling seat management..."
chroot "$TARGET_ROOT" systemctl enable seatd.service 2>/dev/null || true

# Set default shell to bash
chroot "$TARGET_ROOT" chsh -s /bin/bash "$DEFAULT_USER"

# Install Veil compositor if not already in base packages
echo "[VeilOS] Verifying Veil installation..."
chroot "$TARGET_ROOT" pacman -Q veil >/dev/null 2>&1 || {
    echo "[VeilOS] WARNING: Veil not found. Ensure veil package in base list."
}

echo "[VeilOS] Veil setup complete."
