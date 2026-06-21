#!/usr/bin/env bash
# =============================================================================
# Kurulus – MCP Environment Setup Script
# =============================================================================
# Installs Bevy MCP servers and verifies the Rust project is correctly
# configured for remote debugging via the Model Context Protocol.
#
# Usage:
#   ./scripts/setup-mcp.sh            # interactive (default)
#   ./scripts/setup-mcp.sh --check     # only check, don't install
#   ./scripts/setup-mcp.sh --force     # reinstall even if already present
# =============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

SHOW_HEADER=true
MODE="install"

# ─── Parse arguments ────────────────────────────────────────────────────────
for arg in "$@"; do
  case "$arg" in
    --check)  MODE="check"  ;;
    --force)  MODE="force"  ;;
    --quiet)  SHOW_HEADER=false ;;
    --help)
      echo "Usage: $(basename "$0") [--check|--force|--quiet|--help]"
      exit 0
      ;;
  esac
done

# ─── Colors ─────────────────────────────────────────────────────────────────
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

ok()   { echo -e "  ${GREEN}✓${NC} $1"; }
warn() { echo -e "  ${YELLOW}⚠${NC} $1"; }
fail() { echo -e "  ${RED}✗${NC} $1"; }
info() { echo -e "  ${BLUE}→${NC} $1"; }
header() {
  if [ "$SHOW_HEADER" = true ]; then
    echo ""
    echo -e "${BLUE}══════════════════════════════════════════════${NC}"
    echo -e "${BLUE}  Kurulus — MCP Environment Setup${NC}"
    echo -e "${BLUE}══════════════════════════════════════════════${NC}"
    echo ""
  fi
}

# ─── Check prerequisites ────────────────────────────────────────────────────
check_prereqs() {
  local ok=true

  # cargo
  if command -v cargo &>/dev/null; then
    ok "cargo found at $(which cargo)"
  else
    fail "cargo not found — install Rust: https://rustup.rs"
    ok=false
  fi

  # rustc version
  if command -v rustc &>/dev/null; then
    local ver
    ver=$(rustc --version 2>/dev/null | awk '{print $2}')
    ok "rustc $ver"
  fi

  # opencode.json at project root
  if [ -f "$PROJECT_ROOT/opencode.json" ]; then
    ok "opencode.json found at project root"
  else
    warn "opencode.json not found — run this script from the project root"
  fi

  # cargo bin in PATH?
  local cargo_bin
  cargo_bin=$(dirname "$(which cargo)" 2>/dev/null)
  if echo "$PATH" | tr ':' '\n' | grep -q "$cargo_bin"; then
    :
  else
    warn "\$PATH may not include cargo's bin directory ($cargo_bin)"
    info "Add to ~/.bashrc:  export PATH=\"\$PATH:\$HOME/.cargo/bin\""
  fi

  echo ""
  return 0
}

# ─── Install/check bevy_brp_mcp ─────────────────────────────────────────────
install_brp_mcp() {
  local action="Installing"
  [ "$1" = "check" ] && action="Checking"
  [ "$1" = "force" ] && action="Reinstalling"

  header

  echo -e "${BLUE}  ── bevy_brp_mcp (BRP core via MCP) ──${NC}"
  echo ""

  if command -v bevy_brp_mcp &>/dev/null && [ "$1" != "force" ]; then
    local ver
    ver=$(bevy_brp_mcp --version 2>/dev/null || echo "?")
    ok "bevy_brp_mcp already installed ($(which bevy_brp_mcp))"
    info "Version: $ver"
    echo ""
    return 0
  fi

  if [ "$1" = "check" ]; then
    fail "bevy_brp_mcp is not installed"
    echo ""
    return 1
  fi

  info "$action bevy_brp_mcp via cargo (this may take a while)..."
  cargo install bevy_brp_mcp
  ok "bevy_brp_mcp installed at $(which bevy_brp_mcp)"
  echo ""
}

# ─── Install/check bevy-debugger-mcp ────────────────────────────────────────
install_debugger_mcp() {
  local action="Installing"
  [ "$1" = "check" ] && action="Checking"
  [ "$1" = "force" ] && action="Reinstalling"

  echo -e "${BLUE}  ── bevy-debugger-mcp (AI debugger) ──${NC}"
  echo ""

  if command -v bevy-debugger-mcp &>/dev/null && [ "$1" != "force" ]; then
    ok "bevy-debugger-mcp already installed ($(which bevy-debugger-mcp))"
    echo ""
    return 0
  fi

  if [ "$1" = "check" ]; then
    fail "bevy-debugger-mcp is not installed"
    echo ""
    return 1
  fi

  info "Installing bevy_debugger_mcp via cargo (this may take a while)..."
  cargo install bevy_debugger_mcp
  ok "bevy-debugger-mcp installed at $(which bevy-debugger-mcp)"
  echo ""
}

