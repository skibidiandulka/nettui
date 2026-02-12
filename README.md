<div align="center">
  <h2>🖧 TUI for managing Wi-Fi and Ethernet</h2>
</div>

# nettui

`nettui` is a unified terminal UI for Wi-Fi and Ethernet.

It is a clean-room project inspired by the UX direction of tools like `impala` and `ethtui`, with one app shell and switchable Wi-Fi/Ethernet panels.  
This project was inspired by and builds upon ideas from Impala by pythops.

## ✨ Features

- One TUI with two transport tabs: `Wi-Fi` and `Ethernet`
- Startup tab policy: prefer active transport (`Ethernet` if active, else `Wi-Fi` if active)
- Wi-Fi workflow with split sections: `Known Networks`, `New Networks`, `Device`
- Non-blocking scan/connect with spinner feedback
- Connect/disconnect, forget, autoconnect toggle, hidden SSID connect
- Passphrase fallback flow when iwd reports `No Agent registered`
- Ethernet details + link up/down + DHCP renew
- Configurable keybinds via `~/.config/nettui/keybinds.toml`
- Toast/error popups and terminal size guard (`119x35` minimum)

## 💡 Prerequisites

- Linux
- `iwd` running and reachable on D-Bus
- `systemd-networkd` + `networkctl` available
- Nerd Fonts recommended for icon rendering

> [!IMPORTANT]
> To avoid network stack conflicts, keep one wireless manager in control. If `iwd` is your backend, avoid running overlapping managers for Wi-Fi (for example `NetworkManager` or `wpa_supplicant`) at the same time.

## 🚀 Installation

### crates.io

```bash
cargo install nettui
```

### Arch Linux (AUR source build)

```bash
yay -S nettui
```

### Arch Linux (AUR prebuilt binary)

```bash
yay -S nettui-bin
```

## 🪄 Usage

```bash
nettui
```

## ⌨️ Controls

Global:

- `h/l` or `←/→`: switch transport tab (`Wi-Fi` / `Ethernet`)
- `j/k` or `↓/↑`: move selection
- `r`: refresh (shows info toast)
- `q` or `Esc`: quit

Wi-Fi tab:

- `Tab` / `Shift+Tab`: switch focus (`Known` / `New` / `Device`)
- `s`: scan
- `Enter`: connect/disconnect selected network
- `a`: show/hide extra entries (`Known`: unavailable, `New`: hidden)
- `d`: forget selected known network
- `t`: toggle autoconnect for selected known network
- `n`: connect hidden network (in `New`)
- `i`: toggle Wi-Fi details popup
- Empty `New Networks` list shows `- no new networks -`

Ethernet tab:

- `Enter`: toggle selected interface link (`up/down`)
- `n`: renew DHCP on selected interface

## ⚙️ Keybind config

Config file path:

```bash
~/.config/nettui/keybinds.toml
```

On first launch, `nettui` auto-creates this file with defaults.

To reset from template:

```bash
mkdir -p ~/.config/nettui
cp /usr/share/doc/nettui/keybinds.toml.example ~/.config/nettui/keybinds.toml
```

Edit this file directly and restart `nettui` after changes.

## 🔄 Restart / control

`nettui` is not a `systemd` service, so `systemctl` does not apply.

Quick restart:

```bash
pkill -x nettui || true
omarchy-launch-or-focus-tui nettui
```

## 🧩 Omarchy integration

Launcher behavior depends on your local Omarchy scripts.

Check current behavior:

```bash
sed -n 1,220p ~/.local/share/omarchy/bin/omarchy-launch-wifi
sed -n 1,180p ~/.local/share/omarchy/bin/omarchy-launch-ethernet
```

By default on many setups:

- Wi-Fi click path prefers `impala`
- Ethernet fallback can use `ethtui` when available

That means installing `ethtui` alone usually does **not** replace Wi-Fi handling automatically.

To force `nettui` for network clicks:

```bash
sed -i 's/omarchy-launch-or-focus-tui impala/omarchy-launch-or-focus-tui nettui/g' ~/.local/share/omarchy/bin/omarchy-launch-wifi
sed -i 's/omarchy-launch-or-focus-tui ethtui/omarchy-launch-or-focus-tui nettui/g' ~/.local/share/omarchy/bin/omarchy-launch-ethernet
```

Optional Hyprland size rule for `org.omarchy.nettui`:

```bash
grep -q "match:class org.omarchy.nettui" ~/.config/hypr/apps/system.conf || echo "windowrule = size 1190 735, match:class org.omarchy.nettui" >> ~/.config/hypr/apps/system.conf
hyprctl reload
```

## 🛠️ Build

```bash
cargo build
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```

## ⚖️ License

`nettui` is licensed under `GPL-3.0-only`. See `LICENSE`.
