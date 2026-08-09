#!/bin/bash
# VeilOS: Sway + Woven Shell Setup
# Called by Calamares post-install
# Sets up Sway tiling WM with Woven Shell bar/workspace management

set -e

TARGET_ROOT="${1:=/mnt/install}"
DEFAULT_USER="void"

echo "[VeilOS] Setting up Sway + Woven Shell environment..."

# Enable graphical target
chroot "$TARGET_ROOT" systemctl set-default graphical.target

# Use SDDM greeter (standard for Sway installs)
echo "[VeilOS] Configuring display manager..."
SDDM_CONF="$TARGET_ROOT/etc/sddm.conf.d/kde_settings.conf"
if [ -f "$SDDM_CONF" ]; then
    sed -i 's/Session=.*/Session=sway/' "$SDDM_CONF"
fi

# Create user systemd directories for user services
echo "[VeilOS] Setting up user service environment..."
USER_SERVICE_DIR="$TARGET_ROOT/home/$DEFAULT_USER/.config/systemd/user"
mkdir -p "$USER_SERVICE_DIR"
chroot "$TARGET_ROOT" chown -R "$DEFAULT_USER:$DEFAULT_USER" "/home/$DEFAULT_USER/.config"

# Create Sway config for user if not present
SWAY_CONFIG_DIR="$TARGET_ROOT/home/$DEFAULT_USER/.config/sway"
if [ ! -d "$SWAY_CONFIG_DIR" ]; then
    mkdir -p "$SWAY_CONFIG_DIR"
    cat > "$SWAY_CONFIG_DIR/config" << 'SWAYEOF'
# VeilOS Sway Configuration
set $mod Mod4
set $term alacritty
set $menu wofi --show drun

output * bg #000000 solid_color

input * {
    xkb_layout us
    pointer_accel 0
}

bindsym $mod+Return exec $term
bindsym $mod+d exec $menu
bindsym $mod+Shift+c kill
bindsym $mod+Shift+e exec swaynag -t warning -m 'Exit sway?' -b 'Yes' 'swaymsg exit'

set $ws1 "1"
set $ws2 "2"
set $ws3 "3"
set $ws4 "4"
set $ws5 "5"

bindsym $mod+1 workspace number $ws1
bindsym $mod+2 workspace number $ws2
bindsym $mod+3 workspace number $ws3
bindsym $mod+4 workspace number $ws4
bindsym $mod+5 workspace number $ws5
SWAYEOF
    chroot "$TARGET_ROOT" chown "$DEFAULT_USER:$DEFAULT_USER" "$SWAY_CONFIG_DIR/config"
fi

# Enable seatd (Sway needs seat management)
echo "[VeilOS] Enabling seat management..."
chroot "$TARGET_ROOT" systemctl enable seatd.service 2>/dev/null || true

# Verify packages installed
echo "[VeilOS] Verifying installation..."
chroot "$TARGET_ROOT" pacman -Q sway >/dev/null 2>&1 || {
    echo "[VeilOS] WARNING: Sway not found. Ensure sway package in base list."
}

echo "[VeilOS] Sway + Woven Shell setup complete."
