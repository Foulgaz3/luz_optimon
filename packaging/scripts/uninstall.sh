#!/usr/bin/env bash
set -euo pipefail

APP="luz_optima"
BIN="luz_optimon"
UNIT_DIR="/etc/systemd/system"
PURGE=0

if [[ "${1:-}" == "--purge" ]]; then
  PURGE=1
fi

if [[ "$(id -u)" -ne 0 ]]; then
  echo "Run as root (example): sudo ./packaging/scripts/uninstall.sh [--purge]"
  exit 1
fi

command -v systemctl >/dev/null 2>&1 || { echo "systemctl not found"; exit 1; }

# Stop/disable (ignore errors if units aren't present)
systemctl stop "${APP}-reload.path" 2>/dev/null || true
systemctl stop "${APP}-reload.service" 2>/dev/null || true
systemctl stop "${APP}.service" 2>/dev/null || true

systemctl disable "${APP}-reload.path" 2>/dev/null || true
systemctl disable "${APP}.service" 2>/dev/null || true

# Remove unit files
rm -f \
  "${UNIT_DIR}/${APP}.service" \
  "${UNIT_DIR}/${APP}-reload.service" \
  "${UNIT_DIR}/${APP}-reload.path"

systemctl daemon-reload
systemctl reset-failed "${APP}.service" 2>/dev/null || true

# Remove installed artifacts
rm -f "/usr/local/bin/${BIN}"
rm -rf "/var/lib/${APP}"

if [[ "${PURGE}" -eq 1 ]]; then
  rm -rf "/etc/${APP}"
  echo "Config removed: /etc/${APP}"
else
  echo "Config kept at /etc/${APP} (use --purge to remove)."
fi

echo "Uninstalled."
