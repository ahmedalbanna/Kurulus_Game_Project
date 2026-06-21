#!/usr/bin/env bash
# =============================================================================
# Kurulus – Project Setup Script
# =============================================================================
# Installs system dependencies, Rust toolchain, builds the project, and
# optionally installs MCP development tools.
#
# Usage:
#   ./setup.sh                    # full setup (interactive prompts for MCP)
#   ./setup.sh --check            # only check prerequisites, don't install
#   ./setup.sh --quick            # skip MCP tools, just deps + build
#   ./setup.sh --ci               # non-interactive, full install
#   ./setup.sh --help             # show help
# =============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$SCRIPT_DIR"

SHOW_HEADER=true
MODE="full"   # full | quick | check | ci

# ─── Parse arguments ────────────────────────────────────────────────────────
for arg in "$@"; do
  case "$arg" in
    --check) MODE="check"  ;;
    --quick) MODE="quick"  ;;
    --ci)    MODE="ci"     ;;
    --quiet) SHOW_HEADER=false ;;
    --help)
      echo "Kurulus — Project Setup Script"
      echo ""
      echo "Usage: $(basename "$0") [OPTION]"
      echo ""
      echo "Options:"
      echo "  --check    Check prerequisites only (no installs)"
      echo "  --quick    System deps + Rust + build, skip MCP tools"
      echo "  --ci       Non-interactive full install (no prompts)"
      echo "  --quiet    Suppress header banner"
      echo "  --help     Show this help"
      exit 0
      ;;
  esac
done

# ─── Colors ─────────────────────────────────────────────────────────────────
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
MAGENTA='\033[0;35m'
NC='\033[0m'

ok()     { echo -e "  ${GREEN}✓${NC} $1"; }
warn()   { echo -e "  ${YELLOW}⚠${NC} $1"; }
fail()   { echo -e "  ${RED}✗${NC} $1"; }
info()   { echo -e "  ${BLUE}→${NC} $1"; }
step()   { echo -e "\n${MAGENTA}── $1 ──${NC}"; }
header() {
  if [ "$SHOW_HEADER" = true ]; then
    echo ""
    echo -e "${BLUE}╔══════════════════════════════════════════════════╗${NC}"
    echo -e "${BLUE}║${NC}          ${YELLOW}Kurulus — Project Setup${NC}            ${BLUE}║${NC}"
    echo -e "${BLUE}║${NC}     ${GREEN}Bevy Game Engine — Ottoman Empire${NC}        ${BLUE}║${NC}"
    echo -e "${BLUE}╚══════════════════════════════════════════════════╝${NC}"
    echo ""
  fi
}

# ─── Helpers ────────────────────────────────────────────────────────────────
command_exists() { command -v "$1" &>/dev/null; }

# ─── Detect OS / package manager ────────────────────────────────────────────
detect_pkg_manager() {
  if   command_exists apt-get; then echo "apt"
  elif command_exists dnf;     then echo "dnf"
  elif command_exists yum;     then echo "yum"
  elif command_exists pacman;  then echo "pacman"
  elif command_exists zypper;  then echo "zypper"
  elif command_exists apk;     then echo "apk"
  else echo "unknown"
  fi
}

PKG_MANAGER=$(detect_pkg_manager)

# ─── Bevy system dependencies per distro ────────────────────────────────────
# Ref: https://bevyengine.org/learn/quick-start/getting-started/setup/
BEVY_APT_DEPS=(
  libasound2-dev
  libudev-dev
  libx11-dev
  libx11-xcb-dev
  libxcb-randr0-dev
  libxcb-shape0-dev
  libxcb-xfixes0-dev
  libxcb-xinput-dev
  libxkbcommon-dev
  libwayland-dev
)

BEVY_DNF_DEPS=(
  alsa-lib-devel
  systemd-devel
  libX11-devel
  libX11-xcb
  libxcb-devel
  libxkbcommon-devel
  wayland-devel
)

BEVY_PACMAN_DEPS=(
  alsa-lib
  systemd-libs
  libx11
  libxcb
  libxkbcommon
  wayland
)

BEVY_ZYPPER_DEPS=(
  alsa-devel
  systemd-devel
  libX11-devel
  libxcb-devel
  libxkbcommon-devel
  wayland-devel
)

BEVY_APK_DEPS=(
  alsa-lib-dev
  eudev-dev
  libx11-dev
  libxcb-dev
  libxkbcommon-dev
  wayland-dev
)

