#!/bin/bash
# AUR PKGBUILD test harness using real makepkg
# Usage: bash aur-test-makepkg.sh /path/to/PKGBUILD

set -e

PKGBUILD_PATH="${1:?Usage: $0 /path/to/PKGBUILD}"
PKGBUILD_DIR="$(cd "$(dirname "$PKGBUILD_PATH")" && pwd)"
TEST_BUILD="$HOME/aur-test-build-$(date +%s)"
BUILD_LOG="$TEST_BUILD/build.log"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${BLUE}🏗️  Setting up isolated AUR build with makepkg...${NC}"
mkdir -p "$TEST_BUILD"

echo -e "${BLUE}📋 PKGBUILD location: $PKGBUILD_PATH${NC}"
echo -e "${BLUE}📁 Build directory: $TEST_BUILD${NC}"
echo -e "${BLUE}📝 Build log: $BUILD_LOG${NC}"
echo ""

# Copy PKGBUILD and any local files
cp "$PKGBUILD_PATH" "$TEST_BUILD/PKGBUILD"
if [ -d "$PKGBUILD_DIR/.git" ]; then
    echo -e "${BLUE}📦 Detected git repo, copying full directory...${NC}"
    cp -r "$PKGBUILD_DIR"/* "$TEST_BUILD/" 2>/dev/null || true
fi

echo ""
echo -e "${YELLOW}▶ Running makepkg (full build)...${NC}"
echo "====================================================" | tee "$BUILD_LOG"
echo ""

# Run makepkg in test directory with full output
(
    cd "$TEST_BUILD"
    
    # -s = install dependencies (with sudo)
    # -C = check integrity (skip SHA256 if not online)
    # -f = force build (even if pkg already exists)
    # Use 2>&1 to capture all output
    makepkg -f 2>&1 | tee -a "$BUILD_LOG"
    MAKEPKG_EXIT=${PIPESTATUS[0]}
    exit $MAKEPKG_EXIT
) || BUILD_EXIT=$?

BUILD_EXIT=${BUILD_EXIT:-0}

echo ""
echo "====================================================="
echo -e "${BLUE}🏗️  BUILD RESULTS${NC}"
echo "====================================================="
echo ""

if [ $BUILD_EXIT -eq 0 ]; then
    echo -e "${GREEN}✓ Build completed successfully${NC}"
else
    echo -e "${RED}✗ Build failed with exit code $BUILD_EXIT${NC}"
    echo -e "${RED}  This likely means missing dependencies or build errors.${NC}"
fi

echo ""

# Source PKGBUILD for metadata
source "$TEST_BUILD/PKGBUILD"

echo -e "${BLUE}📦 Package: $pkgname v$pkgver-$pkgrel${NC}"
echo -e "${BLUE}📝 Description: $pkgdesc${NC}"

echo ""
echo -e "${BLUE}📂 Build output:${NC}"
if [ -d "$TEST_BUILD/src" ]; then
    echo "  Source downloaded:"
    find "$TEST_BUILD/src" -maxdepth 1 -type d ! -name "src" | while read dir; do
        echo "    📁 $(basename "$dir")"
    done
else
    echo "  ✗ No src/ (download may have failed)"
fi

echo ""
echo -e "${BLUE}📦 Built packages:${NC}"
if ls "$TEST_BUILD"/*.pkg.tar.zst >/dev/null 2>&1; then
    ls -lh "$TEST_BUILD"/*.pkg.tar.zst | awk '{print "  " $9 " (" $5 ")"}'
    echo ""
    echo -e "${BLUE}📄 Package contents:${NC}"
    ls "$TEST_BUILD"/*.pkg.tar.zst | head -1 | xargs tar -tzf | head -20
else
    echo "  ✗ No .pkg.tar.zst created (build may have failed)"
fi

echo ""
echo -e "${BLUE}🔍 Key file checks:${NC}"
if [ -d "$TEST_BUILD/pkg" ]; then
    [ -f "$TEST_BUILD/pkg/usr/bin/woven" ] && echo "  ✓ Binary: /usr/bin/woven" || echo "  ✗ Binary NOT found"
    [ -f "$TEST_BUILD/pkg/usr/bin/woven-ctrl" ] && echo "  ✓ Binary: /usr/bin/woven-ctrl" || echo "  ✗ Binary NOT found"
    [ -f "$TEST_BUILD/pkg/usr/lib/systemd/user/woven.service" ] && echo "  ✓ Service: /usr/lib/systemd/user/woven.service" || echo "  ✗ Service NOT found"
    [ -f "$TEST_BUILD/pkg/usr/share/applications/woven-ctrl.desktop" ] && echo "  ✓ Desktop: /usr/share/applications/woven-ctrl.desktop" || echo "  ✗ Desktop NOT found"
    [ -f "$TEST_BUILD/pkg/usr/share/icons/hicolor/256x256/apps/woven.png" ] && echo "  ✓ Icon: /usr/share/icons/.../woven.png" || echo "  ✗ Icon NOT found"
    [ -f "$TEST_BUILD/pkg/etc/woven/woven.lua" ] && echo "  ✓ Config: /etc/woven/woven.lua" || echo "  ✗ Config NOT found"
    [ -d "$TEST_BUILD/pkg/usr/share/woven/runtime" ] && echo "  ✓ Runtime: /usr/share/woven/runtime/" || echo "  ✗ Runtime NOT found"
    [ -d "$TEST_BUILD/pkg/usr/share/woven/plugins" ] && echo "  ✓ Plugins: /usr/share/woven/plugins/" || echo "  ✗ Plugins NOT found"
else
    echo "  ✗ No pkg/ directory (package phase did not run)"
fi

echo ""
echo -e "${BLUE}⚙️  Dependencies:${NC}"
echo "  Runtime depends:"
for dep in "${depends[@]}"; do
    pacman -Qq "$dep" >/dev/null 2>&1 && echo "    ✓ $dep" || echo "    ✗ $dep (NOT installed)"
done
echo ""
echo "  Build depends:"
for dep in "${makedepends[@]}"; do
    pacman -Qq "$dep" >/dev/null 2>&1 && echo "    ✓ $dep" || echo "    ✗ $dep (NOT installed)"
done

echo ""
echo "====================================================="
echo -e "${YELLOW}📍 Next steps:${NC}"
echo ""
if [ $BUILD_EXIT -eq 0 ]; then
    echo -e "${GREEN}Build succeeded! You can:${NC}"
    echo ""
    pkg_file=$(ls "$TEST_BUILD"/*.pkg.tar.zst 2>/dev/null | head -1)
    if [ -n "$pkg_file" ]; then
        echo "  Test install the package:"
        echo "    sudo pacman -U '$pkg_file'"
        echo ""
    fi
    echo "  View full package contents:"
    echo "    tree -L 3 $TEST_BUILD/pkg"
    echo ""
    echo "  Upload to AUR:"
    echo "    cd $TEST_BUILD && git add -A && git commit -m 'v$pkgver-$pkgrel'"
else
    echo -e "${RED}Build failed. Check:${NC}"
    echo ""
    echo "  Full log:"
    echo "    cat $BUILD_LOG | tail -50"
    echo ""
    echo "  Specific error:"
    echo "    grep -i 'error\|failed' $BUILD_LOG"
fi

echo ""
echo -e "${YELLOW}View/cleanup:${NC}"
echo "  Full log: $BUILD_LOG"
echo "  Build dir: $TEST_BUILD"
echo "  Remove: rm -rf $TEST_BUILD"
echo ""
