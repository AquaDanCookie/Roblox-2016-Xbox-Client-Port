#!/usr/bin/env bash
set -euo pipefail

bin="${HOME}/.local/bin/roblox-2016-xbox"
desktop="${HOME}/.local/share/applications/roblox-2016-xbox.desktop"
icon_file="${HOME}/.local/share/icons/roblox-2016-xbox.png"
icon_sizes=(16 24 32 48 64 128 208 256)

rm -f "$bin" "$desktop" "$icon_file"
for size in "${icon_sizes[@]}"; do
    rm -f "${HOME}/.local/share/icons/hicolor/${size}x${size}/apps/roblox-2016-xbox.png"
done

if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database "${HOME}/.local/share/applications" >/dev/null 2>&1 || true
fi

if [[ "${1:-}" == "--purge" ]]; then
    rm -rf \
        "${HOME}/.local/share/roblox-2016-xbox" \
        "${HOME}/.config/roblox-2016-xbox" \
        "${HOME}/.cache/roblox-2016-xbox" \
        "${HOME}/.local/state/roblox-2016-xbox"
    echo "Removed the installed app and local user data."
else
    echo "Removed the installed app."
    echo "User data, music, and background videos were kept."
    echo "Run with --purge to remove them as well."
fi
