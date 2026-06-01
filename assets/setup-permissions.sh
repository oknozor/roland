#!/usr/bin/env bash
# Sets up the permissions needed to run roland without sudo.
# Run once as root (or via sudo), then log out and back in.
set -euo pipefail

UDEV_RULE_SRC="$(cd "$(dirname "$0")" && pwd)/99-roland-input.rules"
UDEV_RULE_DST="/etc/udev/rules.d/99-roland-input.rules"

echo "==> Installing udev rule to ${UDEV_RULE_DST}"
install -m 644 "${UDEV_RULE_SRC}" "${UDEV_RULE_DST}"

echo "==> Reloading udev rules"
udevadm control --reload-rules
udevadm trigger --subsystem-match=input

echo "==> Ensuring 'input' group exists"
getent group input >/dev/null || groupadd --system input

TARGET_USER="${SUDO_USER:-$USER}"
echo "==> Adding '${TARGET_USER}' to the 'input' group"
usermod -aG input "${TARGET_USER}"

echo ""
echo "Done. Log out and back in (or run 'newgrp input') for group membership to take effect."
