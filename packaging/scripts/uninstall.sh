#!/usr/bin/env bash
set -euo pipefail

APP="luz_optima"
BIN="luz_optimon"
UNIT_DIR="/etc/systemd/system"

if [[ "$(id -u)" -ne 0 ]]; then
  echo "Run as root: sudo ./uninstall.sh"
  exit 1
fi

systemctl disable --now "${APP}-reload.path" 2>/dev/null || true
systemctl disable --now "${APP}.service" 2>/dev/null || true

rm -f \
  "${UNIT_DIR}/${APP}.service" \
  "${UNIT_DIR}/${APP}-reload.service" \
  "${UNIT_DIR}/${APP}-reload.path"

systemctl daemon-reload

rm -f "/usr/local/bin/${BIN}"
rm -rf "/var/lib/${APP}"

echo "Config kept at /etc/${APP} (remove manually if desired)."
echo "Uninstalled."
