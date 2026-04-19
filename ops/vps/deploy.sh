#!/usr/bin/env bash

set -euo pipefail

APP_SERVICE="aenyrathia"
APP_USER="aenyrathia"
APP_DIR="/opt/aenyrathia/current"

if [[ "${EUID}" -ne 0 ]]; then
  echo "Please run as root (sudo)." >&2
  exit 1
fi

if [[ ! -d "${APP_DIR}" ]]; then
  echo "Missing app directory at ${APP_DIR}" >&2
  exit 1
fi

sudo -u "${APP_USER}" nix develop -c cargo build --release --manifest-path "${APP_DIR}/Cargo.toml"

systemctl restart "${APP_SERVICE}"
systemctl --no-pager --full status "${APP_SERVICE}"
