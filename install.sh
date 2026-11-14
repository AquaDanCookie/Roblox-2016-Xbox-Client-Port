#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
binary="$root/target/release/roblox-2016-xbox"
if [[ ! -x "$binary" && -x "$root/roblox-2016-xbox" ]]; then
    binary="$root/roblox-2016-xbox"
fi
bin_dir="${HOME}/.local/bin"
applications_dir="${HOME}/.local/share/applications"
icon_file="${HOME}/.local/share/icons/roblox-2016-xbox.png"
icon_sizes=(16 24 32 48 64 128 208 256)
data_dir="${HOME}/.local/share/roblox-2016-xbox"

if [[ ! -x "$binary" ]]; then
    echo "Release binary not found at $binary" >&2
    echo "Build it first with: cargo build --release" >&2
    exit 1
fi

install -Dm755 "$binary" "$bin_dir/roblox-2016-xbox"
install -Dm644 "$root/packaging/roblox-2016-xbox.desktop" \
    "$applications_dir/roblox-2016-xbox.desktop"
install -Dm644 "$root/packaging/icons/256x256.png" "$icon_file"
for size in "${icon_sizes[@]}"; do
    install -Dm644 "$root/packaging/icons/${size}x${size}.png" \
        "${HOME}/.local/share/icons/hicolor/${size}x${size}/apps/roblox-2016-xbox.png"
done
sed -i "s|^Icon=.*|Icon=${icon_file}|" "$applications_dir/roblox-2016-xbox.desktop"

if [[ -d "$root/assets" ]]; then
    install -d "$data_dir"
    for category in music sounds background; do
        if [[ -d "$root/assets/$category" ]]; then
            install -d "$data_dir/$category"
            cp -an "$root/assets/$category/." "$data_dir/$category/"
        fi
    done
fi

if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database "$applications_dir" >/dev/null 2>&1 || true
fi
if command -v gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache -f -t "${HOME}/.local/share/icons/hicolor" >/dev/null 2>&1 || true
fi
if command -v kbuildsycoca6 >/dev/null 2>&1; then
    kbuildsycoca6 --noincremental >/dev/null 2>&1 || true
fi

echo "Installed:"
echo "  $bin_dir/roblox-2016-xbox"
echo "  $applications_dir/roblox-2016-xbox.desktop"
echo "  $icon_file"
echo "  hicolor app icons: ${icon_sizes[*]}"
if [[ -d "$root/assets" ]]; then
    echo "  media assets: $data_dir"
fi
echo
echo "If the launcher does not appear, make sure ~/.local/bin is in PATH."
echo "User data, music, and background videos are kept in ~/.local/share/roblox-2016-xbox."
