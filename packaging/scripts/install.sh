#!/usr/bin/env bash
set -eou pipefail

APP="luz_optima"
BIN="luz_optimon"

ROOT_DIR="$(cd "$(dirname "$0")/.." && pwd)"

BIN_SRC="${ROOT_DIR}/bin/${BIN}"
BIN_DST="/usr/local/bin/${BIN}"

CONF_DIR="/etc/${APP}"
CONF_DST="${CONF_DIR}/schedule_file.json"

STATE_DIR="/var/lib/${APP}"
UNIT_DIR="/etc/systemd/system"

need_root() {
    if [[ "$(id -u)" -ne 0 ]]; then
        echo "Run as root: sudo ./install.sh"
    exit 1
    fi
}

need_root

echo "[1/6] Verifying bundle contents"
[[ -x "${BIN_SRC}" ]] || {echo "Missing binary: ${BIN_SRC}"; exit 1; }

echo "[2/6] Installing binary"
install -m 0755 "${BIN_SRC}" "${BIN_DST}"

echo "[3/6] Installing config (non-destructive)"
install -d -m 0755 "${CONF_DIR}"
if [[ ! -f "${CONF_DST}" ]]; then
    if [[ -f "${ROOT_DIR}/config/schedule_file.json"]]

echo "[3/6] Installing config (non-destructive)"
install -d -m 0755 "${CONF_DIR}"
if [[ ! -f "${CONF_DST}" ]]; then
    if [[ -f "${ROOT_DIR}/config/schedule_file.json" ]]; then
        install -m 0644 "${ROOT_DIR}/config/schedule_file.json" "${CONF_DST}"
    else
        echo "No default schedule_file.json in bundle; leaving config absent."
    fi
else
    echo "Config already exists; leaving: ${CONF_DST}"
fi

echo "[4/6] Creating state directory"
install -d -m 0755 "${STATE_DIR}"

echo "[5/6] Installing systemd units"
install -m 0644 "${ROOT_DIR}/systemd/${APP}.service" "${UNIT_DIR}/${APP}.service"
install -m 0644 "${ROOT_DIR}/systemd/${APP}-reload.service" "${UNIT_DIR}/${APP}-reload.service"
install -m 0644 "${ROOT_DIR}/systemd/${APP}-reload.path" "${UNIT_DIR}/${APP}-reload.path"

echo "[6/6] Enabling and starting"
systemctl daemon-reload
systemctl enable --now "${APP}.service"
systemctl enable --now "${APP}-reload.path"

echo "Installed. Logs: journalctl -u ${APP}.service -f"