<div align="center">
  <h2>🖧 nettui</h2>
  <p>TUI for Wi-Fi and Ethernet</p>
</div>

![nettui-showcase](https://github.com/user-attachments/assets/3603f7eb-433a-4641-bc38-700d93c67e9f)



`nettui` is a unified terminal UI for Wi-Fi and Ethernet.

It is heavily inspired by `impala` and `ethtui`, but built as one app shell with switchable `Wi-Fi` and `Ethernet` panels.

It is primarily meant for `Omarchy` and tested on `Omarchy`, but it should also work on other Linux distributions that use the same core stack:

- `iwd`
- `systemd-networkd`
- `networkctl`
- Nerd Fonts

## ✨ Features

- `Wi-Fi` and `Ethernet`
- Hidden SSID connect
- QR Wi-Fi sharing
- Wi-Fi power toggle
- Access point mode
- DHCP renew
- Configurable keybinds

## 💡 Prerequisites

- Linux
- `iwd` running on D-Bus
- `systemd-networkd`
- `networkctl`
- Nerd Fonts recommended

> [!IMPORTANT]
> `nettui` works best when `iwd` is the only active Wi-Fi manager. Avoid overlapping managers like `NetworkManager` or `wpa_supplicant`.

## 🚀 Installation

### crates.io

```bash
cargo install nettui
```

### Arch Linux (AUR source)

```bash
yay -S nettui
```

### Arch Linux (AUR binary)

```bash
yay -S nettui-bin
```

## 🪄 Usage

```bash
nettui
```

On first launch, `nettui` creates:

```bash
~/.config/nettui/keybinds.toml
```

## 🧩 Omarchy

Official Omarchy currently launches `impala` for Wi-Fi:

```bash
rfkill unblock wifi
omarchy-launch-or-focus-tui impala
```

To switch your local Omarchy install to `nettui`:

```bash
sed -i 's/omarchy-launch-or-focus-tui impala/omarchy-launch-or-focus-tui nettui/g' ~/.local/share/omarchy/bin/omarchy-launch-wifi
```

Verify:

```bash
sed -n '1,120p' ~/.local/share/omarchy/bin/omarchy-launch-wifi
```

Optional Hyprland rule:

```bash
grep -q "match:class org.omarchy.nettui" ~/.config/hypr/apps/system.conf || echo "windowrule = size 1190 735, match:class org.omarchy.nettui" >> ~/.config/hypr/apps/system.conf
hyprctl reload
```

## ⌨️ Controls

### Global

- `h/l` or `←/→`: switch tab
- `j/k` or `↓/↑`: move
- `r`: refresh
- `q` or `Esc`: quit

### Wi-Fi

- `Tab` / `Shift+Tab`: switch section
- `s`: scan
- `Enter`: connect or disconnect
- `a`: show all
- `d`: forget
- `y`: share
- `t`: autoconnect
- `n`: hidden network
- `i`: details
- `o`: power
- `p`: access point

### Ethernet

- `Enter`: link up/down
- `n`: renew DHCP

## 🩺 Notes

- `Access point` mode is hardware-dependent.
- Some adapters can scan and connect normally, but still fail in AP mode.
- For DHCP in AP mode, `/etc/iwd/main.conf` should enable:

```ini
[General]
EnableNetworkConfiguration=true
```

## 🔄 Restart

```bash
pkill -x nettui || true
omarchy-launch-or-focus-tui nettui
```

## 🛠️ Build

```bash
cargo build
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```

## 📦 Release Asset

Always build the GitHub release tarball with:

```bash
./scripts/build-release-asset.sh
```

## ⚖️ License

`nettui` is licensed under `GPL-3.0-only`. See `LICENSE`.