# ─── 1. System Dependencies ─────────────────────────────────────────────────
check_system_deps() {
  local missing=()

  case "$PKG_MANAGER" in
    apt)
      for pkg in "${BEVY_APT_DEPS[@]}"; do
        if ! dpkg -s "$pkg" &>/dev/null; then
          missing+=("$pkg")
        fi
      done
      ;;
    dnf|yum)
      local pkgs=("${BEVY_DNF_DEPS[@]}")
      for pkg in "${pkgs[@]}"; do
        if ! rpm -q "$pkg" &>/dev/null; then
          missing+=("$pkg")
        fi
      done
      ;;
    pacman)
      for pkg in "${BEVY_PACMAN_DEPS[@]}"; do
        if ! pacman -Qi "$pkg" &>/dev/null; then
          missing+=("$pkg")
        fi
      done
      ;;
    zypper)
      for pkg in "${BEVY_ZYPPER_DEPS[@]}"; do
        if ! rpm -q "$pkg" &>/dev/null; then
          missing+=("$pkg")
        fi
      done
      ;;
    apk)
      for pkg in "${BEVY_APK_DEPS[@]}"; do
        if ! apk info -e "$pkg" &>/dev/null; then
          missing+=("$pkg")
        fi
      done
      ;;
    *)
      warn "Unknown package manager — skipping system dependency check"
      warn "See: https://bevyengine.org/learn/quick-start/getting-started/setup/"
      return 0
      ;;
  esac

  if [ ${#missing[@]} -eq 0 ]; then
    ok "All Bevy system dependencies found"
  else
    fail "Missing Bevy system dependencies: ${missing[*]}"
    if [ "$MODE" != "check" ]; then
      info "Install with:  sudo $PKG_MANAGER install ${missing[*]}"
      if [ "$MODE" != "ci" ]; then
        echo ""
        read -rp "  Attempt to install missing packages? [Y/n] " yn
        case "$yn" in
          [Nn]*) warn "Skipping system dependency installation" ;;
          *)
            info "Installing missing system packages..."
            case "$PKG_MANAGER" in
              apt)   sudo apt-get update -qq && sudo apt-get install -y "${missing[@]}" ;;
              dnf)   sudo dnf install -y "${missing[@]}" ;;
              yum)   sudo yum install -y "${missing[@]}" ;;
              pacman) sudo pacman -S --noconfirm "${missing[@]}" ;;
              zypper) sudo zypper install -y "${missing[@]}" ;;
              apk)   sudo apk add "${missing[@]}" ;;
            esac
            ok "System dependencies installed"
            ;;
        esac
      else
        warn "Skipping — run manually: sudo $PKG_MANAGER install ${missing[*]}"
      fi
    fi
  fi
}

# ─── 2. Rust Toolchain ──────────────────────────────────────────────────────
check_rust() {
  # rustc
  if command_exists rustc; then
    local ver
    ver=$(rustc --version 2>/dev/null | awk '{print $2}')
    ok "rustc $ver"
  else
    fail "rustc not found"
    if [ "$MODE" != "check" ]; then
      info "Installing Rust via rustup..."
      curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal
      # Source the env for the remainder of this script
      # shellcheck source=/dev/null
      source "$HOME/.cargo/env"
      export PATH="$HOME/.cargo/bin:$PATH"
      ok "Rust installed via rustup"
    else
      fail "Install Rust: https://rustup.rs"
      return 1
    fi
  fi

  # cargo
  if command_exists cargo; then
    ok "cargo found at $(which cargo)"
  else
    fail "cargo not found"
    return 1
  fi

  # Verify Rust version >= 1.77 (Bevy 0.15 requires 1.77+)
  if command_exists rustc; then
    local ver_str ver_num major minor
    ver_str=$(rustc --version 2>/dev/null)
    # Extract "X.Y" from "rustc X.Y.Z (...)"
    ver_num=$(echo "$ver_str" | sed -n 's/^rustc \([0-9]\+\.[0-9]\+\).*/\1/p')
    if [ -n "$ver_num" ]; then
      major="${ver_num%.*}"
      minor="${ver_num#*.}"
      if [ "$major" -lt 1 ] || [ "$major" -eq 1 -a "$minor" -lt 77 ]; then
        warn "Bevy 0.15 requires Rust 1.77+. Found $ver_num. Consider: rustup update"
      else
        ok "Rust version meets Bevy requirement (>= 1.77)"
      fi
    fi
  fi
}

# ─── 3. Build Project ───────────────────────────────────────────────────────
build_project() {
  step "Building Kurulus Project"

  local cargo_args=()
  # In CI, use release for faster subsequent runs
  if [ "$MODE" = "ci" ]; then
    cargo_args+=("--release")
  fi

  echo ""

  # Build with default features (2D)
  info "Building with default features (2D)..."
  if (cd "$PROJECT_ROOT/kurulus" && cargo build "${cargo_args[@]}" -p kurulus-app 2>&1); then
    ok "Build succeeded (default features)"
  else
    fail "Build failed (default features)"
    return 1
  fi

  # Also check that remote feature compiles
  info "Verifying 'remote' feature compiles..."
  if (cd "$PROJECT_ROOT/kurulus" && cargo check "${cargo_args[@]}" -p kurulus-app --features remote 2>&1); then
    ok "remote feature compiles"
  else
    warn "remote feature failed to compile (BRP won't work)"
  fi

  return 0
}

