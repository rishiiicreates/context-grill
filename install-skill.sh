#!/usr/bin/env bash
set -e

# ==============================================================================
# Context Grill: Universal AI Agent Skill Installer
# One-liner direct install for Antigravity, Claude Code, Cursor, Windsurf, OpenCode
# ==============================================================================

REPO="rishiiicreates/context-grill"
RAW_BASE="https://raw.githubusercontent.com/${REPO}/main"
HOME_DIR="${HOME:-$(eval echo ~)}"

echo ""
echo "🔥 Context Grill — Universal AI Agent Skill Installer"
echo "=========================================================="
echo "Installing context-grill skill to your agent runtimes..."
echo ""

# Create temporary staging directory
TMP_DIR=$(mktemp -d 2>/dev/null || mktemp -d -t 'cgrill')
cleanup() {
  rm -rf "$TMP_DIR"
}
trap cleanup EXIT

# 1. Obtain SKILL.md (local if running in repo, or download from GitHub)
SKILL_SOURCE=""
if [ -f "./SKILL.md" ]; then
  SKILL_SOURCE="./SKILL.md"
else
  echo "Downloading latest SKILL.md from GitHub..."
  if command -v curl >/dev/null 2>&1; then
    curl -fsSL "${RAW_BASE}/SKILL.md" -o "${TMP_DIR}/SKILL.md"
  elif command -v wget >/dev/null 2>&1; then
    wget -q "${RAW_BASE}/SKILL.md" -O "${TMP_DIR}/SKILL.md"
  else
    echo "Error: curl or wget required to download skill." >&2
    exit 1
  fi
  SKILL_SOURCE="${TMP_DIR}/SKILL.md"
fi

if [ ! -s "$SKILL_SOURCE" ]; then
  echo "Error: Failed to obtain valid SKILL.md." >&2
  exit 1
fi

INSTALLED_COUNT=0

# Helper to install skill to a directory
install_to() {
  local target_path="$1"
  local platform_name="$2"
  local target_dir
  target_dir=$(dirname "$target_path")

  mkdir -p "$target_dir"
  cp "$SKILL_SOURCE" "$target_path"
  echo "  ✓ ${platform_name}: Installed to ${target_path}"
  INSTALLED_COUNT=$((INSTALLED_COUNT + 1))
}

# 1. Antigravity (AGY) & Universal Agent Skills
install_to "${HOME_DIR}/.agents/skills/context-grill/SKILL.md" "Antigravity (AGY) & Universal Agent Skills"

# Also install to global Gemini config if directory exists
if [ -d "${HOME_DIR}/.gemini/config" ]; then
  install_to "${HOME_DIR}/.gemini/config/skills/context-grill/SKILL.md" "Antigravity Global Config"
fi

# 2. Claude Code
install_to "${HOME_DIR}/.claude/skills/context-grill/SKILL.md" "Claude Code"

# 3. Cursor
install_to "${HOME_DIR}/.cursor/skills/context-grill/SKILL.md" "Cursor"

# 4. Windsurf
install_to "${HOME_DIR}/.codeium/windsurf/skills/context-grill/SKILL.md" "Windsurf"

# 5. OpenCode
install_to "${HOME_DIR}/.opencode/skills/context-grill/SKILL.md" "OpenCode"

echo ""
echo "=========================================================="
echo "🎉 Successfully installed context-grill across ${INSTALLED_COUNT} agent runtime locations!"
echo ""
echo "Quick Start Commands:"
echo "  • Status:           cgrill skill status    (or npx context-grill skill status)"
echo "  • Ambiguity Scan:   cgrill scan \"<prompt>\" (or npx context-grill scan \"<prompt>\")"
echo "  • Runtime Check:    cgrill env             (or npx context-grill env)"
echo "  • Native MCP Mode:  cgrill mcp             (or npx context-grill mcp)"
echo ""
echo "Your AI coding agents in Antigravity, Claude Code, Cursor, Windsurf, and"
echo "OpenCode will now automatically interrogate vague prompts, verify runtime"
echo "state, and guard your files against line drift and bracket corruption."
echo ""
