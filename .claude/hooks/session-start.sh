#!/bin/bash
# SessionStart hook for Claude Code cloud sessions (Coder 2, 16 October 2026).
# Installs PowerShell 7 so the Windows scripts (scripts/*.ps1, tools/net/*.ps1)
# can be parsed and dry-run in the cloud before they reach Ayman's PC.
# Idempotent; cloud only. Nothing is changed on the PC by this file.
set -euo pipefail

if [ "${CLAUDE_CODE_REMOTE:-}" != "true" ]; then
  exit 0
fi

# Guard against untested engine code on main (advisor, 28 October 2026):
# commits and pushes to main go through .githooks/guard-main.py.
git -C "${CLAUDE_PROJECT_DIR:-$(cd "$(dirname "$0")/../.." && pwd)}" config core.hooksPath .githooks

PWSH_DIR="$HOME/pwsh"
PWSH_VERSION="7.4.6"
if [ ! -x "$PWSH_DIR/pwsh" ]; then
  tmp="$(mktemp -d)"
  curl -sSL -o "$tmp/pwsh.tar.gz" \
    "https://github.com/PowerShell/PowerShell/releases/download/v${PWSH_VERSION}/powershell-${PWSH_VERSION}-linux-x64.tar.gz"
  mkdir -p "$PWSH_DIR"
  tar -xzf "$tmp/pwsh.tar.gz" -C "$PWSH_DIR"
  chmod +x "$PWSH_DIR/pwsh"
  rm -rf "$tmp"
fi
echo "export PATH=\"$PWSH_DIR:\$PATH\"" >> "${CLAUDE_ENV_FILE:-/dev/null}"
"$PWSH_DIR/pwsh" -NoProfile -Command '"PowerShell " + $PSVersionTable.PSVersion.ToString() + " ready"'