# ─── 4. MCP Tools (optional) ────────────────────────────────────────────────
install_mcp_tools() {
  step "MCP Development Tools"

  local do_install=false
  if [ "$MODE" = "ci" ]; then
    do_install=true
  elif [ "$MODE" = "full" ]; then
    echo ""
    read -rp "  Install Bevy MCP tools (bevy_brp_mcp + bevy-debugger-mcp)? [y/N] " yn
    case "$yn" in
      [Yy]*) do_install=true ;;
    esac
  fi

  if [ "$do_install" = false ]; then
    info "Skipping MCP tool installation"
    return 0
  fi

  # bevy_brp_mcp
  if command_exists bevy_brp_mcp; then
    ok "bevy_brp_mcp already installed ($(which bevy_brp_mcp))"
  else
    info "Installing bevy_brp_mcp via cargo (this may take a while)..."
    cargo install bevy_brp_mcp
    ok "bevy_brp_mcp installed"
  fi

  # bevy-debugger-mcp
  if command_exists bevy-debugger-mcp; then
    ok "bevy-debugger-mcp already installed ($(which bevy-debugger-mcp))"
  else
    info "Installing bevy_debugger_mcp via cargo (this may take a while)..."
    cargo install bevy_debugger_mcp
    ok "bevy-debugger-mcp installed"
  fi
}

# ─── 5. Verify OpenCode Config ──────────────────────────────────────────────
check_opencode_config() {
  step "OpenCode Configuration"

  local cfg="$PROJECT_ROOT/opencode.json"
  if [ -f "$cfg" ]; then
    ok "opencode.json found"

    # Check MCP server entries
    if grep -q '"bevy-brp"' "$cfg" 2>/dev/null; then
      ok "MCP server 'bevy-brp' configured"
    else
      warn "Missing MCP server 'bevy-brp'"
    fi

    if grep -q '"bevy-debugger"' "$cfg" 2>/dev/null; then
      ok "MCP server 'bevy-debugger' configured"
    else
      warn "Missing MCP server 'bevy-debugger'"
    fi
  else
    fail "opencode.json not found — run this script from the project root"
  fi
}

# ─── 6. Print Summary ───────────────────────────────────────────────────────
print_summary() {
  echo ""
  echo -e "${BLUE}╔══════════════════════════════════════════════════╗${NC}"
  echo -e "${BLUE}║${NC}          ${GREEN}✅ Setup Complete!${NC}                  ${BLUE}║${NC}"
  echo -e "${BLUE}╚══════════════════════════════════════════════════╝${NC}"
  echo ""
  echo "  Quick start:"
  echo ""
  echo -e "  ${YELLOW}1.${NC} Run the game (2D mode — default):"
  echo "     cd kurulus && cargo run -p kurulus-app"
  echo ""
  echo -e "  ${YELLOW}2.${NC} Run with 3D rendering:"
  echo "     cd kurulus && cargo run -p kurulus-app --features render-3d"
  echo ""
  echo -e "  ${YELLOW}3.${NC} Run with remote debugging (for MCP / BRP):"
  echo "     cd kurulus && cargo run -p kurulus-app --features remote"
  echo ""
  echo -e "  ${YELLOW}4.${NC} Run tests:"
  echo "     cd kurulus && cargo test"
  echo ""
  echo -e "  ${YELLOW}5.${NC} Launch OpenCode:"
  echo "     opencode ."
  echo ""
  echo "  For details, see: docs/GDD.md"
  echo ""
}

# ═══════════════════════════════════════════════════════════════════════════
#  MAIN
# ═══════════════════════════════════════════════════════════════════════════

header

echo -e "${BLUE}  Project  :${NC} Kurulus — Ottoman Empire Simulation"
echo -e "${BLUE}  Engine   :${NC} Bevy 0.15 (Rust)"
echo -e "${BLUE}  Platform :${NC} $(uname -s) / $(uname -m)"
echo -e "${BLUE}  Mode     :${NC} $MODE"
echo ""

# Phase 1 — System dependencies
step "System Dependencies"
check_system_deps

# Phase 2 — Rust toolchain
step "Rust Toolchain"
check_rust

# Phase 3 — Build project
build_project || {
  fail "Project build failed — see errors above"
  exit 1
}

# Phase 4 — MCP tools (only in full/ci mode)
if [ "$MODE" = "full" ] || [ "$MODE" = "ci" ]; then
  install_mcp_tools
fi

# Phase 5 — Verify config
check_opencode_config

# Phase 6 — Summary
print_summary
