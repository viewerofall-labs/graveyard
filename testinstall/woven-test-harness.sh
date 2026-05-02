#!/bin/bash
# Woven installer test harness with live monitoring
# Usage: bash woven-test-harness.sh /path/to/get.sh

set -e

INSTALL_SCRIPT="${1:?Usage: $0 /path/to/get.sh}"
TEST_ENV="$HOME/woven-test-env"
MONITOR_LOG="$TEST_ENV/monitor.log"
INSTALL_LOG="$TEST_ENV/install.log"

echo "🧪 Setting up isolated test environment..."
rm -rf "$TEST_ENV"
mkdir -p "$TEST_ENV"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Monitor function: watches filesystem changes in real-time
monitor_changes() {
    local test_home="$1"
    local log_file="$2"
    
    echo -e "${BLUE}📊 MONITORING: Real-time filesystem changes${NC}"
    echo "====================================================" | tee -a "$log_file"
    
    # Initial snapshot
    find "$test_home" -type f 2>/dev/null | sort > "$test_home/.before" || true
    
    # Watch and report
    (
        while true; do
            sleep 0.5
            find "$test_home" -type f 2>/dev/null | sort > "$test_home/.after" || true
            
            # Show new files
            comm -13 "$test_home/.before" "$test_home/.after" | while read file; do
                echo -e "${GREEN}✓ Created${NC}: ${file#$test_home/}" | tee -a "$log_file"
            done
            
            # Show deleted files
            comm -23 "$test_home/.before" "$test_home/.after" | while read file; do
                echo -e "${RED}✗ Deleted${NC}: ${file#$test_home/}" | tee -a "$log_file"
            done
            
            cp "$test_home/.after" "$test_home/.before"
        done
    ) &
    MONITOR_PID=$!
}

echo -e "${BLUE}📁 Test environment: $TEST_ENV${NC}"
echo -e "${BLUE}📝 Install log: $INSTALL_LOG${NC}"
echo ""

# Start monitoring in background
monitor_changes "$TEST_ENV" "$MONITOR_LOG"
sleep 1

echo -e "${YELLOW}▶ Running installer with -x (debug mode)...${NC}"
echo "====================================================" | tee -a "$INSTALL_LOG"
echo ""

# Run installer with isolated HOME
(
    export HOME="$TEST_ENV"
    export XDG_CONFIG_HOME="$TEST_ENV/.config"
    export XDG_DATA_HOME="$TEST_ENV/.local/share"
    export XDG_CACHE_HOME="$TEST_ENV/.cache"
    export PATH="$TEST_ENV/.local/bin:$PATH"
    
    # Suppress sudo prompts (will fail gracefully)
    alias sudo='echo "[BLOCKED: sudo not allowed in test]" && false' 2>/dev/null || true
    
    bash -x "$INSTALL_SCRIPT" 2>&1 || true
) 2>&1 | tee -a "$INSTALL_LOG"

INSTALL_EXIT=$?

echo ""
echo -e "${YELLOW}⏹ Stopping monitor...${NC}"
kill $MONITOR_PID 2>/dev/null || true
sleep 1

echo ""
echo "====================================================="
echo -e "${BLUE}📊 INSTALLATION RESULTS${NC}"
echo "====================================================="
echo ""

if [ $INSTALL_EXIT -eq 0 ]; then
    echo -e "${GREEN}✓ Script completed successfully${NC}"
else
    echo -e "${RED}✗ Script failed with exit code $INSTALL_EXIT${NC}"
fi

echo ""
echo -e "${BLUE}📂 Directory structure created:${NC}"
tree -L 3 "$TEST_ENV" 2>/dev/null || find "$TEST_ENV" -type d | head -20

echo ""
echo -e "${BLUE}📄 Files created:${NC}"
find "$TEST_ENV" -type f 2>/dev/null | sed "s|$TEST_ENV|~|" | sort

echo ""
echo -e "${BLUE}🔍 Key paths to inspect:${NC}"
[ -f "$TEST_ENV/.config/woven/woven.lua" ] && echo "  ✓ Config: ~/.config/woven/woven.lua" || echo "  ✗ Config NOT found"
[ -d "$TEST_ENV/.local/bin" ] && echo "  ✓ Binaries: ~/.local/bin/" || echo "  ✗ Binaries dir NOT found"
[ -f "$TEST_ENV/.local/share/applications/woven-ctrl.desktop" ] && echo "  ✓ Desktop: ~/.local/share/applications/woven-ctrl.desktop" || echo "  ✗ Desktop NOT found"
[ -f "$TEST_ENV/.config/systemd/user/woven.service" ] && echo "  ✓ Service: ~/.config/systemd/user/woven.service" || echo "  ✗ Service NOT found"

echo ""
echo -e "${BLUE}📜 Shell RC updates:${NC}"
for rc in "$TEST_ENV"/.bashrc "$TEST_ENV"/.zshrc "$TEST_ENV"/.profile; do
    if [ -f "$rc" ]; then
        echo "  $(basename "$rc"):"
        grep "WOVEN_ROOT" "$rc" && echo "    ✓ WOVEN_ROOT found" || echo "    ✗ WOVEN_ROOT NOT found"
    fi
done

echo ""
echo "====================================================="
echo -e "${BLUE}🔗 Logs saved:${NC}"
echo "  Full install log: $INSTALL_LOG"
echo "  Monitor log: $MONITOR_LOG"
echo ""
echo -e "${YELLOW}View install output:${NC}"
echo "  cat $INSTALL_LOG"
echo ""
echo -e "${YELLOW}View directory tree:${NC}"
echo "  tree -L 3 $TEST_ENV"
echo ""
echo -e "${YELLOW}Cleanup test environment:${NC}"
echo "  rm -rf $TEST_ENV"
echo ""
