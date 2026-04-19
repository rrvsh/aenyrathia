#!/usr/bin/env bash

set -euo pipefail

APP_USER="aenyrathia"
APP_GROUP="aenyrathia"
APP_BASE_DIR="/opt/aenyrathia"
APP_CURRENT_DIR="${APP_BASE_DIR}/current"
APP_DATA_DIR="/var/lib/aenyrathia"
APP_ETC_DIR="/etc/aenyrathia"
SYSTEMD_UNIT_PATH="/etc/systemd/system/aenyrathia.service"
NGINX_AVAILABLE_PATH="/etc/nginx/sites-available/aenyrathia"
NGINX_ENABLED_PATH="/etc/nginx/sites-enabled/aenyrathia"

if [[ "${EUID}" -ne 0 ]]; then
  echo "Please run as root (sudo)." >&2
  exit 1
fi

if ! id -u "${APP_USER}" >/dev/null 2>&1; then
  useradd --system --create-home --home-dir "/home/${APP_USER}" --shell /usr/sbin/nologin "${APP_USER}"
fi

mkdir -p "${APP_BASE_DIR}" "${APP_DATA_DIR}" "${APP_ETC_DIR}"
chown -R "${APP_USER}:${APP_GROUP}" "${APP_BASE_DIR}" "${APP_DATA_DIR}" "${APP_ETC_DIR}"
chmod 0750 "${APP_DATA_DIR}" "${APP_ETC_DIR}"

if [[ ! -f "${APP_ETC_DIR}/aenyrathia.env" ]]; then
  install -m 0600 -o "${APP_USER}" -g "${APP_GROUP}" ./ops/vps/aenyrathia.env.example "${APP_ETC_DIR}/aenyrathia.env"
  sed -i "s|sqlite:///var/lib/aenyrathia/app.db|sqlite://${APP_DATA_DIR}/app.db|" "${APP_ETC_DIR}/aenyrathia.env"
fi

install -m 0644 ./ops/vps/aenyrathia.service "${SYSTEMD_UNIT_PATH}"
sed -i "s|^EnvironmentFile=.*|EnvironmentFile=${APP_ETC_DIR}/aenyrathia.env|" "${SYSTEMD_UNIT_PATH}"

install -m 0644 ./ops/vps/nginx.aenyrathia.conf "${NGINX_AVAILABLE_PATH}"

if [[ ! -L "${NGINX_ENABLED_PATH}" ]]; then
  ln -s "${NGINX_AVAILABLE_PATH}" "${NGINX_ENABLED_PATH}"
fi

nginx -t
systemctl daemon-reload
systemctl enable aenyrathia
echo "Bootstrap complete."
echo "Next steps:"
echo "1) Set your real domain in ${NGINX_AVAILABLE_PATH}."
echo "2) Add SSH deploy key at /home/${APP_USER}/.ssh/id_ed25519 and known_hosts."
echo "3) Build the binary as ${APP_USER} and deploy with ./ops/vps/deploy.sh."
echo "4) Restart services: systemctl restart aenyrathia nginx"