# ─── Verify Rust project BRP config ─────────────────────────────────────────
check_rust_config() {
  echo -e "${BLUE}  ── Rust Project BRP Configuration ──${NC}"
  echo ""

  local app_cargo="$PROJECT_ROOT/kurulus/kurulus-app/Cargo.toml"

  if [ ! -f "$app_cargo" ]; then
    warn "kurulus-app/Cargo.toml not found — skipping Rust config check"
    echo ""
    return 0
  fi

  # Check for 'remote' feature
  if grep -q '^\s*remote\s*=\s*\[]' "$app_cargo"; then
    ok "kurulus-app has 'remote' feature defined"
  else
    warn "kurulus-app does NOT have 'remote' feature — add it to [features]"
  fi

  # Check plugins.rs for RemotePlugin
  local plugins_rs="$PROJECT_ROOT/kurulus/kurulus-app/src/plugins.rs"
  if [ -f "$plugins_rs" ]; then
    if grep -q "RemotePlugin" "$plugins_rs" 2>/dev/null; then
      ok "plugins.rs includes RemotePlugin (behind cfg gate)"
    else
      warn "plugins.rs does NOT include RemotePlugin — BRP won't work"
    fi

    if grep -q "RemoteHttpPlugin" "$plugins_rs" 2>/dev/null; then
      ok "plugins.rs includes RemoteHttpPlugin (behind cfg gate)"
    else
      warn "plugins.rs does NOT include RemoteHttpPlugin — HTTP transport missing"
    fi
  else
    warn "plugins.rs not found at expected path"
  fi

  echo ""
}

# ─── Verify opencode.json config ───────────────────────────────────────────
check_opencode_config() {
  echo -e "${BLUE}  ── OpenCode MCP Configuration ──${NC}"
  echo ""

  local cfg="$PROJECT_ROOT/opencode.json"

  if [ ! -f "$cfg" ]; then
    fail "opencode.json not found at $cfg"
    echo ""
    return 1
  fi

  # Check for bevy-brp server
  if grep -q '"bevy-brp"' "$cfg" 2>/dev/null; then
    ok "opencode.json has 'bevy-brp' MCP server configured"
  else
    fail "opencode.json missing 'bevy-brp' MCP server entry"
  fi

  # Check for bevy-debugger server
  if grep -q '"bevy-debugger"' "$cfg" 2>/dev/null; then
    ok "opencode.json has 'bevy-debugger' MCP server configured"
  else
    fail "opencode.json missing 'bevy-debugger' MCP server entry"
  fi

  echo ""
}

# ─── Navigation helpers ─────────────────────────────────────────────────────
print_next_steps() {
  echo -e "${BLUE}══════════════════════════════════════════════${NC}"
  echo -e "${GREEN}  ✅ Setup complete!${NC}"
  echo -e "${BLUE}══════════════════════════════════════════════${NC}"
  echo ""
  echo "  Next steps:"
  echo ""
  echo "  ${YELLOW}1.${NC} Run the game with remote enabled:"
  echo "     cd kurulus && cargo run -p kurulus-app --features remote"
  echo ""
  echo "  ${YELLOW}2.${NC} Open OpenCode from the project root:"
  echo "     opencode ."
  echo ""
  echo "  ${YELLOW}3.${NC} Ask OpenCode to inspect your game ECS:"
  echo "     \"List all entities in the game scene\""
  echo "     \"Check the health of Kayi units\""
  echo "     \"Debug why morale is dropping\""
  echo ""
  echo "  ${YELLOW}4.${NC} Or connect the MCP servers to another agent:"
  echo "     The agent will auto-discover bevy_brp_mcp and bevy-debugger-mcp"
  echo ""
  echo -e "${BLUE}══════════════════════════════════════════════${NC}"
  echo ""
}

# ═══════════════════════════════════════════════════════════════════════════
#  MAIN
# ═══════════════════════════════════════════════════════════════════════════

header
check_prereqs

if [ "$MODE" = "check" ]; then
  echo -e "${BLUE}  Running in CHECK mode — no changes will be made${NC}"
  echo ""
  install_brp_mcp check || true
  install_debugger_mcp check || true
  check_rust_config
  check_opencode_config
  echo -e "${BLUE}  Check complete.${NC}"
  echo ""
else
  install_brp_mcp "$MODE"
  install_debugger_mcp "$MODE"
  check_rust_config
  check_opencode_config
  print_next_steps
fi
